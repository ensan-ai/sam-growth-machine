#![recursion_limit = "256"]

mod adapters;
mod capabilities;
mod db;
mod definitions;
mod learning;
mod models;
mod nova;
mod orchestrator;
mod policy;
mod producers;
mod providers;
mod router;
mod runner;
mod schema_fixture;

#[cfg(feature = "desktop")]
use db::Database;
#[cfg(feature = "desktop")]
use definitions::{repository_root, DefinitionStore};
#[cfg(feature = "desktop")]
use models::*;
#[cfg(feature = "desktop")]
use orchestrator::RuntimeService;

#[cfg(feature = "desktop")]
use tauri::{Manager, State};

#[cfg(feature = "desktop")]
#[tauri::command]
async fn get_dashboard(service: State<'_, RuntimeService>) -> Result<Dashboard, String> {
    service.dashboard().await
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn get_team(service: State<'_, RuntimeService>) -> Result<Vec<EmployeeSummary>, String> {
    service.db.employees()
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn get_work_items(service: State<'_, RuntimeService>) -> Result<Vec<WorkItemSummary>, String> {
    service.db.work_items()
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn get_work_item(work_item_id: String, service: State<'_, RuntimeService>) -> Result<WorkItemDetail, String> {
    service.db.detail(&work_item_id)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn get_approvals(service: State<'_, RuntimeService>) -> Result<Vec<ApprovalRecord>, String> {
    service.db.approvals(None)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn get_activity(service: State<'_, RuntimeService>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "runs": service.db.runs(None)?,
        "events": service.db.events(None)?,
        "executions": service.db.executions(None)?
    }))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn get_settings(service: State<'_, RuntimeService>) -> Result<RuntimeSettings, String> {
    service.db.settings()
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn save_settings(request: SaveSettingsRequest, service: State<'_, RuntimeService>) -> Result<RuntimeSettings, String> {
    service.db.save_settings(&request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn check_providers(service: State<'_, RuntimeService>) -> Result<ProviderStatus, String> {
    service.provider_status().await
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn start_work_item(request: StartWorkItemRequest, service: State<'_, RuntimeService>) -> Result<WorkItemSummary, String> {
    service.start(request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn run_work_item(work_item_id: String, service: State<'_, RuntimeService>) -> Result<WorkItemDetail, String> {
    service.advance_until_blocked(&work_item_id, false).await
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn decide_approval(decision: ApprovalDecision, service: State<'_, RuntimeService>) -> Result<WorkItemDetail, String> {
    service.approval(decision, false).await
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn company_control(request: ControlRequest, service: State<'_, RuntimeService>) -> Result<RuntimeSettings, String> {
    service.control(request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn nova_research_creator(
    request: nova::NovaResearchRequest,
    service: State<'_, RuntimeService>,
) -> Result<nova::NovaResearchReport, String> {
    nova::research_creator(&service.db, request).await
}

#[cfg(feature = "desktop")]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let root = repository_root().map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let _ = dotenvy::from_path(root.join(".env"));
            let definitions = DefinitionStore::load(&root).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let data_dir = app.path().app_data_dir()?;
            let database = Database::new(data_dir.join("sam-neural-core.sqlite3")).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let service = RuntimeService::new(database, definitions).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            app.manage(service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard, get_team, get_work_items, get_work_item, get_approvals,
            get_activity, get_settings, save_settings, check_providers,
            start_work_item, run_work_item, decide_approval, company_control,
            nova_research_creator
        ])
        .run(tauri::generate_context!())
        .expect("error while running SAM Neural Core");
}

#[cfg(not(feature = "desktop"))]
pub fn run() {
    panic!("SAM Neural Core desktop feature is required to launch the UI");
}
