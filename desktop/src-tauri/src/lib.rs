#![recursion_limit = "256"]

mod adapters;
mod capabilities;
mod db;
mod definitions;
mod learning;
mod models;
mod orchestrator;
mod policy;
mod producers;
mod providers;
mod router;
mod runner;
mod schema_fixture;
mod task_prepare;
mod task_store;

#[cfg(feature = "desktop")]
use db::Database;
#[cfg(feature = "desktop")]
use definitions::{repository_root, DefinitionStore};
#[cfg(feature = "desktop")]
use models::*;
#[cfg(feature = "desktop")]
use orchestrator::RuntimeService;
#[cfg(feature = "desktop")]
use task_prepare::{PrepareCommandTaskResult, TaskPrepareEngine};
#[cfg(feature = "desktop")]
use task_store::{CommandTask, CreateCommandTaskRequest, TaskEvent, TaskPreparation, TaskStore};

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
fn get_command_tasks(store: State<'_, TaskStore>) -> Result<Vec<CommandTask>, String> {
    store.list()
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn get_command_task_events(task_id: Option<String>, store: State<'_, TaskStore>) -> Result<Vec<TaskEvent>, String> {
    store.events(task_id.as_deref())
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn get_command_task_preparation(task_id: String, store: State<'_, TaskStore>) -> Result<Option<TaskPreparation>, String> {
    store.preparation(&task_id)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn create_command_task(request: CreateCommandTaskRequest, store: State<'_, TaskStore>) -> Result<CommandTask, String> {
    store.create(&request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
async fn prepare_command_task(
    task_id: String,
    store: State<'_, TaskStore>,
    service: State<'_, RuntimeService>,
) -> Result<PrepareCommandTaskResult, String> {
    let root = repository_root()?;
    let engine = TaskPrepareEngine::new(store.inner().clone(), service.db.clone(), root);
    engine.prepare(&task_id, "sam").await
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn answer_command_task_preparation(
    task_id: String,
    decision: String,
    store: State<'_, TaskStore>,
    service: State<'_, RuntimeService>,
) -> Result<TaskPreparation, String> {
    let root = repository_root()?;
    let engine = TaskPrepareEngine::new(store.inner().clone(), service.db.clone(), root);
    engine.answer_operator(&task_id, &decision)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn start_command_task(task_id: String, store: State<'_, TaskStore>) -> Result<CommandTask, String> {
    store.start(&task_id, "sam")
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn review_command_task(task_id: String, store: State<'_, TaskStore>) -> Result<CommandTask, String> {
    store.send_to_review(&task_id, "sam")
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn complete_command_task(task_id: String, store: State<'_, TaskStore>) -> Result<CommandTask, String> {
    store.complete(&task_id, "sam")
}

#[cfg(feature = "desktop")]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let root = repository_root().map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let definitions = DefinitionStore::load(&root).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let data_dir = app.path().app_data_dir()?;
            let database_path = data_dir.join("sam-neural-core.sqlite3");
            let database = Database::new(&database_path).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let task_store = TaskStore::new(&database_path).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let service = RuntimeService::new(database, definitions).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            app.manage(service);
            app.manage(task_store);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard, get_team, get_work_items, get_work_item, get_approvals,
            get_activity, get_settings, save_settings, check_providers,
            start_work_item, run_work_item, decide_approval, company_control,
            get_command_tasks, get_command_task_events, get_command_task_preparation,
            create_command_task, prepare_command_task, answer_command_task_preparation,
            start_command_task, review_command_task, complete_command_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running SAM Neural Core");
}

#[cfg(not(feature = "desktop"))]
pub fn run() {
    panic!("SAM Neural Core desktop feature is required to launch the UI");
}
