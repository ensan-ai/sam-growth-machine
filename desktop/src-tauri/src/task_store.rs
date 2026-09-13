use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    pub detail: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPreparation {
    pub task_id: String,
    pub state: String,
    pub explorer_output: Option<Value>,
    pub researcher_output: Option<Value>,
    pub orchestrator_output: Option<Value>,
    pub operator_question: Option<String>,
    pub operator_recommendation: Option<String>,
    pub operator_decision: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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
CREATE TABLE IF NOT EXISTS command_task_preparations (
  task_id TEXT PRIMARY KEY,
  state TEXT NOT NULL,
  explorer_output TEXT,
  researcher_output TEXT,
  orchestrator_output TEXT,
  operator_question TEXT,
  operator_recommendation TEXT,
  operator_decision TEXT,
  error TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
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

    pub fn preparation(&self, task_id: &str) -> Result<Option<TaskPreparation>, String> {
        self.open()?.query_row(
            "SELECT task_id,state,explorer_output,researcher_output,orchestrator_output,operator_question,operator_recommendation,operator_decision,error,created_at,updated_at FROM command_task_preparations WHERE task_id=?1",
            [task_id],
            |r| {
                Ok(TaskPreparation {
                    task_id: r.get(0)?, state: r.get(1)?,
                    explorer_output: parse_optional_json(r.get::<_, Option<String>>(2)?),
                    researcher_output: parse_optional_json(r.get::<_, Option<String>>(3)?),
                    orchestrator_output: parse_optional_json(r.get::<_, Option<String>>(4)?),
                    operator_question: r.get(5)?, operator_recommendation: r.get(6)?, operator_decision: r.get(7)?,
                    error: r.get(8)?, created_at: r.get(9)?, updated_at: r.get(10)?,
                })
            }
        ).optional().map_err(|e| e.to_string())
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
        self.record_event(&task_id, "task.created", "sam", json!({"status": status, "dependencies": request.dependency_ids}))?;
        self.get(&task_id)
    }

    pub fn begin_preparation(&self, task_id: &str, actor: &str) -> Result<CommandTask, String> {
        self.refresh_block_state(task_id)?;
        let task = self.get(task_id)?;
        if task.status == "BLOCKED" {
            self.record_event(task_id, "task.prepare_blocked", actor, json!({"reason": task.blocked_reason}))?;
            return Err(task.blocked_reason.unwrap_or_else(|| "Task is blocked".into()));
        }
        if task.status != "TODO" { return Err(format!("Only TODO tasks can be prepared; {task_id} is {}", task.status)); }
        if task.prepared_at.is_some() { return Err(format!("Task {task_id} is already prepared")); }
        let now = Utc::now().to_rfc3339();
        self.open()?.execute(
            "INSERT INTO command_task_preparations(task_id,state,created_at,updated_at) VALUES(?1,'PREPARING',?2,?2) ON CONFLICT(task_id) DO UPDATE SET state='PREPARING',error=NULL,updated_at=excluded.updated_at",
            params![task_id, now]
        ).map_err(|e| e.to_string())?;
        self.record_event(task_id, "task.prepare_started", actor, json!({}))?;
        Ok(task)
    }

    pub fn save_prepare_role_output(&self, task_id: &str, role: &str, output: &Value) -> Result<(), String> {
        let column = match role {
            "explorer" => "explorer_output",
            "researcher" => "researcher_output",
            "orchestrator" => "orchestrator_output",
            _ => return Err(format!("Unsupported preparation role {role}")),
        };
        let sql = format!("UPDATE command_task_preparations SET {column}=?2,updated_at=?3 WHERE task_id=?1");
        self.open()?.execute(&sql, params![task_id, output.to_string(), Utc::now().to_rfc3339()]).map_err(|e| e.to_string())?;
        self.record_event(task_id, &format!("prepare.{role}.completed"), role, json!({"persisted": true}))?;
        Ok(())
    }

    pub fn require_operator_input(&self, task_id: &str, question: &str, recommendation: Option<&str>) -> Result<TaskPreparation, String> {
        self.open()?.execute(
            "UPDATE command_task_preparations SET state='NEEDS_OPERATOR_INPUT',operator_question=?2,operator_recommendation=?3,error=NULL,updated_at=?4 WHERE task_id=?1",
            params![task_id, question, recommendation, Utc::now().to_rfc3339()]
        ).map_err(|e| e.to_string())?;
        self.record_event(task_id, "prepare.operator_input_required", "orchestrator", json!({"question": question, "recommendation": recommendation}))?;
        self.preparation(task_id)?.ok_or_else(|| "Preparation state disappeared".into())
    }

    pub fn set_operator_decision(&self, task_id: &str, decision: &str) -> Result<TaskPreparation, String> {
        if decision.trim().is_empty() { return Err("Operator decision cannot be empty".into()); }
        let now = Utc::now().to_rfc3339();
        let changed = self.open()?.execute(
            "UPDATE command_task_preparations SET state='PREPARING',operator_decision=?2,operator_question=NULL,operator_recommendation=NULL,error=NULL,updated_at=?3 WHERE task_id=?1",
            params![task_id, decision.trim(), now]
        ).map_err(|e| e.to_string())?;
        if changed == 0 { return Err(format!("Task {task_id} has no preparation session")); }
        self.record_event(task_id, "prepare.operator_decision", "sam", json!({"decision": decision.trim()}))?;
        self.preparation(task_id)?.ok_or_else(|| "Preparation state disappeared".into())
    }

    pub fn finalize_preparation(&self, task_id: &str, prompt_markdown: &str, orchestrator_output: &Value, actor: &str) -> Result<CommandTask, String> {
        let now = Utc::now().to_rfc3339();
        let conn = self.open()?;
        conn.execute(
            "UPDATE command_task_preparations SET state='PREPARED',orchestrator_output=?2,operator_question=NULL,operator_recommendation=NULL,error=NULL,updated_at=?3 WHERE task_id=?1",
            params![task_id, orchestrator_output.to_string(), now]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE command_tasks SET prepared_at=?2,prompt_markdown=?3,updated_at=?2 WHERE task_id=?1",
            params![task_id, now, prompt_markdown]
        ).map_err(|e| e.to_string())?;
        self.record_event(task_id, "task.prepared", actor, json!({"promptGenerated": true, "method": "explorer+researcher+orchestrator"}))?;
        self.get(task_id)
    }

    pub fn fail_preparation(&self, task_id: &str, role: &str, error: &str) -> Result<(), String> {
        self.open()?.execute(
            "UPDATE command_task_preparations SET state='FAILED',error=?2,updated_at=?3 WHERE task_id=?1",
            params![task_id, error, Utc::now().to_rfc3339()]
        ).map_err(|e| e.to_string())?;
        self.record_event(task_id, "task.prepare_failed", role, json!({"error": error}))
    }

    pub fn start(&self, task_id: &str, actor: &str) -> Result<CommandTask, String> {
        self.refresh_block_state(task_id)?;
        let task = self.get(task_id)?;
        if task.status == "BLOCKED" { return Err(task.blocked_reason.unwrap_or_else(|| "Task is blocked".into())); }
        if task.status != "TODO" { return Err(format!("Only TODO tasks can start; {task_id} is {}", task.status)); }
        if task.prepared_at.is_none() { return Err("Prepare the task before starting it".into()); }
        let prep = self.preparation(task_id)?.ok_or("Task is missing its preparation record")?;
        if prep.state != "PREPARED" { return Err(format!("Preparation is {}, not PREPARED", prep.state)); }
        let now = Utc::now().to_rfc3339();
        self.open()?.execute("UPDATE command_tasks SET status='IN_PROGRESS',started_at=?2,updated_at=?2 WHERE task_id=?1", params![task_id, now]).map_err(|e| e.to_string())?;
        self.record_event(task_id, "task.started", actor, json!({"owner": task.owner, "executionMode": task.execution_mode}))?;
        self.get(task_id)
    }

    pub fn send_to_review(&self, task_id: &str, actor: &str) -> Result<CommandTask, String> {
        let task = self.get(task_id)?;
        if task.status != "IN_PROGRESS" { return Err(format!("Only IN_PROGRESS tasks can enter review; {task_id} is {}", task.status)); }
        let now = Utc::now().to_rfc3339();
        self.open()?.execute("UPDATE command_tasks SET status='REVIEW',review_at=?2,updated_at=?2 WHERE task_id=?1", params![task_id, now]).map_err(|e| e.to_string())?;
        self.record_event(task_id, "task.review_requested", actor, json!({}))?;
        self.get(task_id)
    }

    pub fn complete(&self, task_id: &str, actor: &str) -> Result<CommandTask, String> {
        let task = self.get(task_id)?;
        if task.status != "REVIEW" { return Err(format!("Only REVIEW tasks can complete; {task_id} is {}", task.status)); }
        let now = Utc::now().to_rfc3339();
        self.open()?.execute("UPDATE command_tasks SET status='DONE',completed_at=?2,blocked_reason=NULL,updated_at=?2 WHERE task_id=?1", params![task_id, now]).map_err(|e| e.to_string())?;
        self.record_event(task_id, "task.completed", actor, json!({}))?;
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
            self.record_event(task_id, "task.auto_unblocked", "system", json!({}))?;
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
        let result = stmt.query_map([task_id], |r| r.get(0)).map_err(|e| e.to_string())?
            .map(|r| r.map_err(|e| e.to_string())).collect();
        result
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

    pub fn record_event(&self, task_id: &str, event_type: &str, actor: &str, detail: Value) -> Result<(), String> {
        self.open()?.execute(
            "INSERT INTO command_task_events(event_id,task_id,event_type,actor,detail,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![format!("task-event-{}", Uuid::new_v4()), task_id, event_type, actor, detail.to_string(), Utc::now().to_rfc3339()]
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn parse_optional_json(raw: Option<String>) -> Option<Value> {
    raw.and_then(|value| serde_json::from_str(&value).ok())
}
