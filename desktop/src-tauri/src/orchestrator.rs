use serde_json::{json, Value};
use crate::{
    db::Database,
    definitions::DefinitionStore,
    models::*,
    providers,
    runner::AgentRunner,
    schema_fixture::generate_for_type,
};

#[derive(Clone)]
pub struct RuntimeService {
    pub db: Database,
    pub definitions: DefinitionStore,
    runner: AgentRunner,
}

impl RuntimeService {
    pub fn new(db: Database, definitions: DefinitionStore) -> Result<Self, String> {
        db.sync_employees(&definitions.employees())?;
        let runner = AgentRunner::new(db.clone(), definitions.clone());
        Ok(Self { db, definitions, runner })
    }

    pub async fn provider_status(&self) -> Result<ProviderStatus, String> {
        Ok(providers::detect(&self.db.settings()?).await)
    }

    pub async fn dashboard(&self) -> Result<Dashboard, String> {
        let settings = self.db.settings()?;
        let employees = self.db.employees()?;
        Ok(Dashboard {
            company_state: settings.company_control,
            provider_status: self.provider_status().await?,
            active_agents: employees.iter().filter(|e| e.runtime_status == "WORKING").map(|e| e.employee_id.clone()).collect(),
            waiting_approvals: self.db.count_state("READY_FOR_APPROVAL")?,
            blocked_work: self.db.count_state("BLOCKED")?,
            work_items: self.db.work_items()?,
        })
    }

    pub fn start(&self, request: StartWorkItemRequest) -> Result<WorkItemSummary, String> {
        self.db.create_work_item(&request.title, &request.research_signal)
    }

    pub async fn advance_until_blocked(&self, id: &str, test_mode: bool) -> Result<WorkItemDetail, String> {
        loop {
            let control = self.db.settings()?.company_control;
            if control != "RUNNING" {
                self.db.event(Some(id), "orchestrator.halted", "system", json!({"control": control}))?;
                break;
            }
            let item = self.db.work_item(id)?;
            let result: Result<bool, String> = match item.state.as_str() {
                "NEW" => self.agent_step(id, "saly", "research", "OPPORTUNITY_BATCH", "RESEARCH_ASSIGNMENT", "RESEARCHED", "travis", test_mode).await,
                "RESEARCHED" => self.travis_select(id, test_mode).await,
                "SELECTED" => self.agent_step(id, "adam", "strategy", "CONTENT_BRIEF", "ASSIGNMENT", "STRATEGIZED", "brain + jax", test_mode).await,
                "STRATEGIZED" => self.begin_production(id),
                "IN_PRODUCTION" => self.complete_production(id, test_mode).await,
                "REVISION_REQUIRED" => self.complete_revision(id, test_mode).await,
                "APPROVED" => { self.db.update_work_item(id, "READY_TO_PUBLISH", "maro", None)?; Ok(true) }
                "READY_TO_PUBLISH" => self.publish(id, test_mode).await,
                "PUBLISHED" => self.snapshot(id),
                "MEASURING" => self.measure_and_learn(id, test_mode).await,
                "BLOCKED" => self.resume_retryable_block(id),
                "READY_FOR_APPROVAL" | "MEASURED" | "REJECTED" | "CANCELLED" => Ok(false),
                other => Err(format!("Unsupported workflow state {other}")),
            };
            match result {
                Ok(true) => continue,
                Ok(false) => break,
                Err(error) => {
                    self.db.update_work_item(id, "BLOCKED", "system", Some(&error))?;
                    self.db.event(Some(id), "workflow.blocked", "system", json!({"error": error}))?;
                    break;
                }
            }
        }
        self.db.detail(id)
    }

    async fn agent_step(&self, id: &str, agent: &str, task: &str, output: &str, input: &str, next_state: &str, next_owner: &str, test: bool) -> Result<bool, String> {
        let context = self.context(id, input, json!({}))?;
        let artifact = self.runner.run(id, agent, task, output, context, test).await?;
        self.db.acknowledge_handoffs(id, agent)?;
        self.db.create_handoff(&format!("{id}:{output}:{next_owner}:{}", artifact.version), id, agent, next_owner, &artifact.artifact_id)?;
        self.db.update_work_item(id, next_state, next_owner, None)?;
        Ok(true)
    }

