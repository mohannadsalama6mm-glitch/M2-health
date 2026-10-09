//! Phase 6 read-only integration, reconciliation, traceability, and CSV export.
pub mod commands;
pub mod models;

use crate::{
    db::{
        catalog_validation::{id, invalid},
        connection::AppDb,
    },
    error::{AppError, ErrorCode},
    finance, reporting,
};
use models::{CsvExportInput, Phase6Filter};
use rusqlite::params;
use serde_json::{json, Value};

pub fn reconciliation(db: &AppDb, filter: &Phase6Filter) -> Result<Value, AppError> {
    // Reuse Reporting validation so branch and inclusive date semantics cannot drift.
    reporting::overview(db, &filter.report())?;
    let c = db.lock()?;
    let p = params![filter.branch_id, filter.date_from, filter.date_to];
    let sales: (i64, i64) = c.query_row(
        "SELECT count(*),coalesce(sum(CASE WHEN j.id IS NOT NULL THEN 1 ELSE 0 END),0) FROM sales s LEFT JOIN finance_journals j ON j.source_type='SALE' AND j.source_id=s.id AND j.status='POSTED' WHERE s.status IN ('POSTED','VOIDED') AND (?1 IS NULL OR s.branch_id=?1) AND (?2 IS NULL OR substr(s.posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(s.posted_at,1,10)<=?3)",
        p,
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let returns: (i64, i64) = c.query_row(
        "SELECT count(*),coalesce(sum(CASE WHEN j.id IS NOT NULL THEN 1 ELSE 0 END),0) FROM sale_returns r JOIN sales s ON s.id=r.sale_id LEFT JOIN finance_journals j ON j.source_type='SALE_RETURN' AND j.source_id=r.id AND j.status='POSTED' WHERE r.status='POSTED' AND (?1 IS NULL OR s.branch_id=?1) AND (?2 IS NULL OR substr(r.posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(r.posted_at,1,10)<=?3)",
        params![filter.branch_id, filter.date_from, filter.date_to],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let voids: (i64, i64) = c.query_row(
        "SELECT count(*),coalesce(sum(CASE WHEN j.id IS NOT NULL THEN 1 ELSE 0 END),0) FROM sale_voids v JOIN sales s ON s.id=v.sale_id LEFT JOIN finance_journals j ON j.source_type='SALE_VOID' AND j.source_id=v.id AND j.status='POSTED' WHERE (?1 IS NULL OR s.branch_id=?1) AND (?2 IS NULL OR substr(v.created_at,1,10)>=?2) AND (?3 IS NULL OR substr(v.created_at,1,10)<=?3)",
        params![filter.branch_id, filter.date_from, filter.date_to],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let missing = sales.0 - sales.1 + returns.0 - returns.1 + voids.0 - voids.1;
    Ok(json!({
        "eligibleSaleCount": sales.0, "representedSaleCount": sales.1,
        "unrepresentedSaleCount": sales.0-sales.1,
        "postedReturnCount": returns.0, "representedReturnCount": returns.1,
        "unrepresentedReturnCount": returns.0-returns.1,
        "voidCount": voids.0, "representedVoidCount": voids.1,
        "unrepresentedVoidCount": voids.0-voids.1,
        "unrepresentedEventCount": missing, "coverageComplete": missing == 0,
        "scopeNote": "Finance includes only operational source events with posted Finance journals. Historical operational records were not backfilled."
    }))
}

pub fn dashboard(db: &AppDb, filter: &Phase6Filter) -> Result<Value, AppError> {
    let operational = reporting::overview(db, &filter.report())?;
    let financial = finance::sales_summary(db, &filter.finance())?;
    let coverage = reconciliation(db, filter)?;
    Ok(json!({"operational": operational, "financial": financial, "coverage": coverage}))
}

pub fn sale_finance_trace(db: &AppDb, sale_id: &str) -> Result<Value, AppError> {
    id(sale_id)?;
    let c = db.lock()?;
    let exists = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sales WHERE id=?1)",
        [sale_id],
        |r| r.get::<_, bool>(0),
    )?;
    if !exists {
        return Err(AppError::new(ErrorCode::NotFound, "Sale not found."));
    }
    let mut q = c.prepare("SELECT json_object('id',id,'journalNumber',journal_number,'sourceType',source_type,'sourceId',source_id,'sourceParentId',source_parent_id,'description',description,'debitTotalMinor',debit_total_minor,'creditTotalMinor',credit_total_minor,'cogsComplete',json(CASE cogs_complete WHEN 1 THEN 'true' ELSE 'false' END),'postedAt',posted_at) FROM finance_journals WHERE status='POSTED' AND ((source_type='SALE' AND source_id=?1) OR (source_type IN ('SALE_RETURN','SALE_VOID') AND source_parent_id=?1)) ORDER BY posted_at,id")?;
    let journals = q
        .query_map([sale_id], |r| {
            let text: String = r.get(0)?;
            serde_json::from_str(&text).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?
        .collect::<Result<Vec<Value>, _>>()?;
    let sale_posted = journals.iter().any(|j| j["sourceType"] == "SALE");
    Ok(
        json!({"saleId":sale_id,"financePosted":sale_posted,"historicalUnposted":!sale_posted,"journals":journals,"coverageNote":if sale_posted {"This sale has a posted Finance journal."} else {"No Finance journal exists for this sale. It may be historical operational data that was intentionally not backfilled."}}),
    )
}

type Column = (&'static str, &'static str, bool);
fn csv_cell(value: &str) -> String {
    let mut value = value.replace('\r', " ").replace('\n', " ");
    if value.trim_start().starts_with(['=', '+', '-', '@']) {
        value.insert(0, '\'');
    }
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}
fn money(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let n = i128::from(value).abs();
    format!("{sign}{}.{:02} EGP", n / 100, n % 100)
}
fn render_csv(rows: &[Value], columns: &[Column]) -> String {
    let mut lines = vec![columns
        .iter()
        .map(|(_, h, _)| csv_cell(h))
        .collect::<Vec<_>>()
        .join(",")];
    for row in rows {
        lines.push(
            columns
                .iter()
                .map(|(key, _, cash)| {
                    let v = &row[*key];
                    let text = if *cash {
                        money(v.as_i64().unwrap_or(0))
                    } else if let Some(s) = v.as_str() {
                        s.to_owned()
                    } else if v.is_null() {
                        String::new()
                    } else {
                        v.to_string()
                    };
                    csv_cell(&text)
                })
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    lines.join("\r\n") + "\r\n"
}
fn collect_pages<F>(mut load: F) -> Result<Vec<Value>, AppError>
where
    F: FnMut(i64) -> Result<Value, AppError>,
{
    let mut rows = Vec::new();
    let mut offset = 0;
    loop {
        let page = load(offset)?;
        let total = page["total"].as_i64().unwrap_or(0);
        let mut items = page["items"].as_array().cloned().unwrap_or_default();
        rows.append(&mut items);
        offset = rows.len() as i64;
        if offset >= total {
            break;
        }
    }
    Ok(rows)
}
pub fn export_csv(db: &AppDb, input: &CsvExportInput) -> Result<Value, AppError> {
    let (name, rows, columns): (&str, Vec<Value>, Vec<Column>) = match input.kind.as_str() {
        "SALES" => {
            let mut f = input.report_filter.clone();
            f.limit = Some(200);
            (
                "sales-report.csv",
                collect_pages(|o| {
                    f.offset = Some(o);
                    reporting::sales_report(db, &f)
                })?,
                vec![
                    ("saleNumber", "Sale", false),
                    ("customerName", "Customer", false),
                    ("status", "Status", false),
                    ("units", "Units", false),
                    ("returnedUnits", "Returned units", false),
                    ("grossMinor", "Gross", true),
                    ("refundedMinor", "Refunded", true),
                    ("netMinor", "Net", true),
                    ("postedAt", "Posted", false),
                ],
            )
        }
        "INVENTORY_STOCK" => {
            let mut f = input.report_filter.clone();
            f.limit = Some(200);
            (
                "inventory-stock.csv",
                collect_pages(|o| {
                    f.offset = Some(o);
                    reporting::inventory_stock(db, &f)
                })?,
                vec![
                    ("productName", "Product", false),
                    ("packageLabel", "Package", false),
                    ("branchName", "Branch", false),
                    ("quantity", "Stock", false),
                    ("reorderLevel", "Reorder level", false),
                    ("stockState", "State", false),
                    (
                        "acquisitionCostValuationMinor",
                        "Acquisition valuation",
                        true,
                    ),
                    ("unvaluedQuantity", "Unvalued units", false),
                ],
            )
        }
        "EXPENSES" => {
            let mut q = input.finance_query.clone();
            q.limit = Some(200);
            (
                "finance-expenses.csv",
                collect_pages(|o| {
                    q.offset = Some(o);
                    finance::expenses(db, &q)
                })?,
                vec![
                    ("expenseNumber", "Expense", false),
                    ("expenseAccountName", "Category", false),
                    ("description", "Description", false),
                    ("amountMinor", "Amount", true),
                    ("branchName", "Branch", false),
                    ("postedAt", "Posted", false),
                ],
            )
        }
        "ACCOUNT_ACTIVITY" => {
            let mut q = input.finance_query.clone();
            q.limit = Some(200);
            (
                "account-activity.csv",
                collect_pages(|o| {
                    q.offset = Some(o);
                    finance::account_activity(db, &q)
                })?,
                vec![
                    ("journalNumber", "Journal", false),
                    ("sourceType", "Source", false),
                    ("description", "Description", false),
                    ("side", "Side", false),
                    ("amountMinor", "Amount", true),
                    ("postedAt", "Posted", false),
                ],
            )
        }
        "TRIAL_BALANCE" => {
            let v = finance::trial_balance(db, &input.finance_query)?;
            (
                "trial-balance.csv",
                v["items"].as_array().cloned().unwrap_or_default(),
                vec![
                    ("accountCode", "Account code", false),
                    ("accountName", "Account", false),
                    ("accountType", "Type", false),
                    ("debitMinor", "Debit", true),
                    ("creditMinor", "Credit", true),
                    ("balanceMinor", "Normal balance", true),
                ],
            )
        }
        _ => return Err(invalid("Unsupported CSV export.")),
    };
    Ok(
        json!({"fileName":name,"mimeType":"text/csv;charset=utf-8","content":render_csv(&rows,&columns),"rowCount":rows.len()}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_quotes_and_blocks_formulas() {
        assert_eq!(csv_cell("=SUM(A1)"), "'=SUM(A1)");
        assert_eq!(csv_cell("Arabic, عربي"), "\"Arabic, عربي\"");
        assert_eq!(csv_cell("a\"b"), "\"a\"\"b\"");
    }
    #[test]
    fn money_is_exact_without_floats() {
        assert_eq!(money(125), "1.25 EGP");
        assert_eq!(money(-5), "-0.05 EGP");
        assert_eq!(money(i64::MAX), "92233720368547758.07 EGP");
    }
    #[test]
    fn empty_csv_has_header_and_crlf() {
        assert_eq!(
            render_csv(&[], &[("id", "Identifier", false)]),
            "Identifier\r\n"
        );
    }
}
