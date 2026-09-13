use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TaskStore {
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandTask {
    pub task_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub execution_mode: String,
    pub owner: String,
    pub milestone: Option<String>,
    pub priority: i64,
    pub dependency_ids: Vec<String>,
    pub blocked_reason: Option<String>,
    pub prepared_at: Option<String>,
    pub started_at: Option<String>,
    pub review_at: Option<String>,
    pub completed_at: Option<String>,
    pub prompt_markdown: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommandTaskRequest {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_owner")]
    pub owner: String,
    #[serde(default = "default_execution_mode")]
    pub execution_mode: String,
    pub milestone: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: i64,
    #[serde(default)]
    pub dependency_ids: Vec<String>,
}

fn default_owner() -> String { "travis".into() }
fn default_execution_mode() -> String { "AGENT".into() }
fn default_priority() -> i64 { 3 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEvent {
    pub event_id: String,
    pub task_id: String,
    pub event_type: String,
    pub actor: String,
    pub detail: serde_json::Value,
    pub created_at: String,
}

impl TaskStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let store = Self { path };
        store.init()?;
        Ok(store)
    }

    fn open(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
            .map_err(|e| e.to_string())?;
        Ok(conn)
    }

    fn init(&self) -> Result<(), String> {
        self.open()?.execute_batch(r#"
CREATE TABLE IF NOT EXISTS command_tasks (
  task_id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL,
  execution_mode TEXT NOT NULL,
  owner TEXT NOT NULL,
  milestone TEXT,
  priority INTEGER NOT NULL DEFAULT 3,
  blocked_reason TEXT,
  prepared_at TEXT,
  started_at TEXT,
  review_at TEXT,
  completed_at TEXT,
  prompt_markdown TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS command_task_dependencies (
  task_id TEXT NOT NULL,
  depends_on_task_id TEXT NOT NULL,
  PRIMARY KEY(task_id, depends_on_task_id),
  FOREIGN KEY(task_id) REFERENCES command_tasks(task_id) ON DELETE CASCADE,
  FOREIGN KEY(depends_on_task_id) REFERENCES command_tasks(task_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS command_task_events (
  event_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  actor TEXT NOT NULL,
  detail TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY(task_id) REFERENCES command_tasks(task_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_command_tasks_status ON command_tasks(status);
CREATE INDEX IF NOT EXISTS idx_command_task_events_task ON command_task_events(task_id, created_at);
"#).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<CommandTask>, String> {
        let conn = self.open()?;
        let mut stmt = conn.prepare(
            "SELECT task_id,title,description,status,execution_mode,owner,milestone,priority,blocked_reason,prepared_at,started_at,review_at,completed_at,prompt_markdown,created_at,updated_at FROM command_tasks ORDER BY priority ASC, created_at ASC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| Ok((
            r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?,
            r.get::<_, String>(3)?, r.get::<_, String>(4)?, r.get::<_, String>(5)?,
            r.get::<_, Option<String>>(6)?, r.get::<_, i64>(7)?, r.get::<_, Option<String>>(8)?,
            r.get::<_, Option<String>>(9)?, r.get::<_, Option<String>>(10)?, r.get::<_, Option<String>>(11)?,
            r.get::<_, Option<String>>(12)?, r.get::<_, Option<String>>(13)?, r.get::<_, String>(14)?,
            r.get::<_, String>(15)?
        ))).map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            let (task_id,title,description,status,execution_mode,owner,milestone,priority,blocked_reason,prepared_at,started_at,review_at,completed_at,prompt_markdown,created_at,updated_at) = row.map_err(|e| e.to_string())?;
            let dependency_ids = self.dependencies_with_conn(&conn, &task_id)?;
            result.push(CommandTask { task_id,title,description,status,execution_mode,owner,milestone,priority,dependency_ids,blocked_reason,prepared_at,started_at,review_at,completed_at,prompt_markdown,created_at,updated_at });
        }
        Ok(result)
    }

    pub fn get(&self, task_id: &str) -> Result<CommandTask, String> {
        self.list()?.into_iter().find(|t| t.task_id == task_id)
            .ok_or_else(|| format!("Task {task_id} was not found"))
    }

    pub fn events(&self, task_id: Option<&str>) -> Result<Vec<TaskEvent>, String> {
        let conn = self.open()?;
        let sql = if task_id.is_some() {
            "SELECT event_id,task_id,event_type,actor,detail,created_at FROM command_task_events WHERE task_id=?1 ORDER BY created_at DESC"
        } else {
            "SELECT event_id,task_id,event_type,actor,detail,created_at FROM command_task_events ORDER BY created_at DESC LIMIT 250"
        };
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let mapper = |r: &rusqlite::Row<'_>| -> rusqlite::Result<TaskEvent> {
            let raw: String = r.get(4)?;
            Ok(TaskEvent {
                event_id: r.get(0)?, task_id: r.get(1)?, event_type: r.get(2)?, actor: r.get(3)?,
                detail: serde_json::from_str(&raw).unwrap_or_else(|_| json!({"raw": raw})),
                created_at: r.get(5)?,
            })
        };
        let rows = if let Some(id) = task_id {
            stmt.query_map([id], mapper).map_err(|e| e.to_string())?
        } else {
            stmt.query_map([], mapper).map_err(|e| e.to_string())?
        };
        rows.map(|row| row.map_err(|e| e.to_string())).collect()
    }

    pub fn create(&self, request: &CreateCommandTaskRequest) -> Result<CommandTask, String> {
        if request.title.trim().is_empty() { return Err("Task title is required".into()); }
        if !matches!(request.execution_mode.as_str(), "AGENT" | "HUMAN" | "PAIR") {
            return Err("executionMode must be AGENT, HUMAN, or PAIR".into());
        }
        if !(1..=5).contains(&request.priority) { return Err("priority must be between 1 and 5".into()); }

        let mut conn = self.open()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for dep in &request.dependency_ids {
            let exists: Option<String> = tx.query_row("SELECT task_id FROM command_tasks WHERE task_id=?1", [dep], |r| r.get(0)).optional().map_err(|e| e.to_string())?;
            if exists.is_none() { return Err(format!("Dependency {dep} does not exist")); }
        }
        let next: i64 = tx.query_row(
            "SELECT COALESCE(MAX(CAST(SUBSTR(task_id,4) AS INTEGER)),0)+1 FROM command_tasks WHERE task_id LIKE 'SG-%'",
            [], |r| r.get(0)
        ).map_err(|e| e.to_string())?;
        let task_id = format!("SG-{next:03}");
        let now = Utc::now().to_rfc3339();
        let unresolved = Self::unresolved_dependencies_tx(&tx, &request.dependency_ids)?;
        let (status, blocked_reason) = if unresolved.is_empty() {
            ("TODO", None)
        } else {
            ("BLOCKED", Some(format!("Waiting for {}", unresolved.join(", "))))
        };
        tx.execute(
            "INSERT INTO command_tasks(task_id,title,description,status,execution_mode,owner,milestone,priority,blocked_reason,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
            params![task_id, request.title.trim(), request.description.trim(), status, request.execution_mode, request.owner, request.milestone, request.priority, blocked_reason, now]
        ).map_err(|e| e.to_string())?;
        for dep in &request.dependency_ids {
            tx.execute("INSERT INTO command_task_dependencies(task_id,depends_on_task_id) VALUES(?1,?2)", params![task_id, dep]).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        self.event(&task_id, "task.created", "sam", json!({"status": status, "dependencies": request.dependency_ids}))?;
        self.get(&task_id)
    }

    pub fn prepare(&self, task_id: &str, actor: &str) -> Result<CommandTask, String> {
        self.refresh_block_state(task_id)?;
        let task = self.get(task_id)?;
        if task.status == "BLOCKED" {
            self.event(task_id, "task.prepare_blocked", actor, json!({"reason": task.blocked_reason}))?;
            return Ok(task);
        }
        if task.status != "TODO" { return Err(format!("Only TODO tasks can be prepared; {task_id} is {}", task.status)); }
        let prompt = format!(
            "# TASK {id}\n\n## Objective\n{title}\n\n## Description\n{description}\n\n## Execution mode\n{mode}\n\n## Owner\n{owner}\n\n## Milestone\n{milestone}\n\n## Dependencies\n{deps}\n\n## Operating rule\nStudy the task context before execution. Do not invent missing requirements. Surface operator decisions when they materially change the outcome. Preserve an auditable event trail and return a reviewable artifact before completion.\n",
            id=task.task_id, title=task.title, description=if task.description.is_empty(){"No additional description supplied."}else{&task.description}, mode=task.execution_mode,
            owner=task.owner, milestone=task.milestone.clone().unwrap_or_else(|| "Unassigned".into()), deps=if task.dependency_ids.is_empty(){"None".into()}else{task.dependency_ids.join(", ")}
        );
        let now = Utc::now().to_rfc3339();
        self.open()?.execute("UPDATE command_tasks SET prepared_at=?2,prompt_markdown=?3,updated_at=?2 WHERE task_id=?1", params![task_id, now, prompt]).map_err(|e| e.to_string())?;
        self.event(task_id, "task.prepared", actor, json!({"promptGenerated": true}))?;
        self.get(task_id)
    }

    pub fn start(&self, task_id: &str, actor: &str) -> Result<CommandTask, String> {
        self.refresh_block_state(task_id)?;
        let task = self.get(task_id)?;
        if task.status == "BLOCKED" { return Err(task.blocked_reason.unwrap_or_else(|| "Task is blocked".into())); }
        if task.status != "TODO" { return Err(format!("Only TODO tasks can start; {task_id} is {}", task.status)); }
        if task.prepared_at.is_none() { return Err("Prepare the task before starting it".into()); }
        let now = Utc::now().to_rfc3339();
        self.open()?.execute("UPDATE command_tasks SET status='IN_PROGRESS',started_at=?2,updated_at=?2 WHERE task_id=?1", params![task_id, now]).map_err(|e| e.to_string())?;
        self.event(task_id, "task.started", actor, json!({"owner": task.owner, "executionMode": task.execution_mode}))?;
        self.get(task_id)
    }

    pub fn send_to_review(&self, task_id: &str, actor: &str) -> Result<CommandTask, String> {
        let task = self.get(task_id)?;
        if task.status != "IN_PROGRESS" { return Err(format!("Only IN_PROGRESS tasks can enter review; {task_id} is {}", task.status)); }
        let now = Utc::now().to_rfc3339();
        self.open()?.execute("UPDATE command_tasks SET status='REVIEW',review_at=?2,updated_at=?2 WHERE task_id=?1", params![task_id, now]).map_err(|e| e.to_string())?;
        self.event(task_id, "task.review_requested", actor, json!({}))?;
        self.get(task_id)
    }

    pub fn complete(&self, task_id: &str, actor: &str) -> Result<CommandTask, String> {
        let task = self.get(task_id)?;
        if task.status != "REVIEW" { return Err(format!("Only REVIEW tasks can complete; {task_id} is {}", task.status)); }
        let now = Utc::now().to_rfc3339();
        self.open()?.execute("UPDATE command_tasks SET status='DONE',completed_at=?2,blocked_reason=NULL,updated_at=?2 WHERE task_id=?1", params![task_id, now]).map_err(|e| e.to_string())?;
        self.event(task_id, "task.completed", actor, json!({}))?;
        self.auto_unblock_dependents(task_id)?;
        self.get(task_id)
    }

    fn refresh_block_state(&self, task_id: &str) -> Result<(), String> {
        let task = self.get(task_id)?;
        if !matches!(task.status.as_str(), "BLOCKED" | "TODO") { return Ok(()); }
        let unresolved = self.unresolved_dependencies(&task.dependency_ids)?;
        let conn = self.open()?;
        if unresolved.is_empty() && task.status == "BLOCKED" {
            conn.execute("UPDATE command_tasks SET status='TODO',blocked_reason=NULL,updated_at=?2 WHERE task_id=?1", params![task_id, Utc::now().to_rfc3339()]).map_err(|e| e.to_string())?;
            self.event(task_id, "task.auto_unblocked", "system", json!({}))?;
        } else if !unresolved.is_empty() {
            conn.execute("UPDATE command_tasks SET status='BLOCKED',blocked_reason=?2,updated_at=?3 WHERE task_id=?1", params![task_id, format!("Waiting for {}", unresolved.join(", ")), Utc::now().to_rfc3339()]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn auto_unblock_dependents(&self, completed_task_id: &str) -> Result<(), String> {
        let conn = self.open()?;
        let mut stmt = conn.prepare("SELECT task_id FROM command_task_dependencies WHERE depends_on_task_id=?1").map_err(|e| e.to_string())?;
        let ids: Vec<String> = stmt.query_map([completed_task_id], |r| r.get(0)).map_err(|e| e.to_string())?
            .map(|r| r.map_err(|e| e.to_string())).collect::<Result<_,_>>()?;
        drop(stmt); drop(conn);
        for id in ids { self.refresh_block_state(&id)?; }
        Ok(())
    }

    fn dependencies_with_conn(&self, conn: &Connection, task_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = conn.prepare("SELECT depends_on_task_id FROM command_task_dependencies WHERE task_id=?1 ORDER BY depends_on_task_id").map_err(|e| e.to_string())?;
        stmt.query_map([task_id], |r| r.get(0)).map_err(|e| e.to_string())?
            .map(|r| r.map_err(|e| e.to_string())).collect()
    }

    fn unresolved_dependencies(&self, deps: &[String]) -> Result<Vec<String>, String> {
        let conn = self.open()?;
        Self::unresolved_dependencies_conn(&conn, deps)
    }

    fn unresolved_dependencies_conn(conn: &Connection, deps: &[String]) -> Result<Vec<String>, String> {
        let mut unresolved = Vec::new();
        for dep in deps {
            let state: Option<String> = conn.query_row("SELECT status FROM command_tasks WHERE task_id=?1", [dep], |r| r.get(0)).optional().map_err(|e| e.to_string())?;
            if state.as_deref() != Some("DONE") { unresolved.push(dep.clone()); }
        }
        Ok(unresolved)
    }

    fn unresolved_dependencies_tx(tx: &rusqlite::Transaction<'_>, deps: &[String]) -> Result<Vec<String>, String> {
        let mut unresolved = Vec::new();
        for dep in deps {
            let state: Option<String> = tx.query_row("SELECT status FROM command_tasks WHERE task_id=?1", [dep], |r| r.get(0)).optional().map_err(|e| e.to_string())?;
            if state.as_deref() != Some("DONE") { unresolved.push(dep.clone()); }
        }
        Ok(unresolved)
    }

    fn event(&self, task_id: &str, event_type: &str, actor: &str, detail: serde_json::Value) -> Result<(), String> {
        self.open()?.execute(
            "INSERT INTO command_task_events(event_id,task_id,event_type,actor,detail,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![format!("task-event-{}", Uuid::new_v4()), task_id, event_type, actor, detail.to_string(), Utc::now().to_rfc3339()]
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}