    async fn travis_select(&self, id: &str, test: bool) -> Result<bool, String> {
        let context = self.context(id, "OPPORTUNITY_BATCH", json!({}))?;
        let decision = self.runner.run(id, "travis", "prioritization", "GROWTH_DECISION", context, test).await?;
        let definition = self.definitions.get("travis")?;
        let generation = self.db.revision_generation(id)?;
        let assignment_payload = generate_for_type(&definition.output_schema, "ASSIGNMENT");
        let prior = self.db.latest_artifact(id, "ASSIGNMENT")?;
        let assignment = self.db.insert_artifact(id, "ASSIGNMENT", generation, "travis", prior.as_ref().map(|a| a.artifact_id.as_str()), assignment_payload)?;
        self.db.create_handoff(&format!("{id}:assignment:adam:{generation}"), id, "travis", "adam", &assignment.artifact_id)?;
        self.db.event(Some(id), "decision.selected", "travis", json!({"decision_id": decision.artifact_id, "assignment_id": assignment.artifact_id}))?;
        self.db.update_work_item(id, "SELECTED", "adam", None)?;
        Ok(true)
    }

    fn begin_production(&self, id: &str) -> Result<bool, String> {
        let brief = self.required_artifact(id, "CONTENT_BRIEF")?;
        self.db.create_handoff(&format!("{id}:brief:brain:{}", brief.version), id, "adam", "brain", &brief.artifact_id)?;
        self.db.create_handoff(&format!("{id}:brief:jax:{}", brief.version), id, "adam", "jax", &brief.artifact_id)?;
        self.db.event(Some(id), "jax.proof_planning_eligible", "system", json!({"brief_id": brief.artifact_id, "rule": "Jax may plan from brief; final creative waits for exact Brain draft"}))?;
        self.db.update_work_item(id, "IN_PRODUCTION", "brain + jax", None)?;
        Ok(true)
    }

    async fn complete_production(&self, id: &str, test: bool) -> Result<bool, String> {
        let brief = self.required_artifact(id, "CONTENT_BRIEF")?;
        let draft_context = self.context(id, "CONTENT_BRIEF", json!({"_bindings": {"content_brief_id": brief.artifact_id}}))?;
        let draft = self.runner.run(id, "brain", "public_writing", "CONTENT_DRAFT", draft_context, test).await?;
        self.db.acknowledge_handoffs(id, "brain")?;
        self.db.create_handoff(&format!("{id}:draft:jax:{}", draft.version), id, "brain", "jax", &draft.artifact_id)?;
        let creative_context = self.context(id, "CONTENT_DRAFT", json!({"_bindings": {
            "content_brief_id": brief.artifact_id, "content_draft_id": draft.artifact_id,
            "draft_id": draft.artifact_id, "draft_version": draft.version.to_string(),
            "content_version": draft.version.to_string()
        }}))?;
        let creative = self.runner.run(id, "jax", "creative_planning", "CREATIVE_PACKAGE", creative_context, test).await?;
        self.db.acknowledge_handoffs(id, "jax")?;
        self.assemble(id, &brief, &draft, &creative)?;
        Ok(false)
    }

    async fn complete_revision(&self, id: &str, test: bool) -> Result<bool, String> {
        self.db.update_work_item(id, "IN_PRODUCTION", "brain + jax", None)?;
        self.complete_production(id, test).await
    }

    fn assemble(&self, id: &str, brief: &ArtifactRecord, draft: &ArtifactRecord, creative: &ArtifactRecord) -> Result<ArtifactRecord, String> {
        if brief.work_item_id != id || draft.work_item_id != id || creative.work_item_id != id {
            return Err("Package components do not belong to the same work item".into());
        }
        let generation = self.db.revision_generation(id)?;
        let prior = self.db.latest_artifact(id, "PUBLISH_PACKAGE")?;
        let payload = json!({
            "artifact_type": "PUBLISH_PACKAGE", "work_item_id": id,
            "content_brief": {"artifact_id": brief.artifact_id, "version": brief.version},
            "content_draft": {"artifact_id": draft.artifact_id, "version": draft.version},
            "creative_package": {"artifact_id": creative.artifact_id, "version": creative.version, "compatible_draft_id": draft.artifact_id},
            "target_platforms": ["LINKEDIN"], "asset_references": [],
            "required_metadata": {"title": "Practical AI for Real Work"},
            "approval_sensitive_flags": ["PUBLIC_CONTENT"], "status": "AWAITING_APPROVAL"
        });
        let package = self.db.insert_artifact(id, "PUBLISH_PACKAGE", generation, "system", prior.as_ref().map(|a| a.artifact_id.as_str()), payload)?;
        self.db.create_handoff(&format!("{id}:package:sam:{generation}"), id, "system", "sam", &package.artifact_id)?;
        self.db.event(Some(id), "package.assembled", "system", json!({"package_id": package.artifact_id, "draft_id": draft.artifact_id, "creative_id": creative.artifact_id}))?;
        self.db.update_work_item(id, "READY_FOR_APPROVAL", "sam", None)?;
        Ok(package)
    }

