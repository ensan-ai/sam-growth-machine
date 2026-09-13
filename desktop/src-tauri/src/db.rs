use crate::models::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Database { path: PathBuf }

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        let db = Self { path };
        db.init()?;
        Ok(db)
    }

    fn open(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
            .map_err(|e| e.to_string())?;
        Ok(conn)
    }

    fn init(&self) -> Result<(), String> {
        let conn = self.open()?;
        conn.execute_batch(r#"
CREATE TABLE IF NOT EXISTS employees (
  employee_id TEXT PRIMARY KEY, name TEXT NOT NULL, title TEXT NOT NULL,
  reports_to TEXT NOT NULL, version TEXT NOT NULL, definition_status TEXT NOT NULL,
  runtime_status TEXT NOT NULL DEFAULT 'IDLE'
);
CREATE TABLE IF NOT EXISTS work_items (
  work_item_id TEXT PRIMARY KEY, title TEXT NOT NULL, state TEXT NOT NULL,
  current_owner TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  blocked_reason TEXT, resume_state TEXT
);
CREATE TABLE IF NOT EXISTS artifacts (
  artifact_id TEXT PRIMARY KEY, artifact_type TEXT NOT NULL, work_item_id TEXT NOT NULL,
  version INTEGER NOT NULL, producer TEXT NOT NULL, created_at TEXT NOT NULL,
  supersedes TEXT, payload TEXT NOT NULL,
  UNIQUE(work_item_id, artifact_type, version),
  FOREIGN KEY(work_item_id) REFERENCES work_items(work_item_id)
);
CREATE TABLE IF NOT EXISTS handoffs (
  handoff_id TEXT PRIMARY KEY, logical_key TEXT NOT NULL UNIQUE, work_item_id TEXT NOT NULL,
  sender TEXT NOT NULL, receiver TEXT NOT NULL, artifact_id TEXT NOT NULL,
  status TEXT NOT NULL, created_at TEXT NOT NULL, acknowledged_at TEXT,
  FOREIGN KEY(work_item_id) REFERENCES work_items(work_item_id),
  FOREIGN KEY(artifact_id) REFERENCES artifacts(artifact_id)
);
CREATE TABLE IF NOT EXISTS approvals (
  approval_id TEXT PRIMARY KEY, work_item_id TEXT NOT NULL, approver TEXT NOT NULL,
  content_artifact_id TEXT NOT NULL, content_version INTEGER NOT NULL,
  creative_artifact_id TEXT NOT NULL, creative_version INTEGER NOT NULL,
  package_artifact_id TEXT NOT NULL, package_version INTEGER NOT NULL,
  platform_scope TEXT NOT NULL, status TEXT NOT NULL, feedback TEXT,
  created_at TEXT NOT NULL, superseded_at TEXT,
  FOREIGN KEY(work_item_id) REFERENCES work_items(work_item_id)
);
CREATE TABLE IF NOT EXISTS agent_runs (
  run_id TEXT PRIMARY KEY, logical_key TEXT NOT NULL UNIQUE, work_item_id TEXT NOT NULL,
  agent TEXT NOT NULL, task_type TEXT NOT NULL, provider TEXT NOT NULL, model TEXT NOT NULL,
  started_at TEXT NOT NULL, finished_at TEXT, success INTEGER NOT NULL DEFAULT 0,
  escalation_occurred INTEGER NOT NULL DEFAULT 0, token_usage INTEGER,
  estimated_api_cost REAL, error TEXT,
  FOREIGN KEY(work_item_id) REFERENCES work_items(work_item_id)
);
CREATE TABLE IF NOT EXISTS system_events (
  event_id TEXT PRIMARY KEY, work_item_id TEXT, event_type TEXT NOT NULL,
  actor TEXT NOT NULL, detail TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS publication_records (
  publication_id TEXT PRIMARY KEY, work_item_id TEXT NOT NULL, receipt_artifact_id TEXT NOT NULL,
  platform TEXT NOT NULL, external_reference TEXT NOT NULL, status TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS performance_snapshots (
  snapshot_id TEXT PRIMARY KEY, work_item_id TEXT NOT NULL, publication_id TEXT NOT NULL,
  window_name TEXT NOT NULL, payload TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS learning_records (
  learning_id TEXT PRIMARY KEY, source_work_item_id TEXT NOT NULL,
  opportunity_fingerprint TEXT NOT NULL, cycle_decision TEXT NOT NULL,
  data_quality TEXT NOT NULL, recommended_attention TEXT, payload TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS executions (
  execution_id TEXT PRIMARY KEY, logical_key TEXT NOT NULL UNIQUE,
  work_item_id TEXT, role TEXT NOT NULL, capability TEXT NOT NULL, kind TEXT NOT NULL,
  provider TEXT, model TEXT, started_at TEXT NOT NULL, finished_at TEXT,
  success INTEGER NOT NULL DEFAULT 0, error TEXT
);
"#).map_err(|e| e.to_string())?;
        let _ = conn.execute("ALTER TABLE work_items ADD COLUMN resume_state TEXT", []);
        for (key, value) in [
            ("ollama_endpoint", "http://localhost:11434"), ("ollama_model", "qwen3:14b"),
            ("openai_model", "gpt-5.6"), ("allow_openai_escalation", "false"),
            ("company_control", "RUNNING"), ("observe_quality", "UNAVAILABLE"),
            ("brain_force_fail", "false"), ("publish_force_fail", "false")
        ] {
            conn.execute("INSERT OR IGNORE INTO settings(key,value) VALUES(?1,?2)", params![key, value]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn sync_employees(&self, employees: &[EmployeeSummary]) -> Result<(), String> {
        let mut conn = self.open()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for e in employees {
            tx.execute(r#"INSERT INTO employees(employee_id,name,title,reports_to,version,definition_status,runtime_status)
VALUES(?1,?2,?3,?4,?5,?6,'IDLE') ON CONFLICT(employee_id) DO UPDATE SET
name=excluded.name,title=excluded.title,reports_to=excluded.reports_to,
version=excluded.version,definition_status=excluded.definition_status"#,
                params![e.employee_id,e.name,e.title,e.reports_to,e.version,e.definition_status]).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

    pub fn employees(&self) -> Result<Vec<EmployeeSummary>, String> {
        let conn = self.open()?;
        let mut stmt = conn.prepare("SELECT employee_id,name,title,reports_to,version,definition_status,runtime_status FROM employees ORDER BY CASE employee_id WHEN 'travis' THEN 0 WHEN 'saly' THEN 1 WHEN 'adam' THEN 2 WHEN 'brain' THEN 3 WHEN 'jax' THEN 4 WHEN 'maro' THEN 5 ELSE 6 END").map_err(|e| e.to_string())?;
        let mapped = stmt.query_map([], |r| Ok(EmployeeSummary { employee_id:r.get(0)?,name:r.get(1)?,title:r.get(2)?,reports_to:r.get(3)?,version:r.get(4)?,definition_status:r.get(5)?,runtime_status:r.get(6)? })).map_err(|e| e.to_string())?;
        let result = rows(mapped);
        result
    }

    pub fn set_employee_status(&self, id: &str, status: &str) -> Result<(), String> {
        self.open()?.execute("UPDATE employees SET runtime_status=?2 WHERE employee_id=?1", params![id,status]).map_err(|e| e.to_string())?; Ok(())
    }

    pub fn settings(&self) -> Result<RuntimeSettings, String> {
        let conn = self.open()?;
        let get = |key: &str| -> Result<String,String> { conn.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get(0)).map_err(|e| e.to_string()) };
        Ok(RuntimeSettings { ollama_endpoint:get("ollama_endpoint")?, ollama_model:get("ollama_model")?, openai_model:get("openai_model")?, allow_openai_escalation:get("allow_openai_escalation")? == "true", company_control:get("company_control")?, observe_quality: get("observe_quality").unwrap_or_else(|_| "UNAVAILABLE".into()), brain_force_fail: get("brain_force_fail").unwrap_or_else(|_| "false".into()) == "true", publish_force_fail: get("publish_force_fail").unwrap_or_else(|_| "false".into()) == "true" })
    }

    pub fn save_settings(&self, request: &SaveSettingsRequest) -> Result<RuntimeSettings, String> {
        let conn = self.open()?;
        for (key,value) in [("ollama_endpoint",request.ollama_endpoint.clone()),("ollama_model",request.ollama_model.clone()),("openai_model",request.openai_model.clone()),("allow_openai_escalation",request.allow_openai_escalation.to_string())] {
            conn.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,value]).map_err(|e| e.to_string())?;
        }
        self.settings()
    }

    pub fn set_control(&self, control: &str) -> Result<(), String> {
        self.open()?.execute("UPDATE settings SET value=?1 WHERE key='company_control'",[control]).map_err(|e| e.to_string())?; Ok(())
    }

    pub fn create_work_item(&self, title: &str, signal: &str) -> Result<WorkItemSummary, String> {
        let id = format!("work-{}", Uuid::new_v4()); let now = Utc::now().to_rfc3339();
        let conn = self.open()?;
        conn.execute("INSERT INTO work_items(work_item_id,title,state,current_owner,created_at,updated_at,blocked_reason,resume_state) VALUES(?1,?2,'NEW','saly',?3,?3,NULL,NULL)",params![id,title,now]).map_err(|e| e.to_string())?;
        self.insert_artifact(&id,"RESEARCH_SIGNAL",1,"sam",None,json!({"signal":signal,"source":"HUMAN_REQUEST","created_at":now}))?;
        self.event(Some(&id),"work_item.created","sam",json!({"title":title}))?;
        self.work_item(&id)
    }

    pub fn update_work_item(&self, id: &str, state: &str, owner: &str, blocked_reason: Option<&str>) -> Result<(), String> {
        self.update_work_item_with_resume(id, state, owner, blocked_reason, None)
    }

    pub fn is_valid_resume_state(state: &str) -> bool {
        matches!(
            state,
            "RESEARCHED" | "SELECTED" | "STRATEGIZED" | "IN_PRODUCTION" | "REVISION_REQUIRED"
                | "READY_FOR_APPROVAL" | "APPROVED" | "READY_TO_PUBLISH" | "PUBLISHED" | "MEASURING"
        )
    }

    pub fn enter_blocked(&self, id: &str, owner: &str, reason: &str, resume_state: &str) -> Result<String, String> {
        let current = self.work_item(id)?;
        let resume = if current.state == "BLOCKED" {
            match current.resume_state.as_deref() {
                Some(existing) if Self::is_valid_resume_state(existing) => existing.to_string(),
                _ => {
                    if Self::is_valid_resume_state(resume_state) {
                        resume_state.to_string()
                    } else {
                        return Err("BLOCKED requires a valid non-NEW, non-BLOCKED resume_state".into());
                    }
                }
            }
        } else if Self::is_valid_resume_state(resume_state) {
            resume_state.to_string()
        } else {
            return Err(format!("Refusing to persist invalid resume_state '{resume_state}'"));
        };
        let owner = if current.state == "BLOCKED" { current.current_owner.clone() } else { owner.to_string() };
        let now = Utc::now().to_rfc3339();
        self.open()?.execute(
            "UPDATE work_items SET state='BLOCKED', current_owner=?2, blocked_reason=?3, resume_state=?4, updated_at=?5 WHERE work_item_id=?1",
            params![id, owner, reason, resume.as_str(), now],
        ).map_err(|e| e.to_string())?;
        Ok(resume)
    }

    pub fn update_work_item_with_resume(&self, id: &str, state: &str, owner: &str, blocked_reason: Option<&str>, resume_state: Option<&str>) -> Result<(), String> {
        if state == "BLOCKED" {
            let resume = resume_state.ok_or_else(|| "BLOCKED requires a valid resume_state".to_string())?;
            self.enter_blocked(id, owner, blocked_reason.unwrap_or("blocked"), resume)?;
            return Ok(());
        }
        self.open()?.execute(
            "UPDATE work_items SET state=?2, current_owner=?3, blocked_reason=?4, resume_state=NULL, updated_at=?5 WHERE work_item_id=?1",
            params![id, state, owner, blocked_reason, Utc::now().to_rfc3339()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_blocked_reason(&self, id: &str, reason: &str) -> Result<(), String> {
        let changed = self.open()?.execute(
            "UPDATE work_items SET blocked_reason=?2, updated_at=?3 WHERE work_item_id=?1 AND state='BLOCKED'",
            params![id, reason, Utc::now().to_rfc3339()],
        ).map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err("blocked_reason can only be updated while the work item is BLOCKED".into());
        }
        Ok(())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        self.open()?.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key, value]).map_err(|e| e.to_string())?; Ok(())
    }

    pub fn work_item(&self, id: &str) -> Result<WorkItemSummary, String> {
        self.open()?.query_row("SELECT work_item_id,title,state,current_owner,created_at,updated_at,blocked_reason,resume_state FROM work_items WHERE work_item_id=?1",[id], work_row).map_err(|e| e.to_string())
    }

    pub fn work_items(&self) -> Result<Vec<WorkItemSummary>, String> {
        let conn=self.open()?; let mut stmt=conn.prepare("SELECT work_item_id,title,state,current_owner,created_at,updated_at,blocked_reason,resume_state FROM work_items ORDER BY updated_at DESC").map_err(|e|e.to_string())?;
        let mapped=stmt.query_map([],work_row).map_err(|e|e.to_string())?;
        let result=rows(mapped);
        result
    }

    pub fn insert_artifact(&self, work_item_id:&str, artifact_type:&str, version:i64, producer:&str, supersedes:Option<&str>, payload:Value) -> Result<ArtifactRecord,String> {
        let id=format!("artifact-{}",Uuid::new_v4()); let now=Utc::now().to_rfc3339(); let payload_text=payload.to_string();
        let conn=self.open()?;
        conn.execute("INSERT INTO artifacts VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![id,artifact_type,work_item_id,version,producer,now,supersedes,payload_text]).map_err(|e|e.to_string())?;
        if version>1 && matches!(artifact_type,"CONTENT_DRAFT"|"CREATIVE_PACKAGE"|"NO_VISUAL_REQUIRED"|"PUBLISH_PACKAGE"|"CONTENT_BRIEF") {
            conn.execute("UPDATE approvals SET superseded_at=?2 WHERE work_item_id=?1 AND superseded_at IS NULL AND status='APPROVED'",params![work_item_id,now]).map_err(|e|e.to_string())?;
        }
        Ok(ArtifactRecord{artifact_id:id,artifact_type:artifact_type.into(),work_item_id:work_item_id.into(),version,producer:producer.into(),created_at:now,supersedes:supersedes.map(str::to_string),payload})
    }

    pub fn revision_generation(&self, item:&str)->Result<i64,String>{self.open()?.query_row("SELECT 1+count(*) FROM approvals WHERE work_item_id=?1 AND status='REQUEST_REVISION'",[item],|r|r.get(0)).map_err(|e|e.to_string())}

    pub fn latest_artifact(&self, work_item_id:&str, artifact_type:&str) -> Result<Option<ArtifactRecord>,String> {
        self.open()?.query_row("SELECT artifact_id,artifact_type,work_item_id,version,producer,created_at,supersedes,payload FROM artifacts WHERE work_item_id=?1 AND artifact_type=?2 ORDER BY version DESC LIMIT 1",params![work_item_id,artifact_type],artifact_row).optional().map_err(|e|e.to_string())
    }

    pub fn artifacts(&self, work_item_id:&str) -> Result<Vec<ArtifactRecord>,String> {
        let conn=self.open()?; let mut stmt=conn.prepare("SELECT artifact_id,artifact_type,work_item_id,version,producer,created_at,supersedes,payload FROM artifacts WHERE work_item_id=?1 ORDER BY created_at").map_err(|e|e.to_string())?;
        let mapped=stmt.query_map([work_item_id],artifact_row).map_err(|e|e.to_string())?;
        let result=rows(mapped);
        result
    }

    pub fn create_handoff(&self, logical_key:&str, work_item_id:&str, sender:&str, receiver:&str, artifact_id:&str) -> Result<(),String> {
        let conn=self.open()?; let now=Utc::now().to_rfc3339();
        conn.execute("INSERT OR IGNORE INTO handoffs(handoff_id,logical_key,work_item_id,sender,receiver,artifact_id,status,created_at) VALUES(?1,?2,?3,?4,?5,?6,'READY',?7)",params![format!("handoff-{}",Uuid::new_v4()),logical_key,work_item_id,sender,receiver,artifact_id,now]).map_err(|e|e.to_string())?; Ok(())
    }

    pub fn acknowledge_handoffs(&self, work_item_id:&str, receiver:&str) -> Result<(),String> {
        self.open()?.execute("UPDATE handoffs SET status='ACKNOWLEDGED',acknowledged_at=?3 WHERE work_item_id=?1 AND receiver=?2 AND status='READY'",params![work_item_id,receiver,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?; Ok(())
    }

    pub fn handoffs(&self, work_item_id:&str) -> Result<Vec<HandoffRecord>,String> {
        let conn=self.open()?; let mut stmt=conn.prepare("SELECT handoff_id,work_item_id,sender,receiver,artifact_id,status,created_at,acknowledged_at FROM handoffs WHERE work_item_id=?1 ORDER BY created_at").map_err(|e|e.to_string())?;
        let mapped=stmt.query_map([work_item_id],|r|Ok(HandoffRecord{handoff_id:r.get(0)?,work_item_id:r.get(1)?,sender:r.get(2)?,receiver:r.get(3)?,artifact_id:r.get(4)?,status:r.get(5)?,created_at:r.get(6)?,acknowledged_at:r.get(7)?})).map_err(|e|e.to_string())?;
        let result=rows(mapped);
        result
    }

    pub fn start_run(&self, logical_key:&str, work_item_id:&str, agent:&str, task_type:&str, provider:&str, model:&str) -> Result<Option<String>,String> {
        let id=format!("run-{}",Uuid::new_v4());
        let conn=self.open()?;
        let now=Utc::now().to_rfc3339();
        let changed=conn.execute("INSERT OR IGNORE INTO agent_runs(run_id,logical_key,work_item_id,agent,task_type,provider,model,started_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![id,logical_key,work_item_id,agent,task_type,provider,model,now]).map_err(|e|e.to_string())?;
        if changed==1{return Ok(Some(id));}
        let retry=conn.execute("UPDATE agent_runs SET run_id=?2,provider=?3,model=?4,started_at=?5,finished_at=NULL,success=0,escalation_occurred=0,token_usage=NULL,estimated_api_cost=NULL,error=NULL WHERE logical_key=?1 AND finished_at IS NOT NULL AND success=0",params![logical_key,id,provider,model,now]).map_err(|e|e.to_string())?;
        Ok((retry==1).then_some(id))
    }

    pub fn finish_run(&self, run_id:&str, success:bool, escalated:bool, token_usage:Option<i64>, cost:Option<f64>, error:Option<&str>) -> Result<(),String> {
        self.open()?.execute("UPDATE agent_runs SET finished_at=?2,success=?3,escalation_occurred=?4,token_usage=?5,estimated_api_cost=?6,error=?7 WHERE run_id=?1",params![run_id,Utc::now().to_rfc3339(),success as i32,escalated as i32,token_usage,cost,error]).map_err(|e|e.to_string())?; Ok(())
    }

    pub fn update_run_provider(&self, run_id:&str, provider:&str, model:&str) -> Result<(),String> {
        self.open()?.execute("UPDATE agent_runs SET provider=?2,model=?3 WHERE run_id=?1",params![run_id,provider,model]).map_err(|e|e.to_string())?; Ok(())
    }

    pub fn runs(&self, work_item_id:Option<&str>) -> Result<Vec<AgentRunRecord>,String> {
        let conn=self.open()?; let sql=if work_item_id.is_some(){"SELECT run_id,work_item_id,agent,task_type,provider,model,started_at,finished_at,success,escalation_occurred,token_usage,estimated_api_cost,error FROM agent_runs WHERE work_item_id=?1 ORDER BY started_at DESC"}else{"SELECT run_id,work_item_id,agent,task_type,provider,model,started_at,finished_at,success,escalation_occurred,token_usage,estimated_api_cost,error FROM agent_runs ORDER BY started_at DESC LIMIT 200"};
        let mut stmt=conn.prepare(sql).map_err(|e|e.to_string())?;
        let result=if let Some(id)=work_item_id { let mapped=stmt.query_map([id],run_row).map_err(|e|e.to_string())?; rows(mapped) } else { let mapped=stmt.query_map([],run_row).map_err(|e|e.to_string())?; rows(mapped) };
        result
    }

    pub fn event(&self, work_item_id:Option<&str>, event_type:&str, actor:&str, detail:Value) -> Result<(),String> {
        self.open()?.execute("INSERT INTO system_events VALUES(?1,?2,?3,?4,?5,?6)",params![format!("event-{}",Uuid::new_v4()),work_item_id,event_type,actor,detail.to_string(),Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?; Ok(())
    }

    pub fn events(&self, work_item_id:Option<&str>) -> Result<Vec<SystemEventRecord>,String> {
        let conn=self.open()?; let sql=if work_item_id.is_some(){"SELECT event_id,work_item_id,event_type,actor,detail,created_at FROM system_events WHERE work_item_id=?1 ORDER BY created_at DESC"}else{"SELECT event_id,work_item_id,event_type,actor,detail,created_at FROM system_events ORDER BY created_at DESC LIMIT 200"};
        let mut stmt=conn.prepare(sql).map_err(|e|e.to_string())?;
        let result=if let Some(id)=work_item_id { let mapped=stmt.query_map([id],event_row).map_err(|e|e.to_string())?; rows(mapped) } else { let mapped=stmt.query_map([],event_row).map_err(|e|e.to_string())?; rows(mapped) };
        result
    }

    pub fn create_approval(&self, item:&str, content:&ArtifactRecord, creative:&ArtifactRecord, package:&ArtifactRecord, status:&str, feedback:Option<&str>) -> Result<ApprovalRecord,String> {
        let conn=self.open()?; let now=Utc::now().to_rfc3339();
        conn.execute("UPDATE approvals SET superseded_at=?2 WHERE work_item_id=?1 AND superseded_at IS NULL",params![item,now]).map_err(|e|e.to_string())?;
        let record=ApprovalRecord{approval_id:format!("approval-{}",Uuid::new_v4()),work_item_id:item.into(),approver:"Sam".into(),content_artifact_id:content.artifact_id.clone(),content_version:content.version,creative_artifact_id:creative.artifact_id.clone(),creative_version:creative.version,package_artifact_id:package.artifact_id.clone(),package_version:package.version,platform_scope:vec!["LINKEDIN".into()],status:status.into(),feedback:feedback.map(str::to_string),created_at:now,superseded_at:None};
        conn.execute("INSERT INTO approvals VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,NULL)",params![record.approval_id,record.work_item_id,record.approver,record.content_artifact_id,record.content_version,record.creative_artifact_id,record.creative_version,record.package_artifact_id,record.package_version,serde_json::to_string(&record.platform_scope).unwrap(),record.status,record.feedback,record.created_at]).map_err(|e|e.to_string())?;
        Ok(record)
    }

    pub fn active_approval(&self, item:&str) -> Result<Option<ApprovalRecord>,String> {
        self.open()?.query_row("SELECT approval_id,work_item_id,approver,content_artifact_id,content_version,creative_artifact_id,creative_version,package_artifact_id,package_version,platform_scope,status,feedback,created_at,superseded_at FROM approvals WHERE work_item_id=?1 AND superseded_at IS NULL ORDER BY created_at DESC LIMIT 1",[item],approval_row).optional().map_err(|e|e.to_string())
    }

    pub fn approvals(&self, item:Option<&str>) -> Result<Vec<ApprovalRecord>,String> {
        let conn=self.open()?; let sql=if item.is_some(){"SELECT approval_id,work_item_id,approver,content_artifact_id,content_version,creative_artifact_id,creative_version,package_artifact_id,package_version,platform_scope,status,feedback,created_at,superseded_at FROM approvals WHERE work_item_id=?1 ORDER BY created_at DESC"}else{"SELECT approval_id,work_item_id,approver,content_artifact_id,content_version,creative_artifact_id,creative_version,package_artifact_id,package_version,platform_scope,status,feedback,created_at,superseded_at FROM approvals ORDER BY created_at DESC"};
        let mut stmt=conn.prepare(sql).map_err(|e|e.to_string())?;
        let result=if let Some(id)=item { let mapped=stmt.query_map([id],approval_row).map_err(|e|e.to_string())?; rows(mapped) } else { let mapped=stmt.query_map([],approval_row).map_err(|e|e.to_string())?; rows(mapped) };
        result
    }

    pub fn create_publication(&self,item:&str,receipt_id:&str)->Result<String,String>{
        self.create_publication_with_ref(item, receipt_id, "LOCAL_LEDGER", &format!("ledger://local/{item}"))
    }
    pub fn create_publication_with_ref(&self,item:&str,receipt_id:&str,platform:&str,external_reference:&str)->Result<String,String>{
        let conn=self.open()?;
        if let Ok(existing)=conn.query_row("SELECT publication_id FROM publication_records WHERE work_item_id=?1 AND external_reference=?2",[item,external_reference],|r|r.get::<_,String>(0)) {
            return Ok(existing);
        }
        let id=format!("publication-{}",Uuid::new_v4());
        conn.execute("INSERT INTO publication_records VALUES(?1,?2,?3,?4,?5,'PUBLISHED',?6)",params![id,item,receipt_id,platform,external_reference,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        Ok(id)
    }
    pub fn create_snapshot(&self,item:&str,publication_id:&str,payload:&Value)->Result<String,String>{let id=format!("snapshot-{}",Uuid::new_v4());self.open()?.execute("INSERT INTO performance_snapshots VALUES(?1,?2,?3,'EARLY',?4,?5)",params![id,item,publication_id,payload.to_string(),Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;Ok(id)}

    pub fn insert_learning(&self, source_work_item_id:&str, payload:Value) -> Result<LearningRecord,String> {
        let id=format!("learning-{}",Uuid::new_v4());
        let now=Utc::now().to_rfc3339();
        let fp=payload.get("opportunity_fingerprint").and_then(Value::as_str).unwrap_or("").to_string();
        let decision=payload.get("cycle_decision").and_then(Value::as_str).unwrap_or("CONTINUE").to_string();
        let quality=payload.get("data_quality").and_then(Value::as_str).unwrap_or("UNAVAILABLE").to_string();
        let attention=payload.get("recommended_attention").and_then(Value::as_str).map(str::to_string);
        self.open()?.execute("INSERT INTO learning_records VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![id,source_work_item_id,fp,decision,quality,attention,payload.to_string(),now]).map_err(|e|e.to_string())?;
        Ok(LearningRecord{learning_id:id,source_work_item_id:source_work_item_id.into(),opportunity_fingerprint:fp,cycle_decision:decision,data_quality:quality,recommended_attention:attention,payload,created_at:now})
    }
    pub fn recent_learning(&self, limit: i64) -> Result<Vec<LearningRecord>,String> {
        let conn=self.open()?;
        let mut stmt=conn.prepare("SELECT learning_id,source_work_item_id,opportunity_fingerprint,cycle_decision,data_quality,recommended_attention,payload,created_at FROM learning_records ORDER BY created_at DESC LIMIT ?1").map_err(|e|e.to_string())?;
        let mapped=stmt.query_map([limit], learning_row).map_err(|e|e.to_string())?;
        rows(mapped)
    }
    pub fn record_execution(&self, work_item_id:Option<&str>, role:&str, capability:&str, kind:&str, provider:Option<&str>, model:Option<&str>, success:bool, error:Option<&str>) -> Result<String,String> {
        let id=format!("exec-{}",Uuid::new_v4());
        let now=Utc::now().to_rfc3339();
        let logical=format!("{}/{}/{}/{}/{}", work_item_id.unwrap_or("-"), capability, kind, now, Uuid::new_v4());
        self.open()?.execute("INSERT INTO executions(execution_id,logical_key,work_item_id,role,capability,kind,provider,model,started_at,finished_at,success,error) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9,?10,?11)",
            params![id,logical,work_item_id,role,capability,kind,provider,model,now,success as i32,error]).map_err(|e|e.to_string())?;
        Ok(id)
    }
    pub fn executions(&self, work_item_id:Option<&str>) -> Result<Vec<ExecutionRecord>,String> {
        let conn=self.open()?;
        let sql=if work_item_id.is_some(){"SELECT execution_id,work_item_id,role,capability,kind,provider,model,started_at,finished_at,success,error FROM executions WHERE work_item_id=?1 ORDER BY started_at DESC"}else{"SELECT execution_id,work_item_id,role,capability,kind,provider,model,started_at,finished_at,success,error FROM executions ORDER BY started_at DESC LIMIT 200"};
        let mut stmt=conn.prepare(sql).map_err(|e|e.to_string())?;
        if let Some(id)=work_item_id { let mapped=stmt.query_map([id], exec_row).map_err(|e|e.to_string())?; rows(mapped) } else { let mapped=stmt.query_map([], exec_row).map_err(|e|e.to_string())?; rows(mapped) }
    }

    pub fn detail(&self,id:&str)->Result<WorkItemDetail,String>{Ok(WorkItemDetail{work_item:self.work_item(id)?,artifacts:self.artifacts(id)?,handoffs:self.handoffs(id)?,approvals:self.approvals(Some(id))?,runs:self.runs(Some(id))?,events:self.events(Some(id))?,executions:self.executions(Some(id))?})}
    pub fn count_state(&self,state:&str)->Result<i64,String>{self.open()?.query_row("SELECT count(*) FROM work_items WHERE state=?1",[state],|r|r.get(0)).map_err(|e|e.to_string())}
}

fn rows<T>(mapped: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>)->rusqlite::Result<T>>) -> Result<Vec<T>,String> { mapped.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string()) }
fn work_row(r:&rusqlite::Row<'_>)->rusqlite::Result<WorkItemSummary>{Ok(WorkItemSummary{work_item_id:r.get(0)?,title:r.get(1)?,state:r.get(2)?,current_owner:r.get(3)?,created_at:r.get(4)?,updated_at:r.get(5)?,blocked_reason:r.get(6)?,resume_state:r.get(7)?})}
fn artifact_row(r:&rusqlite::Row<'_>)->rusqlite::Result<ArtifactRecord>{let text:String=r.get(7)?;Ok(ArtifactRecord{artifact_id:r.get(0)?,artifact_type:r.get(1)?,work_item_id:r.get(2)?,version:r.get(3)?,producer:r.get(4)?,created_at:r.get(5)?,supersedes:r.get(6)?,payload:serde_json::from_str(&text).unwrap_or(Value::Null)})}
fn run_row(r:&rusqlite::Row<'_>)->rusqlite::Result<AgentRunRecord>{Ok(AgentRunRecord{run_id:r.get(0)?,work_item_id:r.get(1)?,agent:r.get(2)?,task_type:r.get(3)?,provider:r.get(4)?,model:r.get(5)?,started_at:r.get(6)?,finished_at:r.get(7)?,success:r.get::<_,i64>(8)?!=0,escalation_occurred:r.get::<_,i64>(9)?!=0,token_usage:r.get(10)?,estimated_api_cost:r.get(11)?,error:r.get(12)?})}
fn event_row(r:&rusqlite::Row<'_>)->rusqlite::Result<SystemEventRecord>{let text:String=r.get(4)?;Ok(SystemEventRecord{event_id:r.get(0)?,work_item_id:r.get(1)?,event_type:r.get(2)?,actor:r.get(3)?,detail:serde_json::from_str(&text).unwrap_or(Value::Null),created_at:r.get(5)?})}
fn approval_row(r:&rusqlite::Row<'_>)->rusqlite::Result<ApprovalRecord>{let scope:String=r.get(9)?;Ok(ApprovalRecord{approval_id:r.get(0)?,work_item_id:r.get(1)?,approver:r.get(2)?,content_artifact_id:r.get(3)?,content_version:r.get(4)?,creative_artifact_id:r.get(5)?,creative_version:r.get(6)?,package_artifact_id:r.get(7)?,package_version:r.get(8)?,platform_scope:serde_json::from_str(&scope).unwrap_or_default(),status:r.get(10)?,feedback:r.get(11)?,created_at:r.get(12)?,superseded_at:r.get(13)?})}
fn learning_row(r:&rusqlite::Row<'_>)->rusqlite::Result<LearningRecord>{let text:String=r.get(6)?;Ok(LearningRecord{learning_id:r.get(0)?,source_work_item_id:r.get(1)?,opportunity_fingerprint:r.get(2)?,cycle_decision:r.get(3)?,data_quality:r.get(4)?,recommended_attention:r.get(5)?,payload:serde_json::from_str(&text).unwrap_or(Value::Null),created_at:r.get(7)?})}
fn exec_row(r:&rusqlite::Row<'_>)->rusqlite::Result<ExecutionRecord>{Ok(ExecutionRecord{execution_id:r.get(0)?,work_item_id:r.get(1)?,role:r.get(2)?,capability:r.get(3)?,kind:r.get(4)?,provider:r.get(5)?,model:r.get(6)?,started_at:r.get(7)?,finished_at:r.get(8)?,success:r.get::<_,i64>(9)?!=0,error:r.get(10)?})}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sqlite_persists_versions_and_duplicate_handoffs(){
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("core.sqlite");
        let db=Database::new(&path).unwrap();let work=db.create_work_item("test","signal").unwrap();
        let a1=db.insert_artifact(&work.work_item_id,"CONTENT_DRAFT",1,"brain",None,json!({"v":1})).unwrap();
        let a2=db.insert_artifact(&work.work_item_id,"CONTENT_DRAFT",2,"brain",Some(&a1.artifact_id),json!({"v":2})).unwrap();assert_eq!(a2.supersedes.as_deref(),Some(a1.artifact_id.as_str()));
        db.create_handoff("same",&work.work_item_id,"brain","jax",&a1.artifact_id).unwrap();db.create_handoff("same",&work.work_item_id,"brain","jax",&a1.artifact_id).unwrap();
        let first_run=db.start_run("logical-run",&work.work_item_id,"brain","writing","OLLAMA_PROVIDER","qwen3:14b").unwrap();let duplicate_run=db.start_run("logical-run",&work.work_item_id,"brain","writing","OLLAMA_PROVIDER","qwen3:14b").unwrap();assert!(first_run.is_some());assert!(duplicate_run.is_none());
        db.finish_run(first_run.as_deref().unwrap(),false,false,None,None,Some("retryable failure")).unwrap();
        let retry=db.start_run("logical-run",&work.work_item_id,"brain","writing","OLLAMA_PROVIDER","qwen3:14b").unwrap();assert!(retry.is_some());
        assert!(db.start_run("logical-run",&work.work_item_id,"brain","writing","OLLAMA_PROVIDER","qwen3:14b").unwrap().is_none());
        drop(db);let reopened=Database::new(&path).unwrap();assert_eq!(reopened.latest_artifact(&work.work_item_id,"CONTENT_DRAFT").unwrap().unwrap().version,2);assert_eq!(reopened.handoffs(&work.work_item_id).unwrap().len(),1);
    }

    #[test]
    fn blocked_requires_and_preserves_valid_resume_state() {
        let dir=tempfile::tempdir().unwrap();
        let db=Database::new(dir.path().join("core.sqlite")).unwrap();
        let work=db.create_work_item("test","signal").unwrap();
        db.update_work_item(&work.work_item_id,"IN_PRODUCTION","brain + jax",None).unwrap();
        let stored=db.enter_blocked(&work.work_item_id,"brain + jax","Provider output is not JSON","STRATEGIZED").unwrap();
        assert_eq!(stored,"STRATEGIZED");
        let item=db.work_item(&work.work_item_id).unwrap();
        assert_eq!(item.state,"BLOCKED");
        assert_eq!(item.resume_state.as_deref(),Some("STRATEGIZED"));
        assert!(db.enter_blocked(&work.work_item_id,"system","BLOCKED is missing resume_state","BLOCKED").is_ok());
        let preserved=db.work_item(&work.work_item_id).unwrap();
        assert_eq!(preserved.resume_state.as_deref(),Some("STRATEGIZED"));
        db.update_work_item(&work.work_item_id,"IN_PRODUCTION","brain + jax",None).unwrap();
        assert!(db.update_work_item_with_resume(&work.work_item_id,"BLOCKED","system",Some("x"),None).is_err());
        assert!(db.update_work_item_with_resume(&work.work_item_id,"BLOCKED","system",Some("x"),Some("NEW")).is_err());
        assert!(db.update_work_item_with_resume(&work.work_item_id,"BLOCKED","system",Some("x"),Some("BLOCKED")).is_err());
    }
}
