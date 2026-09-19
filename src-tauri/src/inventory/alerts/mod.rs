pub mod commands;
pub mod models;
use crate::{
    db::{
        catalog_validation::{id, invalid},
        connection::AppDb,
    },
    error::AppError,
};
use models::*;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::HashSet;
fn branch(c: &Connection, b: &str) -> Result<(), AppError> {
    id(b)?;
    if !c.query_row(
        "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1)",
        [b],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("Branch not found."));
    }
    Ok(())
}
pub fn business_date(c: &Connection, input: Option<&str>) -> Result<String, AppError> {
    let d = match input {
        Some(s) => s.to_owned(),
        None => c.query_row("SELECT date('now')", [], |r| r.get(0))?,
    };
    super::date(c, &d)?;
    Ok(d)
}
fn settings_on(c: &Connection, b: &str) -> Result<Settings, AppError> {
    branch(c, b)?;
    Ok(c.query_row("SELECT expiry_warning_days,critical_expiry_days,expiry_enabled,low_stock_enabled FROM inventory_alert_settings WHERE branch_id=?1",[b],|r|Ok(Settings{branch_id:b.into(),expiry_warning_days:r.get(0)?,critical_expiry_days:r.get(1)?,expiry_enabled:r.get(2)?,low_stock_enabled:r.get(3)?})).optional()?.unwrap_or(Settings{branch_id:b.into(),expiry_warning_days:90,critical_expiry_days:30,expiry_enabled:true,low_stock_enabled:true}))
}
pub fn settings(db: &AppDb, b: &str) -> Result<Settings, AppError> {
    settings_on(&*db.lock()?, b)
}
pub fn update_settings(db: &AppDb, s: &Settings) -> Result<Settings, AppError> {
    if !(0..=36500).contains(&s.expiry_warning_days)
        || !(0..=s.expiry_warning_days).contains(&s.critical_expiry_days)
    {
        return Err(invalid(
            "Use 0–36500 warning days and a critical threshold no larger than warning.",
        ));
    }
    let c = db.lock()?;
    branch(&c, &s.branch_id)?;
    c.execute("INSERT INTO inventory_alert_settings(branch_id,expiry_warning_days,critical_expiry_days,expiry_enabled,low_stock_enabled) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(branch_id) DO UPDATE SET expiry_warning_days=excluded.expiry_warning_days,critical_expiry_days=excluded.critical_expiry_days,expiry_enabled=excluded.expiry_enabled,low_stock_enabled=excluded.low_stock_enabled,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![s.branch_id,s.expiry_warning_days,s.critical_expiry_days,s.expiry_enabled,s.low_stock_enabled])?;
    settings_on(&c, &s.branch_id)
}
fn event(c: &Connection, id: &str, time: &str) -> Result<(), AppError> {
    c.execute("INSERT INTO inventory_alert_history(id,alert_id,status,severity,details_json,occurred_at) SELECT ?1,id,status,severity,details_json,?3 FROM inventory_alerts WHERE id=?2",params![uuid::Uuid::new_v4().to_string(),id,time])?;
    Ok(())
}
pub fn evaluate(db: &AppDb, b: &str, as_of: Option<&str>) -> Result<serde_json::Value, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let s = settings_on(&tx, b)?;
    let date = business_date(&tx, as_of)?;
    tx.execute(
        "INSERT OR IGNORE INTO inventory_alert_settings(branch_id) VALUES (?1)",
        [b],
    )?;
    let now:String=tx.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),coalesce((SELECT julianday(max(updated_at))+0.001/86400 FROM inventory_alerts WHERE branch_id=?1),0)))",[b],|r|r.get(0))?;
    let mut wanted: Vec<(String, String, Option<String>, String, serde_json::Value)> = Vec::new();
    if s.expiry_enabled {
        let mut q=tx.prepare("SELECT id,product_package_id,expiry_date,quantity,cast(julianday(expiry_date)-julianday(?2) AS INTEGER) FROM inventory_alert_batch_view WHERE branch_id=?1 AND is_active=1 AND quantity>0 AND expiry_date IS NOT NULL AND expiry_date<=date(?2,'+'||?3||' days')")?;
        for r in q.query_map(params![b, date, s.expiry_warning_days], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })? {
            let (batch, p, expiry, quantity, days) = r?;
            let (kind, severity) = if days < 0 {
                ("BATCH_EXPIRED", "CRITICAL")
            } else if days <= s.critical_expiry_days {
                ("BATCH_EXPIRING", "HIGH")
            } else {
                ("BATCH_EXPIRING", "MEDIUM")
            };
            wanted.push((kind.into(),p,Some(batch),severity.into(),serde_json::json!({"expiryDate":expiry,"quantity":quantity,"daysRemaining":days,"businessDate":date})));
        }
    }
    if s.low_stock_enabled {
        let mut q=tx.prepare("SELECT product_package_id,quantity,reorder_level FROM inventory_alert_level_view WHERE branch_id=?1 AND quantity<=reorder_level")?;
        for r in q.query_map([b], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })? {
            let (p, quantity, level) = r?;
            let (kind, severity) = if quantity == 0 {
                ("OUT_OF_STOCK", "CRITICAL")
            } else {
                ("LOW_STOCK", "MEDIUM")
            };
            wanted.push((
                kind.into(),
                p,
                None,
                severity.into(),
                serde_json::json!({"quantity":quantity,"reorderLevel":level}),
            ));
        }
    }
    let mut fingerprints = HashSet::new();
    let mut detected = 0;
    for (kind, p, batch, severity, details) in wanted {
        let fingerprint = format!("{b}:{kind}:{}", batch.as_deref().unwrap_or(&p));
        fingerprints.insert(fingerprint.clone());
        let details = details.to_string();
        let old: Option<(String, String, String, String)> = tx
            .query_row(
                "SELECT id,status,severity,details_json FROM inventory_alerts WHERE fingerprint=?1",
                [&fingerprint],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;
        if let Some((id, status, old_severity, old_details)) = old {
            let next = if status == "DISMISSED" {
                "DISMISSED"
            } else {
                "ACTIVE"
            };
            tx.execute("UPDATE inventory_alerts SET status=?2,severity=?3,details_json=?4,last_detected_at=?5,updated_at=?5,resolved_at=NULL WHERE id=?1",params![id,next,severity,details,now])?;
            if status != next || severity != old_severity || details != old_details {
                event(&tx, &id, &now)?;
            }
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            tx.execute("INSERT INTO inventory_alerts(id,branch_id,alert_type,product_package_id,batch_id,severity,status,fingerprint,details_json,first_detected_at,last_detected_at,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,'ACTIVE',?7,?8,?9,?9,?9,?9)",params![id,b,kind,p,batch,severity,fingerprint,details,now])?;
            event(&tx, &id, &now)?;
        }
        detected += 1;
    }
    let old = {
        let mut q = tx.prepare(
            "SELECT id,fingerprint FROM inventory_alerts WHERE branch_id=?1 AND status!='RESOLVED'",
        )?;
        let r = q
            .query_map([b], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        r
    };
    let mut resolved = 0;
    for (id, fingerprint) in old {
        if !fingerprints.contains(&fingerprint) {
            tx.execute("UPDATE inventory_alerts SET status='RESOLVED',resolved_at=?2,updated_at=?2 WHERE id=?1",params![id,now])?;
            event(&tx, &id, &now)?;
            resolved += 1;
        }
    }
    tx.commit()?;
    Ok(serde_json::json!({"businessDate":date,"detected":detected,"resolved":resolved}))
}
pub fn refresh_all(db: &AppDb) {
    let branches = (|| -> Result<Vec<String>, AppError> {
        let c = db.lock()?;
        let mut q = c.prepare("SELECT id FROM branches WHERE is_active=1")?;
        let rows = q.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
        Ok(rows)
    })();
    if let Ok(branches) = branches {
        for b in branches {
            if let Err(e) = evaluate(db, &b, None) {
                eprintln!("[inventory alerts] Advisory refresh failed: {}", e.message);
            }
        }
    }
}
const ALERT_COLUMNS:&str="a.id,a.branch_id,a.alert_type,a.product_package_id,a.batch_id,a.severity,a.status,a.fingerprint,a.details_json,a.first_detected_at,a.last_detected_at,a.resolved_at,a.updated_at,p.commercial_name_en,k.package_label,p.id";
fn map(r: &rusqlite::Row) -> rusqlite::Result<Alert> {
    let details: String = r.get(8)?;
    Ok(Alert {
        id: r.get(0)?,
        branch_id: r.get(1)?,
        alert_type: r.get(2)?,
        product_package_id: r.get(3)?,
        batch_id: r.get(4)?,
        severity: r.get(5)?,
        status: r.get(6)?,
        fingerprint: r.get(7)?,
        details: serde_json::from_str(&details).unwrap_or_default(),
        first_detected_at: r.get(9)?,
        last_detected_at: r.get(10)?,
        resolved_at: r.get(11)?,
        updated_at: r.get(12)?,
        product_name: r.get(13)?,
        product_id: r.get(15)?,
        package_label: r.get(14)?,
    })
}
pub fn list(db: &AppDb, q: &Query) -> Result<Page<Alert>, AppError> {
    let mut connection = db.lock()?;
    let c = connection.transaction()?;
    branch(&c, &q.branch_id)?;
    let (l, o) = super::page(q.limit, q.offset)?;
    for d in [&q.from_date, &q.to_date].into_iter().flatten() {
        super::date(&c, d)?;
    }
    if matches!((&q.from_date, &q.to_date), (Some(from), Some(to)) if from > to) {
        return Err(invalid("The start date must not follow the end date."));
    }
    let filter="a.branch_id=?1 AND (?2 IS NULL OR a.alert_type=?2) AND (?3 IS NULL OR a.severity=?3) AND (?4 IS NULL OR a.status=?4) AND (?5 IS NULL OR substr(a.updated_at,1,10)>=?5) AND (?6 IS NULL OR substr(a.updated_at,1,10)<=?6)";
    let args = params![
        q.branch_id,
        value(q.alert_type),
        value(q.severity),
        value(q.status),
        q.from_date,
        q.to_date
    ];
    let total = c.query_row(
        &format!("SELECT count(*) FROM inventory_alerts a WHERE {filter}"),
        args,
        |r| r.get(0),
    )?;
    let mut stmt=c.prepare(&format!("SELECT {ALERT_COLUMNS} FROM inventory_alerts a JOIN product_packages k ON k.id=a.product_package_id JOIN products p ON p.id=k.product_id WHERE {filter} ORDER BY a.updated_at DESC,a.id LIMIT ?7 OFFSET ?8"))?;
    let items = stmt
        .query_map(
            params![
                q.branch_id,
                value(q.alert_type),
                value(q.severity),
                value(q.status),
                q.from_date,
                q.to_date,
                l,
                o
            ],
            map,
        )?
        .collect::<Result<_, _>>()?;
    Ok(Page {
        items,
        total,
        business_date: business_date(&c, None)?,
    })
}
pub fn get(db: &AppDb, b: &str, alert: &str) -> Result<serde_json::Value, AppError> {
    let c = db.lock()?;
    branch(&c, b)?;
    id(alert)?;
    let row=c.query_row(&format!("SELECT {ALERT_COLUMNS} FROM inventory_alerts a JOIN product_packages k ON k.id=a.product_package_id JOIN products p ON p.id=k.product_id WHERE a.branch_id=?1 AND a.id=?2"),params![b,alert],map)?;
    let mut q=c.prepare("SELECT status,severity,details_json,occurred_at FROM inventory_alert_history WHERE alert_id=?1 ORDER BY occurred_at,id")?;
    let history=q.query_map([alert],|r|Ok(serde_json::json!({"status":r.get::<_,String>(0)?,"severity":r.get::<_,String>(1)?,"details":serde_json::from_str::<serde_json::Value>(&r.get::<_,String>(2)?).unwrap_or_default(),"occurredAt":r.get::<_,String>(3)?})))?.collect::<Result<Vec<_>,_>>()?;
    Ok(serde_json::json!({"alert":row,"history":history}))
}
pub fn dismiss(db: &AppDb, b: &str, alert: &str) -> Result<(), AppError> {
    let mut c = db.lock()?;
    branch(&c, b)?;
    id(alert)?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now: String = tx.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),coalesce((SELECT julianday(max(updated_at))+0.001/86400 FROM inventory_alerts WHERE branch_id=?1),0)))", [b], |r| r.get(0))?;
    if tx.execute("UPDATE inventory_alerts SET status='DISMISSED',updated_at=?3 WHERE branch_id=?1 AND id=?2 AND status='ACTIVE'",params![b,alert,now])?!=1{return Err(invalid("Only an active alert in this branch can be dismissed."));}
    event(&tx, alert, &now)?;
    tx.commit()?;
    Ok(())
}
pub fn monitor(db: &AppDb, q: &MonitorQuery, low: bool) -> Result<Page<Monitor>, AppError> {
    let mut connection = db.lock()?;
    let c = connection.transaction()?;
    branch(&c, &q.branch_id)?;
    let date = business_date(&c, q.business_date.as_deref())?;
    let (l, o) = super::page(q.limit, q.offset)?;
    let min = q.minimum_days.unwrap_or(0);
    let max = q.maximum_days.unwrap_or(90);
    if !(0..=36500).contains(&min) || !(min..=36500).contains(&max) {
        return Err(invalid("Invalid expiry window."));
    }
    let (base, filter, columns) = if low {
        ("inventory_alert_level_view v JOIN product_packages k ON k.id=v.product_package_id","v.branch_id=?1 AND v.quantity<=v.reorder_level AND (?3=0 OR v.quantity=0)","v.branch_id,v.product_package_id,NULL,v.product_name,v.package_label,NULL,NULL,NULL,v.quantity,NULL,v.reorder_level,v.minimum_stock,v.maximum_stock")
    } else {
        ("inventory_alert_batch_view v JOIN product_packages k ON k.id=v.product_package_id","v.branch_id=?1 AND v.is_active=1 AND v.quantity>0 AND v.expiry_date IS NOT NULL AND ((?3=1 AND v.expiry_date<?2) OR (?3=0 AND v.expiry_date BETWEEN date(?2,'+'||?4||' days') AND date(?2,'+'||?5||' days')))","v.branch_id,v.product_package_id,v.id,v.product_name,v.package_label,v.lot_number,v.expiry_date,cast(julianday(v.expiry_date)-julianday(?2) AS INTEGER),v.quantity,v.cost_price_minor,NULL,NULL,NULL")
    };
    let flag = if low { q.out_of_stock } else { q.expired };
    let sql = if low {
        format!("SELECT {columns},a.status,a.severity,k.product_id FROM {base} LEFT JOIN inventory_alerts a ON a.fingerprint=v.branch_id||':'||CASE WHEN v.quantity=0 THEN 'OUT_OF_STOCK' ELSE 'LOW_STOCK' END||':'||v.product_package_id WHERE {filter} AND (?2 IS NOT NULL) AND (?4<=?5)")
    } else {
        format!("SELECT {columns},a.status,a.severity,k.product_id FROM {base} LEFT JOIN inventory_alerts a ON a.fingerprint=v.branch_id||':'||CASE WHEN v.expiry_date<?2 THEN 'BATCH_EXPIRED' ELSE 'BATCH_EXPIRING' END||':'||v.id WHERE {filter}")
    };
    let total = c.query_row(
        &format!("SELECT count(*) FROM ({sql})"),
        params![q.branch_id, date, flag, min, max],
        |r| r.get(0),
    )?;
    let mut s = c.prepare(&format!(
        "{sql} ORDER BY {} LIMIT ?6 OFFSET ?7",
        if low {
            "v.product_package_id"
        } else {
            "v.expiry_date,v.id"
        }
    ))?;
    let items = s
        .query_map(params![q.branch_id, date, flag, min, max, l, o], |r| {
            Ok(Monitor {
                branch_id: r.get(0)?,
                product_package_id: r.get(1)?,
                batch_id: r.get(2)?,
                product_name: r.get(3)?,
                product_id: r.get(15)?,
                package_label: r.get(4)?,
                lot_number: r.get(5)?,
                expiry_date: r.get(6)?,
                days_remaining: r.get(7)?,
                quantity: r.get(8)?,
                cost_price_minor: r.get(9)?,
                reorder_level: r.get(10)?,
                minimum_stock: r.get(11)?,
                maximum_stock: r.get(12)?,
                status: r.get(13)?,
                severity: r.get(14)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(Page {
        items,
        total,
        business_date: date,
    })
}