    pub async fn approval(&self, decision: ApprovalDecision, test_mode: bool) -> Result<WorkItemDetail, String> {
        let item = self.db.work_item(&decision.work_item_id)?;
        if item.state != "READY_FOR_APPROVAL" { return Err(format!("Approval action is invalid from {}", item.state)); }
        let content = self.required_artifact(&decision.work_item_id, "CONTENT_DRAFT")?;
        let creative = self.required_artifact(&decision.work_item_id, "CREATIVE_PACKAGE")?;
        let package = self.required_artifact(&decision.work_item_id, "PUBLISH_PACKAGE")?;
        match decision.action.as_str() {
            "APPROVE" => {
                self.db.create_approval(&decision.work_item_id, &content, &creative, &package, "APPROVED", decision.feedback.as_deref())?;
                self.db.update_work_item(&decision.work_item_id, "APPROVED", "maro", None)?;
                self.db.event(Some(&decision.work_item_id), "approval.granted", "sam", json!({"package_id": package.artifact_id, "package_version": package.version}))?;
                self.advance_until_blocked(&decision.work_item_id, test_mode).await
            }
            "REQUEST_REVISION" => {
                self.db.create_approval(&decision.work_item_id, &content, &creative, &package, "REQUEST_REVISION", decision.feedback.as_deref())?;
                self.db.update_work_item(&decision.work_item_id, "REVISION_REQUIRED", "brain + jax", None)?;
                self.db.event(Some(&decision.work_item_id), "approval.revision_requested", "sam", json!({"feedback": decision.feedback}))?;
                self.db.detail(&decision.work_item_id)
            }
            "REJECT" => {
                self.db.create_approval(&decision.work_item_id, &content, &creative, &package, "REJECTED", decision.feedback.as_deref())?;
                self.db.update_work_item(&decision.work_item_id, "REJECTED", "sam", None)?;
                self.db.event(Some(&decision.work_item_id), "approval.rejected", "sam", json!({"feedback": decision.feedback}))?;
                self.db.detail(&decision.work_item_id)
            }
            _ => Err("Action must be APPROVE, REQUEST_REVISION, or REJECT".into()),
        }
    }

    async fn publish(&self, id: &str, test: bool) -> Result<bool, String> {
        self.assert_valid_approval(id)?;
        let package = self.required_artifact(id, "PUBLISH_PACKAGE")?;
        let approval = self.db.active_approval(id)?.ok_or("Missing approval")?;
        let context = self.context(id, "PUBLISH_PACKAGE", json!({"_bindings": {"publish_package_id": package.artifact_id, "approval_id": approval.approval_id, "approval_status": "APPROVED"}}))?;
        let receipt = self.runner.run(id, "maro", "publication", "PUBLICATION_RECEIPT", context.clone(), test).await?;
        let result = self.runner.run(id, "maro", "publication_result", "DISTRIBUTION_RESULT", context, test).await?;
        let publication_id = self.db.create_publication(id, &receipt.artifact_id)?;
        self.db.create_handoff(&format!("{id}:publication:lara:1"), id, "maro", "lara", &receipt.artifact_id)?;
        self.db.event(Some(id), "publication.mocked", "maro", json!({"publication_id": publication_id, "receipt_id": receipt.artifact_id, "distribution_result_id": result.artifact_id, "external_api": false}))?;
        self.db.update_work_item(id, "PUBLISHED", "system", None)?;
        Ok(true)
    }

    fn snapshot(&self, id: &str) -> Result<bool, String> {
        let receipt = self.required_artifact(id, "PUBLICATION_RECEIPT")?;
        let payload = json!({"artifact_type": "PERFORMANCE_SNAPSHOT", "publication_receipt_id": receipt.artifact_id, "measurement_window": "EARLY", "data_quality": "COMPLETE", "metrics": {"impressions": 1840, "qualified_engagements": 72, "profile_visits": 31, "saves": 26, "comments": 14}, "provenance": "PHASE_4A_DETERMINISTIC_TEST_ADAPTER", "mocked": true});
        let artifact = self.db.insert_artifact(id, "PERFORMANCE_SNAPSHOT", 1, "system", None, payload.clone())?;
        let publication_id = format!("publication-for-{}", receipt.artifact_id);
        self.db.create_snapshot(id, &publication_id, &payload)?;
        self.db.create_handoff(&format!("{id}:snapshot:lara:1"), id, "system", "lara", &artifact.artifact_id)?;
        self.db.event(Some(id), "performance_snapshot.imported", "system", json!({"snapshot_id": artifact.artifact_id, "deterministic": true}))?;
        self.db.update_work_item(id, "MEASURING", "lara", None)?;
        Ok(true)
    }

