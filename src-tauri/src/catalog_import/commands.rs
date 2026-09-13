use super::*;
use crate::commands::branches::DatabaseState;
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};
#[derive(Default)]
pub struct ImportState(pub Mutex<State>);
#[derive(Default)]
pub struct State {
    plan: Option<Plan>,
    report: Option<Report>,
    busy: bool,
}
fn root(app: &AppHandle) -> Result<PathBuf, AppError> {
    if cfg!(debug_assertions) {
        Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("project root")
            .to_path_buf())
    } else {
        app.path()
            .resource_dir()
            .map_err(|_| invalid("Cannot resolve canonical resources."))
    }
}
fn output(app: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(app
        .path()
        .app_local_data_dir()
        .map_err(|_| invalid("Cannot resolve report directory."))?
        .join("import-reports"))
}
fn summary(mut r: Report) -> Report {
    r.rows = r
        .rows
        .into_iter()
        .filter(|r| {
            !matches!(
                r.disposition.as_str(),
                "ready" | "imported" | "already_imported"
            )
        })
        .take(30)
        .collect();
    r
}
#[tauri::command]
pub async fn profile_catalog_source(app: AppHandle) -> Result<Profile, AppError> {
    tauri::async_runtime::spawn_blocking(move || source::read(&root(&app)?).map(|r| r.0))
        .await
        .map_err(|_| invalid("Profile task failed."))?
}
#[tauri::command]
pub async fn dry_run_catalog_import(app: AppHandle) -> Result<Report, AppError> {
    {
        let state = app.state::<ImportState>();
        let mut s = state
            .0
            .lock()
            .map_err(|_| invalid("Import state unavailable."))?;
        if s.busy {
            return Err(invalid("An import operation is already running."));
        }
        s.busy = true;
        s.plan = None;
    }
    tauri::async_runtime::spawn_blocking(move || {
        let result = (|| {
            let db = app.state::<DatabaseState>();
            planner::build(db.db()?, &root(&app)?, &output(&app)?)
        })();
        let state = app.state::<ImportState>();
        let mut s = state
            .0
            .lock()
            .map_err(|_| invalid("Import state unavailable."))?;
        s.busy = false;
        match result {
            Ok(plan) => {
                let report = summary(plan.report.clone());
                s.report = Some(report.clone());
                s.plan = Some(plan);
                Ok(report)
            }
            Err(e) => Err(e),
        }
    })
    .await
    .map_err(|_| invalid("Dry-run task failed."))?
}
#[tauri::command]
pub fn apply_catalog_import(
    app: AppHandle,
    plan_id: String,
    confirmed: bool,
) -> Result<Report, AppError> {
    if !confirmed {
        return Err(invalid("Explicit confirmation is required."));
    }
    let plan = {
        let state = app.state::<ImportState>();
        let mut s = state
            .0
            .lock()
            .map_err(|_| invalid("Import state unavailable."))?;
        if s.busy {
            return Err(invalid("An import is already running."));
        }
        if s.plan.as_ref().is_none_or(|p| p.report.id != plan_id) {
            return Err(invalid("Run a fresh dry run before applying."));
        }
        let plan = s.plan.take().expect("validated plan");
        s.busy = true;
        let mut r = summary(plan.report.clone());
        r.status = "applying".into();
        r.mode = "apply".into();
        s.report = Some(r);
        plan
    };
    let initial = summary(plan.report.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let result = (|| {
            let db = app.state::<DatabaseState>();
            importer::apply(db.db()?, &root(&app)?, &output(&app)?, plan, |r| {
                if let Ok(mut s) = app.state::<ImportState>().0.lock() {
                    s.report = Some(summary(r.clone()));
                }
            })
        })();
        if let Ok(mut s) = app.state::<ImportState>().0.lock() {
            s.busy = false;
            match result {
                Ok(r) => s.report = Some(summary(r)),
                Err(e) => {
                    if let Some(r) = &mut s.report {
                        r.status = "failed".into();
                        r.error = Some(e.message);
                    }
                }
            }
        }
    });
    Ok(Report {
        status: "applying".into(),
        mode: "apply".into(),
        ..initial
    })
}
#[tauri::command]
pub fn get_catalog_import_status(app: AppHandle) -> Result<Option<Report>, AppError> {
    let state = app.state::<ImportState>();
    let s = state
        .0
        .lock()
        .map_err(|_| invalid("Import state unavailable."))?;
    Ok(s.report.clone())
}
#[tauri::command]
pub async fn get_catalog_import_report(app: AppHandle) -> Result<Option<Report>, AppError> {
    if let Some(r) = get_catalog_import_status(app.clone())? {
        return Ok(Some(r));
    }
    tauri::async_runtime::spawn_blocking(move||{
 let state=app.state::<DatabaseState>();let c=state.db()?.lock()?;
 let mut q=c.prepare("SELECT report_path,status,imported_rows FROM catalog_import_runs ORDER BY started_at DESC LIMIT 1")?;
 let mut rows=q.query([])?;
 if let Some(row)=rows.next()?{let path:String=row.get(0)?;let status:String=row.get(1)?;
 if status=="applying" || !std::path::Path::new(&path).exists(){return Ok(Some(Report{status:"interrupted".into(),imported:row.get::<_,i64>(2)? as usize,error:Some("Previous run did not finish its report. Committed rows are retained; run a fresh dry run to resume safely.".into()),..Default::default()}));}
 let r:Report=serde_json::from_slice(&std::fs::read(path)?).map_err(|_|invalid("Cannot read stored import report."))?;Ok(Some(summary(r)))}else{Ok(None)}
 }).await.map_err(|_|invalid("Report task failed."))?
}