    async fn measure_and_learn(&self, id: &str, test: bool) -> Result<bool, String> {
        let context = self.context(id, "PERFORMANCE_SNAPSHOT", json!({}))?;
        let insight = self.runner.run(id, "lara", "performance_analysis", "PERFORMANCE_INSIGHT", context, test).await?;
        self.db.acknowledge_handoffs(id, "lara")?;
        self.db.create_handoff(&format!("{id}:insight:travis:1"), id, "lara", "travis", &insight.artifact_id)?;
        let context = self.context(id, "PERFORMANCE_INSIGHT", json!({}))?;
        let decision = self.runner.run(id, "travis", "learning_decision", "CYCLE_DECISION", context, test).await?;
        self.db.acknowledge_handoffs(id, "travis")?;
        self.db.event(Some(id), "cycle.completed", "travis", json!({"cycle_decision_id": decision.artifact_id}))?;
        self.db.update_work_item(id, "MEASURED", "travis", None)?;
        Ok(false)
    }

    fn assert_valid_approval(&self, id: &str) -> Result<(), String> {
        let a = self.db.active_approval(id)?.ok_or("Maro blocked: explicit Sam approval is missing")?;
        if a.status != "APPROVED" { return Err("Maro blocked: approval is not APPROVED".into()); }
        let c = self.required_artifact(id, "CONTENT_DRAFT")?;
        let j = self.required_artifact(id, "CREATIVE_PACKAGE")?;
        let p = self.required_artifact(id, "PUBLISH_PACKAGE")?;
        if a.content_artifact_id != c.artifact_id || a.content_version != c.version || a.creative_artifact_id != j.artifact_id || a.creative_version != j.version || a.package_artifact_id != p.artifact_id || a.package_version != p.version {
            return Err("Maro blocked: approval is stale or version-mismatched".into());
        }
        Ok(())
    }

    fn required_artifact(&self, id: &str, kind: &str) -> Result<ArtifactRecord, String> {
        self.db.latest_artifact(id, kind)?.ok_or_else(|| format!("Missing required {kind}"))
    }

    fn resume_retryable_block(&self, id: &str) -> Result<bool, String> {
        let item=self.db.work_item(id)?;
        let retryable=item.blocked_reason.as_deref().map(|reason|reason.starts_with("Ollama request failed")||reason.starts_with("Provider output is not JSON")).unwrap_or(false);
        if !retryable{return Ok(false);}
        if self.db.latest_artifact(id,"PERFORMANCE_SNAPSHOT")?.is_some() && self.db.latest_artifact(id,"PERFORMANCE_INSIGHT")?.is_none(){
            self.db.event(Some(id),"workflow.retrying","system",json!({"resume_state":"MEASURING","reason":item.blocked_reason}))?;
            self.db.update_work_item(id,"MEASURING","lara",None)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn context(&self, id: &str, input_type: &str, extra: Value) -> Result<Value, String> {
        let bindings = extra.get("_bindings").cloned().unwrap_or(Value::Null);
        Ok(json!({"_input_type": input_type, "_bindings": bindings, "work_item": self.db.work_item(id)?, "artifacts": self.db.artifacts(id)?}))
    }

    pub fn control(&self, request: ControlRequest) -> Result<RuntimeSettings, String> {
        let value = match request.action.as_str() { "RUN" | "RESUME" => "RUNNING", "PAUSE" => "PAUSED", "STOP" => "STOPPED", _ => return Err("Control action must be RUN, PAUSE, RESUME, or STOP".into()) };
        self.db.set_control(value)?;
        self.db.event(None, "company.control", "sam", json!({"state": value}))?;
        self.db.settings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::{repository_root, DefinitionStore};

    fn runtime() -> (tempfile::TempDir, RuntimeService) {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::new(dir.path().join("test.sqlite")).unwrap();
        let defs = DefinitionStore::load(repository_root().unwrap()).unwrap();
        (dir, RuntimeService::new(db, defs).unwrap())
    }

    #[tokio::test]
    async fn complete_cycle_stops_for_version_bound_approval_and_finishes() {
        let (_dir, service) = runtime();
        let work = service.start(StartWorkItemRequest { title: "Practical AI signal".into(), research_signal: "Teams need reliable AI workflows".into() }).unwrap();
        let waiting = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(waiting.work_item.state, "READY_FOR_APPROVAL", "{:?}", waiting.work_item.blocked_reason);
        let draft = waiting.artifacts.iter().find(|a| a.artifact_type == "CONTENT_DRAFT").unwrap();
        let creative = waiting.artifacts.iter().find(|a| a.artifact_type == "CREATIVE_PACKAGE").unwrap();
        let package = waiting.artifacts.iter().find(|a| a.artifact_type == "PUBLISH_PACKAGE").unwrap();
        assert_eq!(package.payload.pointer("/content_draft/artifact_id").and_then(Value::as_str), Some(draft.artifact_id.as_str()));
        assert_eq!(package.payload.pointer("/creative_package/artifact_id").and_then(Value::as_str), Some(creative.artifact_id.as_str()));
        assert_eq!(package.payload.pointer("/creative_package/compatible_draft_id").and_then(Value::as_str), Some(draft.artifact_id.as_str()));
        assert!(waiting.events.iter().any(|e| e.event_type == "jax.proof_planning_eligible"));
        assert!(waiting.handoffs.iter().any(|h| h.sender == "adam" && h.receiver == "brain"));
        assert!(waiting.handoffs.iter().any(|h| h.sender == "adam" && h.receiver == "jax"));
        let done = service.approval(ApprovalDecision { work_item_id: work.work_item_id, action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        assert_eq!(done.work_item.state, "MEASURED", "{:?}", done.work_item.blocked_reason);
        for required in ["PUBLICATION_RECEIPT", "DISTRIBUTION_RESULT", "PERFORMANCE_SNAPSHOT", "PERFORMANCE_INSIGHT", "CYCLE_DECISION"] {
            assert!(done.artifacts.iter().any(|a| a.artifact_type == required), "missing {required}");
        }
    }

    #[tokio::test]
    async fn maro_is_blocked_without_approval() {
        let (_dir, service) = runtime();
        let work = service.start(StartWorkItemRequest { title: "x".into(), research_signal: "y".into() }).unwrap();
        service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        service.db.update_work_item(&work.work_item_id, "READY_TO_PUBLISH", "maro", None).unwrap();
        let detail = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(detail.work_item.state, "BLOCKED");
        assert!(detail.work_item.blocked_reason.unwrap().contains("approval"));
    }

    #[tokio::test]
    async fn pause_and_restart_preserve_state() {
        let (dir, service) = runtime();
        let work = service.start(StartWorkItemRequest { title: "x".into(), research_signal: "y".into() }).unwrap();
        service.control(ControlRequest { action: "PAUSE".into() }).unwrap();
        let paused = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(paused.work_item.state, "NEW");
        drop(service);
        let db = Database::new(dir.path().join("test.sqlite")).unwrap();
        let defs = DefinitionStore::load(repository_root().unwrap()).unwrap();
        let reopened = RuntimeService::new(db, defs).unwrap();
        assert_eq!(reopened.db.settings().unwrap().company_control, "PAUSED");
        assert_eq!(reopened.db.work_item(&work.work_item_id).unwrap().state, "NEW");
        reopened.control(ControlRequest { action: "RESUME".into() }).unwrap();
        let resumed = reopened.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(resumed.work_item.state, "READY_FOR_APPROVAL");
    }

    #[tokio::test]
    async fn stale_approval_is_rejected() {
        let (_dir, service) = runtime();
        let work = service.start(StartWorkItemRequest { title: "x".into(), research_signal: "y".into() }).unwrap();
        service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        let content = service.required_artifact(&work.work_item_id, "CONTENT_DRAFT").unwrap();
        let creative = service.required_artifact(&work.work_item_id, "CREATIVE_PACKAGE").unwrap();
        let package = service.required_artifact(&work.work_item_id, "PUBLISH_PACKAGE").unwrap();
        service.db.create_approval(&work.work_item_id, &content, &creative, &package, "APPROVED", None).unwrap();
        service.db.insert_artifact(&work.work_item_id, "CONTENT_DRAFT", 2, "brain", Some(&content.artifact_id), json!({"changed": true})).unwrap();
        assert!(service.assert_valid_approval(&work.work_item_id).unwrap_err().contains("approval"));
    }
}
