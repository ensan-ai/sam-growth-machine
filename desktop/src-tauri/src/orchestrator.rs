use serde_json::{json, Value};

use crate::{
    adapters::{local_observe, local_publish},
    capabilities::{self, Capability, ExecutionKind, EXECUTION_TARGET},
    db::Database,
    definitions::DefinitionStore,
    learning,
    models::*,
    policy,
    producers,
    providers,
    runner::AgentRunner,
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
        let work = self.db.create_work_item(&request.title, &request.research_signal)?;
        let records = self.db.recent_learning(10)?;
        let payload = learning::context_payload(&records);
        self.db.insert_artifact(&work.work_item_id, "LEARNING_CONTEXT", 1, "system", None, payload)?;
        self.db.event(Some(&work.work_item_id), "learning.context_bound", "system", json!({"records": records.len()}))?;
        self.db.work_item(&work.work_item_id)
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
                "NEW" => self.saly_package(id),
                "RESEARCHED" => self.travis_select(id),
                "SELECTED" => self.adam_brief(id, None),
                "STRATEGIZED" => self.begin_production(id),
                "IN_PRODUCTION" => self.complete_production(id, test_mode, None).await,
                "REVISION_REQUIRED" => self.complete_revision(id, test_mode).await,
                "APPROVED" => {
                    self.db.update_work_item(id, "READY_TO_PUBLISH", "maro", None)?;
                    Ok(true)
                }
                "READY_TO_PUBLISH" => self.publish(id, test_mode).await,
                "PUBLISHED" => self.snapshot(id, test_mode),
                "MEASURING" => self.measure_and_learn(id),
                "BLOCKED" => self.resume_blocked(id),
                "READY_FOR_APPROVAL" | "MEASURED" | "REJECTED" | "CANCELLED" => Ok(false),
                other => Err(format!("Unsupported workflow state {other}")),
            };
            match result {
                Ok(true) => continue,
                Ok(false) => break,
                Err(error) => {
                    let current = self.db.work_item(id)?;
                    if current.state == "BLOCKED" {
                        self.db.event(Some(id), "workflow.blocked.preserved", "system", json!({
                            "error": error,
                            "resume_state": current.resume_state,
                            "blocked_reason": current.blocked_reason
                        }))?;
                        break;
                    }
                    match resume_target(&current.state) {
                        Some(resume) => {
                            let stored = self.db.enter_blocked(id, &current.current_owner, &error, resume)?;
                            self.db.event(Some(id), "workflow.blocked", "system", json!({
                                "error": error,
                                "resume_state": stored,
                                "failed_from": current.state
                            }))?;
                        }
                        None => {
                            self.db.event(Some(id), "workflow.error", "system", json!({
                                "error": error,
                                "state": current.state
                            }))?;
                        }
                    }
                    break;
                }
            }
        }
        self.db.detail(id)
    }

    fn saly_package(&self, id: &str) -> Result<bool, String> {
        let cap = capabilities::PACKAGE_OPPORTUNITY;
        self.begin_role(cap)?;
        let signal = self.required_artifact(id, "RESEARCH_SIGNAL")?;
        let learning = self.db.recent_learning(10)?;
        let (card, batch) = producers::package_opportunity(id, &self.stamped(&signal), &learning);
        let generation = self.db.revision_generation(id)?;
        let card_rec = self.db.insert_artifact(id, "OPPORTUNITY_CARD", generation, "saly", None, card)?;
        let batch_rec = self.db.insert_artifact(id, "OPPORTUNITY_BATCH", generation, "saly", None, batch)?;
        self.db.acknowledge_handoffs(id, "saly")?;
        self.db.create_handoff(&format!("{id}:opportunity:travis:{generation}"), id, "saly", "travis", &batch_rec.artifact_id)?;
        self.record(id, cap, None, None, true, None)?;
        self.db.event(Some(id), "opportunity.packaged", "saly", json!({"card_id": card_rec.artifact_id, "batch_id": batch_rec.artifact_id}))?;
        self.db.update_work_item(id, "RESEARCHED", "travis", None)?;
        self.end_role(cap)?;
        Ok(true)
    }

    fn travis_select(&self, id: &str) -> Result<bool, String> {
        let cap = capabilities::PRIORITIZE_AND_ASSIGN;
        self.begin_role(cap)?;
        let card = self.required_artifact(id, "OPPORTUNITY_CARD")?;
        let learning = self.db.recent_learning(10)?;
        let (decision, assignment) = producers::prioritize_and_assign(id, &self.stamped(&card), &learning)?;
        let generation = self.db.revision_generation(id)?;
        let decision_rec = self.db.insert_artifact(id, "GROWTH_DECISION", generation, "travis", None, decision.clone())?;
        self.db.acknowledge_handoffs(id, "travis")?;
        if assignment.is_none() {
            let signal = self.research_signal(id)?;
            let cycle = json!({
                "decision": "STOP",
                "opportunity_fingerprint": decision.get("opportunity_fingerprint"),
                "data_quality": "UNAVAILABLE",
                "recommended_attention": "NEGATIVE_SIGNAL",
                "next_action": "Do not rediscover this opportunity without new evidence"
            });
            self.persist_learning(id, &signal, &cycle, None, &[&decision_rec.artifact_id])?;
            self.record(id, cap, None, None, true, None)?;
            self.db.event(Some(id), "decision.rejected", "travis", json!({"decision_id": decision_rec.artifact_id}))?;
            self.db.update_work_item(id, "REJECTED", "travis", None)?;
            self.end_role(cap)?;
            return Ok(false);
        }
        let assignment_rec = self.db.insert_artifact(id, "ASSIGNMENT", generation, "travis", None, assignment.unwrap())?;
        self.db.create_handoff(&format!("{id}:assignment:adam:{generation}"), id, "travis", "adam", &assignment_rec.artifact_id)?;
        self.record(id, cap, None, None, true, None)?;
        self.db.event(Some(id), "decision.selected", "travis", json!({"decision_id": decision_rec.artifact_id, "assignment_id": assignment_rec.artifact_id}))?;
        self.db.update_work_item(id, "SELECTED", "adam", None)?;
        self.end_role(cap)?;
        Ok(true)
    }

    fn adam_brief(&self, id: &str, revision: Option<&Value>) -> Result<bool, String> {
        let cap = capabilities::COMPILE_BRIEF;
        self.begin_role(cap)?;
        let assignment = self.required_artifact(id, "ASSIGNMENT")?;
        let card = self.required_artifact(id, "OPPORTUNITY_CARD")?;
        let payload = producers::compile_brief(&self.stamped(&assignment), &self.stamped(&card), revision);
        let generation = self.db.revision_generation(id)?;
        let prior = self.db.latest_artifact(id, "CONTENT_BRIEF")?;
        let brief = self.db.insert_artifact(id, "CONTENT_BRIEF", generation, "adam", prior.as_ref().map(|a| a.artifact_id.as_str()), payload)?;
        self.db.acknowledge_handoffs(id, "adam")?;
        self.record(id, cap, None, None, true, None)?;
        self.db.event(Some(id), "brief.compiled", "adam", json!({"brief_id": brief.artifact_id}))?;
        self.db.update_work_item(id, "STRATEGIZED", "brain + jax", None)?;
        self.end_role(cap)?;
        Ok(true)
    }

    fn begin_production(&self, id: &str) -> Result<bool, String> {
        let brief = self.required_artifact(id, "CONTENT_BRIEF")?;
        self.db.create_handoff(&format!("{id}:brief:brain:{}", brief.version), id, "adam", "brain", &brief.artifact_id)?;
        self.db.create_handoff(&format!("{id}:brief:jax:{}", brief.version), id, "adam", "jax", &brief.artifact_id)?;
        self.db.event(Some(id), "jax.proof_planning_eligible", "system", json!({"brief_id": brief.artifact_id, "rule": "Jax may plan from brief; this slice emits NO_VISUAL_REQUIRED"}))?;
        self.db.update_work_item(id, "IN_PRODUCTION", "brain + jax", None)?;
        Ok(true)
    }

    async fn complete_production(&self, id: &str, test_mode: bool, revision: Option<&Value>) -> Result<bool, String> {
        let brief = self.required_artifact(id, "CONTENT_BRIEF")?;
        let draft_payload = self.brain_write(id, &self.stamped(&brief), revision, test_mode).await?;
        let generation = self.db.revision_generation(id)?;
        let prior_draft = self.db.latest_artifact(id, "CONTENT_DRAFT")?;
        let draft = self.db.insert_artifact(id, "CONTENT_DRAFT", generation, "brain", prior_draft.as_ref().map(|a| a.artifact_id.as_str()), draft_payload)?;
        self.db.acknowledge_handoffs(id, "brain")?;
        self.db.create_handoff(&format!("{id}:draft:jax:{}", draft.version), id, "brain", "jax", &draft.artifact_id)?;

        let jax_cap = capabilities::DECIDE_CREATIVE;
        self.begin_role(jax_cap)?;
        let visual_payload = producers::decide_creative(&self.stamped(&brief), &self.stamped(&draft));
        let prior_visual = self.db.latest_artifact(id, "NO_VISUAL_REQUIRED")?;
        let visual = self.db.insert_artifact(id, "NO_VISUAL_REQUIRED", generation, "jax", prior_visual.as_ref().map(|a| a.artifact_id.as_str()), visual_payload)?;
        self.db.acknowledge_handoffs(id, "jax")?;
        self.record(id, jax_cap, None, None, true, None)?;
        self.db.event(Some(id), "creative.no_visual", "jax", json!({"visual_id": visual.artifact_id, "draft_id": draft.artifact_id}))?;
        self.end_role(jax_cap)?;
        self.assemble(id, &brief, &draft, &visual)?;
        Ok(false)
    }

    async fn brain_write(&self, id: &str, brief: &Value, revision: Option<&Value>, test_mode: bool) -> Result<Value, String> {
        let cap = capabilities::WRITE_PUBLIC_COPY;
        let settings = self.db.settings()?;
        self.db.set_employee_status("brain", "WORKING")?;
        if settings.brain_force_fail {
            let error = crate::providers::interpret_ollama_body(&crate::providers::truncated_provider_error_body())
                .unwrap_err();
            self.record(id, cap, Some("OLLAMA_PROVIDER"), Some(&settings.ollama_model), false, Some(&error))?;
            self.db.set_employee_status("brain", "BLOCKED")?;
            self.db.event(Some(id), "model.failed", "brain", json!({"error": error, "forced": true, "fixture_recovery": false}))?;
            return Err(error);
        }
        if test_mode {
            let payload = producers::write_public_copy(brief, revision);
            self.db.record_execution(Some(id), "brain", cap.id, ExecutionKind::CapabilityRun.as_str(), None, None, true, None)?;
            self.db.event(Some(id), "capability.ran", "brain", json!({"capability": cap.id, "kind": "CAPABILITY_RUN", "test_double": true}))?;
            self.db.set_employee_status("brain", "IDLE")?;
            return Ok(payload);
        }
        match self.runner.generate_content_draft(id, brief, revision).await {
            Ok(payload) => {
                self.record(id, cap, Some("OLLAMA_PROVIDER"), Some(&settings.ollama_model), true, None)?;
                self.db.set_employee_status("brain", "IDLE")?;
                Ok(payload)
            }
            Err(error) => {
                self.record(id, cap, Some("OLLAMA_PROVIDER"), Some(&settings.ollama_model), false, Some(&error))?;
                Err(error)
            }
        }
    }

    async fn complete_revision(&self, id: &str, test_mode: bool) -> Result<bool, String> {
        let cap = capabilities::COMPILE_REVISION;
        let approval = self.db.active_approval(id)?.ok_or("Missing revision approval")?;
        let feedback = approval.feedback.clone().unwrap_or_default();
        let target = policy::revision_target(Some(&feedback));
        let instruction_payload = producers::revision_instruction(&feedback, target);
        let generation = self.db.revision_generation(id)?;
        let instruction = self.db.insert_artifact(id, "REVISION_INSTRUCTION", generation, "system", None, instruction_payload)?;
        self.record(id, cap, None, None, true, None)?;
        let stamped_instruction = self.stamped(&instruction);
        if target == "adam" {
            self.adam_brief(id, Some(&stamped_instruction))?;
            self.begin_production(id)?;
        } else {
            self.db.update_work_item(id, "IN_PRODUCTION", "brain + jax", None)?;
        }
        self.complete_production(id, test_mode, Some(&stamped_instruction)).await
    }

    fn assemble(&self, id: &str, brief: &ArtifactRecord, draft: &ArtifactRecord, visual: &ArtifactRecord) -> Result<ArtifactRecord, String> {
        if brief.work_item_id != id || draft.work_item_id != id || visual.work_item_id != id {
            return Err("Package components do not belong to the same work item".into());
        }
        let cap = capabilities::ASSEMBLE_PACKAGE;
        let generation = self.db.revision_generation(id)?;
        let prior = self.db.latest_artifact(id, "PUBLISH_PACKAGE")?;
        let payload = producers::publish_package(id, &self.stamped(brief), &self.stamped(draft), &self.stamped(visual), EXECUTION_TARGET);
        let package = self.db.insert_artifact(id, "PUBLISH_PACKAGE", generation, "system", prior.as_ref().map(|a| a.artifact_id.as_str()), payload)?;
        self.db.create_handoff(&format!("{id}:package:sam:{generation}"), id, "system", "sam", &package.artifact_id)?;
        self.record(id, cap, None, None, true, None)?;
        self.db.event(Some(id), "package.assembled", "system", json!({"package_id": package.artifact_id, "draft_id": draft.artifact_id, "visual_id": visual.artifact_id, "execution_target": EXECUTION_TARGET}))?;
        self.db.update_work_item(id, "READY_FOR_APPROVAL", "sam", None)?;
        Ok(package)
    }

    pub async fn approval(&self, decision: ApprovalDecision, test_mode: bool) -> Result<WorkItemDetail, String> {
        let item = self.db.work_item(&decision.work_item_id)?;
        if item.state != "READY_FOR_APPROVAL" {
            return Err(format!("Approval action is invalid from {}", item.state));
        }
        let content = self.required_artifact(&decision.work_item_id, "CONTENT_DRAFT")?;
        let visual = self.required_visual(&decision.work_item_id)?;
        let package = self.required_artifact(&decision.work_item_id, "PUBLISH_PACKAGE")?;
        let cap = capabilities::RECORD_APPROVAL;
        match decision.action.as_str() {
            "APPROVE" => {
                self.db.create_approval(&decision.work_item_id, &content, &visual, &package, "APPROVED", decision.feedback.as_deref())?;
                self.record(&decision.work_item_id, cap, None, None, true, None)?;
                self.db.update_work_item(&decision.work_item_id, "APPROVED", "maro", None)?;
                self.db.event(Some(&decision.work_item_id), "approval.granted", "sam", json!({"package_id": package.artifact_id, "package_version": package.version}))?;
                self.advance_until_blocked(&decision.work_item_id, test_mode).await
            }
            "REQUEST_REVISION" => {
                self.db.create_approval(&decision.work_item_id, &content, &visual, &package, "REQUEST_REVISION", decision.feedback.as_deref())?;
                self.record(&decision.work_item_id, cap, None, None, true, None)?;
                self.db.update_work_item(&decision.work_item_id, "REVISION_REQUIRED", "brain + jax", None)?;
                self.db.event(Some(&decision.work_item_id), "approval.revision_requested", "sam", json!({"feedback": decision.feedback}))?;
                self.db.detail(&decision.work_item_id)
            }
            "REJECT" => {
                self.db.create_approval(&decision.work_item_id, &content, &visual, &package, "REJECTED", decision.feedback.as_deref())?;
                self.record(&decision.work_item_id, cap, None, None, true, None)?;
                self.db.update_work_item(&decision.work_item_id, "REJECTED", "sam", None)?;
                self.db.event(Some(&decision.work_item_id), "approval.rejected", "sam", json!({"feedback": decision.feedback}))?;
                self.db.detail(&decision.work_item_id)
            }
            _ => Err("Action must be APPROVE, REQUEST_REVISION, or REJECT".into()),
        }
    }

    async fn publish(&self, id: &str, test_mode: bool) -> Result<bool, String> {
        let _ = test_mode;
        let content = self.required_artifact(id, "CONTENT_DRAFT")?;
        let visual = self.required_visual(id)?;
        let package = self.required_artifact(id, "PUBLISH_PACKAGE")?;
        let approval = self.db.active_approval(id)?;
        if let Err(error) = policy::assert_publish_allowed(approval.as_ref(), &content, &visual, &package) {
            self.db.record_execution(Some(id), "maro", capabilities::VALIDATE_AND_PUBLISH.id, ExecutionKind::PolicyCheck.as_str(), None, None, false, Some(error.as_str()))?;
            self.db.event(Some(id), "policy.checked", "maro", json!({"capability": "validate_and_publish", "success": false, "error": error.as_str()}))?;
            return Err(error.into());
        }
        self.db.record_execution(Some(id), "maro", capabilities::VALIDATE_AND_PUBLISH.id, ExecutionKind::PolicyCheck.as_str(), None, None, true, None)?;
        self.db.event(Some(id), "policy.checked", "maro", json!({"capability": "validate_and_publish", "kind": "POLICY_CHECK", "success": true}))?;

        let settings = self.db.settings()?;
        if settings.publish_force_fail {
            let error = "LOCAL_LEDGER adapter transient failure";
            self.db.record_execution(Some(id), "maro", capabilities::VALIDATE_AND_PUBLISH.id, ExecutionKind::AdapterRun.as_str(), Some("LOCAL_LEDGER"), None, false, Some(error))?;
            return Err(error.into());
        }

        let approval = approval.ok_or("Missing approval")?;
        let key = local_publish::idempotency_key(id, &package.artifact_id, package.version);
        let receipt_payload = local_publish::receipt(id, &package.artifact_id, &approval.approval_id, &content.artifact_id, &visual.artifact_id, &key);
        let generation = self.db.revision_generation(id)?;
        let receipt = self.db.insert_artifact(id, "PUBLICATION_RECEIPT", generation, "maro", None, receipt_payload)?;
        let result_payload = local_publish::distribution_result(id, &receipt.artifact_id, &key);
        let result = self.db.insert_artifact(id, "DISTRIBUTION_RESULT", generation, "maro", None, result_payload)?;
        let publication_id = self.db.create_publication_with_ref(id, &receipt.artifact_id, EXECUTION_TARGET, &format!("ledger://local/{key}"))?;
        self.db.create_handoff(&format!("{id}:publication:lara:1"), id, "maro", "lara", &receipt.artifact_id)?;
        self.db.record_execution(Some(id), "maro", capabilities::VALIDATE_AND_PUBLISH.id, ExecutionKind::AdapterRun.as_str(), Some("LOCAL_LEDGER"), None, true, None)?;
        self.db.event(Some(id), "adapter.ran", "maro", json!({
            "publication_id": publication_id,
            "receipt_id": receipt.artifact_id,
            "distribution_result_id": result.artifact_id,
            "intended_platforms": ["LINKEDIN"],
            "execution_target": EXECUTION_TARGET,
            "external_api": false
        }))?;
        self.db.update_work_item(id, "PUBLISHED", "system", None)?;
        Ok(true)
    }

    fn snapshot(&self, id: &str, test_mode: bool) -> Result<bool, String> {
        let cap = capabilities::CAPTURE_SNAPSHOT;
        let receipt = self.required_artifact(id, "PUBLICATION_RECEIPT")?;
        let settings = self.db.settings()?;
        let quality = if test_mode && settings.observe_quality == "COMPLETE" {
            "COMPLETE"
        } else {
            "UNAVAILABLE"
        };
        let payload = local_observe::snapshot(&receipt.artifact_id, quality);
        if quality == "COMPLETE" && payload.get("mocked") != Some(&json!(true)) {
            return Err("Synthetic COMPLETE observation must be marked mocked: true".into());
        }
        let artifact = self.db.insert_artifact(id, "PERFORMANCE_SNAPSHOT", 1, "system", None, payload.clone())?;
        let publication_id = format!("publication-for-{}", receipt.artifact_id);
        self.db.create_snapshot(id, &publication_id, &payload)?;
        self.db.create_handoff(&format!("{id}:snapshot:lara:1"), id, "system", "lara", &artifact.artifact_id)?;
        self.record(id, cap, Some("LOCAL_LEDGER"), None, true, None)?;
        self.db.event(Some(id), "performance_snapshot.imported", "system", json!({"snapshot_id": artifact.artifact_id, "data_quality": quality, "mocked": payload.get("mocked")}))?;
        self.db.update_work_item(id, "MEASURING", "lara", None)?;
        Ok(true)
    }

    fn measure_and_learn(&self, id: &str) -> Result<bool, String> {
        let lara_cap = capabilities::INTERPRET_PERFORMANCE;
        self.begin_role(lara_cap)?;
        let snapshot = self.required_artifact(id, "PERFORMANCE_SNAPSHOT")?;
        let draft = self.required_artifact(id, "CONTENT_DRAFT")?;
        let receipt = self.required_artifact(id, "PUBLICATION_RECEIPT")?;
        let insight_payload = producers::interpret_performance(&self.stamped(&snapshot), &self.stamped(&draft), &self.stamped(&receipt));
        let insight = self.db.insert_artifact(id, "PERFORMANCE_INSIGHT", 1, "lara", None, insight_payload)?;
        self.db.acknowledge_handoffs(id, "lara")?;
        self.db.create_handoff(&format!("{id}:insight:travis:1"), id, "lara", "travis", &insight.artifact_id)?;
        self.record(id, lara_cap, None, None, true, None)?;
        self.end_role(lara_cap)?;

        let travis_cap = capabilities::DECIDE_NEXT_CYCLE;
        self.begin_role(travis_cap)?;
        let card = self.required_artifact(id, "OPPORTUNITY_CARD")?;
        let cycle_payload = producers::decide_next_cycle(&self.stamped(&insight), &self.stamped(&card));
        let cycle = self.db.insert_artifact(id, "CYCLE_DECISION", 1, "travis", None, cycle_payload.clone())?;
        self.db.acknowledge_handoffs(id, "travis")?;
        let signal = self.research_signal(id)?;
        self.persist_learning(id, &signal, &cycle_payload, Some(&self.stamped(&insight)), &[&cycle.artifact_id, &insight.artifact_id])?;
        self.record(id, travis_cap, None, None, true, None)?;
        self.db.event(Some(id), "cycle.completed", "travis", json!({"cycle_decision_id": cycle.artifact_id}))?;
        self.db.update_work_item(id, "MEASURED", "travis", None)?;
        self.end_role(travis_cap)?;
        Ok(false)
    }

    fn persist_learning(&self, id: &str, signal: &str, cycle: &Value, insight: Option<&Value>, refs: &[&str]) -> Result<LearningRecord, String> {
        let mut payload = producers::learning_record(id, signal, cycle, insight, refs);
        let generation = self.db.revision_generation(id)?;
        let artifact = self.db.insert_artifact(id, "LEARNING_RECORD", generation, "system", None, payload.clone())?;
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("artifact_id".into(), json!(artifact.artifact_id));
        }
        let record = self.db.insert_learning(id, payload)?;
        self.db.event(Some(id), "learning.recorded", "travis", json!({"learning_id": record.learning_id, "cycle_decision": record.cycle_decision, "fingerprint": record.opportunity_fingerprint}))?;
        Ok(record)
    }

    fn assert_valid_approval(&self, id: &str) -> Result<(), String> {
        let a = self.db.active_approval(id)?.ok_or("Maro blocked: explicit Sam approval is missing")?;
        let c = self.required_artifact(id, "CONTENT_DRAFT")?;
        let j = self.required_visual(id)?;
        let p = self.required_artifact(id, "PUBLISH_PACKAGE")?;
        policy::assert_publish_allowed(Some(&a), &c, &j, &p).map_err(Into::into)
    }

    fn required_artifact(&self, id: &str, kind: &str) -> Result<ArtifactRecord, String> {
        self.db.latest_artifact(id, kind)?.ok_or_else(|| format!("Missing required {kind}"))
    }

    fn required_visual(&self, id: &str) -> Result<ArtifactRecord, String> {
        if let Some(visual) = self.db.latest_artifact(id, "NO_VISUAL_REQUIRED")? {
            return Ok(visual);
        }
        self.required_artifact(id, "CREATIVE_PACKAGE")
    }

    fn resume_blocked(&self, id: &str) -> Result<bool, String> {
        let item = self.db.work_item(id)?;
        let reason = item.blocked_reason.clone().unwrap_or_default();
        if is_policy_block(&reason) {
            return Ok(false);
        }
        let resume = match item.resume_state.as_deref() {
            Some(state) if Database::is_valid_resume_state(state) => state.to_string(),
            _ => match infer_resume(&reason) {
                Some(inferred) => {
                    let stored = self.db.enter_blocked(id, &item.current_owner, &reason, inferred)?;
                    self.db.event(Some(id), "workflow.resume_state.repaired", "system", json!({
                        "inferred": stored,
                        "reason": reason
                    }))?;
                    stored
                }
                None => return Ok(false),
            },
        };
        let owner = owner_for(&resume);
        self.db.event(Some(id), "workflow.resumed", "system", json!({"resume_state": resume, "reason": reason}))?;
        self.db.update_work_item(id, &resume, owner, None)?;
        Ok(true)
    }

    fn stamped(&self, record: &ArtifactRecord) -> Value {
        producers::stamp(&record.artifact_id, record.version, record.payload.clone())
    }

    fn research_signal(&self, id: &str) -> Result<String, String> {
        let signal = self.required_artifact(id, "RESEARCH_SIGNAL")?;
        Ok(producers::signal_text(&signal.payload))
    }

    fn record(&self, id: &str, cap: Capability, provider: Option<&str>, model: Option<&str>, success: bool, error: Option<&str>) -> Result<(), String> {
        self.db.record_execution(Some(id), cap.owner_role, cap.id, cap.kind.as_str(), provider, model, success, error)?;
        let event_type = match cap.kind {
            ExecutionKind::ModelRun => "model.ran",
            ExecutionKind::CapabilityRun => "capability.ran",
            ExecutionKind::AdapterRun => "adapter.ran",
            ExecutionKind::PolicyCheck => "policy.checked",
        };
        self.db.event(Some(id), event_type, cap.owner_role, json!({"capability": cap.id, "kind": cap.kind.as_str(), "success": success, "error": error}))?;
        Ok(())
    }

    fn begin_role(&self, cap: Capability) -> Result<(), String> {
        if cap.owner_role != "system" && cap.owner_role != "sam" {
            self.db.set_employee_status(cap.owner_role, "WORKING")?;
        }
        Ok(())
    }

    fn end_role(&self, cap: Capability) -> Result<(), String> {
        if cap.owner_role != "system" && cap.owner_role != "sam" {
            self.db.set_employee_status(cap.owner_role, "IDLE")?;
        }
        Ok(())
    }

    pub fn control(&self, request: ControlRequest) -> Result<RuntimeSettings, String> {
        let value = match request.action.as_str() {
            "RUN" | "RESUME" => "RUNNING",
            "PAUSE" => "PAUSED",
            "STOP" => "STOPPED",
            _ => return Err("Control action must be RUN, PAUSE, RESUME, or STOP".into()),
        };
        self.db.set_control(value)?;
        self.db.event(None, "company.control", "sam", json!({"state": value}))?;
        self.db.settings()
    }
}

fn resume_target(state: &str) -> Option<&'static str> {
    match state {
        "IN_PRODUCTION" | "STRATEGIZED" | "REVISION_REQUIRED" => Some("STRATEGIZED"),
        "APPROVED" | "READY_TO_PUBLISH" => Some("READY_TO_PUBLISH"),
        "PUBLISHED" | "MEASURING" => Some("MEASURING"),
        "SELECTED" => Some("SELECTED"),
        "RESEARCHED" => Some("RESEARCHED"),
        _ => None,
    }
}

fn infer_resume(reason: &str) -> Option<&'static str> {
    let reason = reason.to_lowercase();
    if reason.contains("json") || reason.contains("ollama") || reason.contains("provider") || reason.contains("brain") {
        return Some("STRATEGIZED");
    }
    if reason.contains("adapter") || reason.contains("local_ledger") || reason.contains("approval") || reason.contains("maro") {
        return Some("READY_TO_PUBLISH");
    }
    None
}

fn owner_for(state: &str) -> &'static str {
    match state {
        "NEW" => "saly",
        "RESEARCHED" => "travis",
        "SELECTED" => "adam",
        "STRATEGIZED" | "IN_PRODUCTION" | "REVISION_REQUIRED" => "brain + jax",
        "READY_FOR_APPROVAL" => "sam",
        "APPROVED" | "READY_TO_PUBLISH" => "maro",
        "PUBLISHED" => "system",
        "MEASURING" => "lara",
        "MEASURED" | "REJECTED" => "travis",
        _ => "system",
    }
}

fn is_policy_block(reason: &str) -> bool {
    reason.contains("approval") || reason.contains("Maro blocked") || reason.contains("Policy blocked")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::{repository_root, DefinitionStore};
    use crate::schema_fixture::generate_for_type;

    fn runtime() -> (tempfile::TempDir, RuntimeService) {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::new(dir.path().join("test.sqlite")).unwrap();
        let defs = DefinitionStore::load(repository_root().unwrap()).unwrap();
        (dir, RuntimeService::new(db, defs).unwrap())
    }

    fn start(service: &RuntimeService, signal: &str) -> WorkItemSummary {
        service.start(StartWorkItemRequest { title: "Practical AI signal".into(), research_signal: signal.into() }).unwrap()
    }

    async fn to_approval(service: &RuntimeService, signal: &str) -> (WorkItemSummary, WorkItemDetail) {
        let work = start(service, signal);
        let waiting = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(waiting.work_item.state, "READY_FOR_APPROVAL", "{:?}", waiting.work_item.blocked_reason);
        (work, waiting)
    }

    fn artifact<'a>(detail: &'a WorkItemDetail, kind: &str) -> &'a ArtifactRecord {
        detail.artifacts.iter().rev().find(|a| a.artifact_type == kind).unwrap_or_else(|| panic!("missing {kind}"))
    }

    fn latest_payload<'a>(detail: &'a WorkItemDetail, kind: &str) -> &'a Value {
        &artifact(detail, kind).payload
    }

    fn assert_blocked_invariant(item: &WorkItemSummary) {
        assert_eq!(item.state, "BLOCKED");
        let resume = item.resume_state.as_deref().expect("BLOCKED must persist resume_state");
        assert!(Database::is_valid_resume_state(resume), "invalid resume_state {resume}");
        assert_ne!(resume, "NEW");
        assert_ne!(resume, "BLOCKED");
    }

    #[tokio::test]
    async fn complete_cycle_stops_for_version_bound_approval_and_finishes() {
        let (_dir, service) = runtime();
        let (work, waiting) = to_approval(&service, "Teams need reliable AI workflows").await;
        let draft = artifact(&waiting, "CONTENT_DRAFT");
        let visual = artifact(&waiting, "NO_VISUAL_REQUIRED");
        let package = artifact(&waiting, "PUBLISH_PACKAGE");
        assert_eq!(package.payload.pointer("/content_draft/artifact_id").and_then(Value::as_str), Some(draft.artifact_id.as_str()));
        assert_eq!(package.payload.pointer("/no_visual_decision/artifact_id").and_then(Value::as_str), Some(visual.artifact_id.as_str()));
        assert_eq!(package.payload.pointer("/no_visual_decision/compatible_draft_id").and_then(Value::as_str), Some(draft.artifact_id.as_str()));
        assert_eq!(package.payload.get("intended_platforms").and_then(Value::as_array).and_then(|a| a[0].as_str()), Some("LINKEDIN"));
        assert_eq!(package.payload.get("execution_target").and_then(Value::as_str), Some("LOCAL_LEDGER"));
        assert!(waiting.events.iter().any(|e| e.event_type == "jax.proof_planning_eligible"));
        assert!(waiting.handoffs.iter().any(|h| h.sender == "adam" && h.receiver == "brain"));
        assert!(waiting.handoffs.iter().any(|h| h.sender == "adam" && h.receiver == "jax"));
        let done = service.approval(ApprovalDecision { work_item_id: work.work_item_id, action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        assert_eq!(done.work_item.state, "MEASURED", "{:?}", done.work_item.blocked_reason);
        for required in ["PUBLICATION_RECEIPT", "DISTRIBUTION_RESULT", "PERFORMANCE_SNAPSHOT", "PERFORMANCE_INSIGHT", "CYCLE_DECISION", "LEARNING_RECORD", "NO_VISUAL_REQUIRED"] {
            assert!(done.artifacts.iter().any(|a| a.artifact_type == required), "missing {required}");
        }
    }

    #[tokio::test]
    async fn maro_is_blocked_without_approval() {
        let (_dir, service) = runtime();
        let work = start(&service, "y");
        service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        service.db.update_work_item(&work.work_item_id, "READY_TO_PUBLISH", "maro", None).unwrap();
        let detail = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(detail.work_item.state, "BLOCKED");
        assert_blocked_invariant(&detail.work_item);
        assert_eq!(detail.work_item.resume_state.as_deref(), Some("READY_TO_PUBLISH"));
        assert!(detail.work_item.blocked_reason.unwrap().contains("approval"));
        let again = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(again.work_item.state, "BLOCKED", "policy BLOCKED must not auto-resume");
    }

    #[tokio::test]
    async fn pause_and_restart_preserve_state() {
        let (dir, service) = runtime();
        let work = start(&service, "y");
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
        let work = start(&service, "y");
        service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        let content = service.required_artifact(&work.work_item_id, "CONTENT_DRAFT").unwrap();
        let visual = service.required_visual(&work.work_item_id).unwrap();
        let package = service.required_artifact(&work.work_item_id, "PUBLISH_PACKAGE").unwrap();
        service.db.create_approval(&work.work_item_id, &content, &visual, &package, "APPROVED", None).unwrap();
        service.db.insert_artifact(&work.work_item_id, "CONTENT_DRAFT", 2, "brain", Some(&content.artifact_id), json!({"changed": true})).unwrap();
        assert!(service.assert_valid_approval(&work.work_item_id).unwrap_err().contains("approval"));
    }

    #[tokio::test]
    async fn signal_survives_the_chain() {
        let (_dir, service) = runtime();
        let signal = "UNIQUE_SIGNAL_ALPHA_WORKFLOW_GATE";
        let (_work, waiting) = to_approval(&service, signal).await;
        let card = latest_payload(&waiting, "OPPORTUNITY_CARD");
        let assignment = latest_payload(&waiting, "ASSIGNMENT");
        let brief = latest_payload(&waiting, "CONTENT_BRIEF");
        let draft = latest_payload(&waiting, "CONTENT_DRAFT");
        assert_eq!(card.get("core_idea").and_then(Value::as_str), Some(signal));
        assert!(assignment.get("objective").and_then(Value::as_str).unwrap_or("").contains(signal));
        assert!(brief.get("core_idea").and_then(Value::as_str).unwrap_or("").contains(signal));
        assert!(draft.get("body").and_then(Value::as_str).unwrap_or("").contains(signal));
        assert!(draft.get("core_idea").and_then(Value::as_str).unwrap_or("").contains(signal));
    }

    #[tokio::test]
    async fn assignment_binds_opportunity() {
        let (_dir, service) = runtime();
        let (_work, waiting) = to_approval(&service, "Bind this opportunity to Travis assignment").await;
        let card = artifact(&waiting, "OPPORTUNITY_CARD");
        let assignment = latest_payload(&waiting, "ASSIGNMENT");
        let card_id = card.payload.get("opportunity_id").and_then(Value::as_str).unwrap();
        assert_eq!(assignment.get("source_artifact_reference").and_then(Value::as_str), Some(card_id));
        assert_eq!(
            assignment.get("opportunity_fingerprint"),
            card.payload.get("opportunity_fingerprint")
        );
        assert_eq!(assignment.get("recipient").and_then(Value::as_str), Some("adam"));
        assert_eq!(assignment.get("created_by").and_then(Value::as_str), Some("travis"));
    }

    #[tokio::test]
    async fn travis_is_not_fixture_assignment() {
        let (_dir, service) = runtime();
        let signal = "Not a schema fixture assignment objective";
        let (_work, waiting) = to_approval(&service, signal).await;
        let assignment = latest_payload(&waiting, "ASSIGNMENT");
        let fixture = generate_for_type(&service.definitions.get("travis").unwrap().output_schema, "ASSIGNMENT");
        assert_ne!(assignment.get("objective"), fixture.get("objective"));
        assert!(assignment.get("opportunity_fingerprint").and_then(Value::as_str).is_some());
        assert!(assignment.get("objective").and_then(Value::as_str).unwrap_or("").contains(signal));
    }

    #[tokio::test]
    async fn jax_owns_no_visual() {
        let (_dir, service) = runtime();
        let (_work, waiting) = to_approval(&service, "Text only LinkedIn treatment").await;
        let visual = artifact(&waiting, "NO_VISUAL_REQUIRED");
        assert_eq!(visual.producer, "jax");
        assert_eq!(visual.payload.get("visual_decision").and_then(Value::as_str), Some("NO_VISUAL_REQUIRED"));
        assert!(waiting.artifacts.iter().all(|a| a.artifact_type != "CREATIVE_PACKAGE"));
        assert!(waiting.handoffs.iter().any(|h| h.sender == "jax" || (h.sender == "adam" && h.receiver == "jax")));
    }

    #[tokio::test]
    async fn maro_zero_llm() {
        let (_dir, service) = runtime();
        let (work, _) = to_approval(&service, "Maro never talks to a model").await;
        let done = service.approval(ApprovalDecision { work_item_id: work.work_item_id.clone(), action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        assert!(done.runs.iter().all(|r| r.agent != "maro"));
        let maro = done.executions.iter().filter(|e| e.role == "maro").collect::<Vec<_>>();
        assert!(maro.iter().any(|e| e.kind == "POLICY_CHECK"));
        assert!(maro.iter().any(|e| e.kind == "ADAPTER_RUN"));
        assert!(maro.iter().all(|e| e.kind != "MODEL_RUN"));
        assert!(!capabilities::VALIDATE_AND_PUBLISH.may_use_model);
    }

    #[tokio::test]
    async fn execute_writes_receipt() {
        let (_dir, service) = runtime();
        let (work, _) = to_approval(&service, "Ledger receipt must exist").await;
        let done = service.approval(ApprovalDecision { work_item_id: work.work_item_id, action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        let receipt = latest_payload(&done, "PUBLICATION_RECEIPT");
        assert_eq!(receipt.get("execution_target").and_then(Value::as_str), Some("LOCAL_LEDGER"));
        assert_eq!(receipt.get("intended_platforms").and_then(|v| v.get(0)).and_then(Value::as_str), Some("LINKEDIN"));
        assert_eq!(receipt.get("confirmation_basis").and_then(Value::as_str), Some("LOCAL_LEDGER_WRITE"));
        assert_eq!(receipt.get("external_api"), Some(&json!(false)));
        assert!(receipt.get("idempotency_key").and_then(Value::as_str).is_some());
    }

    #[tokio::test]
    async fn observe_unavailable_is_honest() {
        let (_dir, service) = runtime();
        let (work, _) = to_approval(&service, "Do not invent metrics").await;
        let done = service.approval(ApprovalDecision { work_item_id: work.work_item_id, action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        let snapshot = latest_payload(&done, "PERFORMANCE_SNAPSHOT");
        assert_eq!(snapshot.get("data_quality").and_then(Value::as_str), Some("UNAVAILABLE"));
        assert_ne!(snapshot.get("mocked"), Some(&json!(true)));
        assert!(snapshot.get("metrics").and_then(Value::as_object).map(|m| m.is_empty()).unwrap_or(true));
    }

    #[tokio::test]
    async fn lara_no_causality_when_unavailable() {
        let (_dir, service) = runtime();
        let (work, _) = to_approval(&service, "Lara must not invent causality").await;
        let done = service.approval(ApprovalDecision { work_item_id: work.work_item_id, action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        let insight = latest_payload(&done, "PERFORMANCE_INSIGHT");
        assert_eq!(insight.get("data_quality").and_then(Value::as_str), Some("UNAVAILABLE"));
        assert_eq!(insight.get("recommended_attention").and_then(Value::as_str), Some("NEED_MORE_DATA"));
        let cannot = insight.pointer("/payload/extensions/lara_analysis/what_cannot_be_concluded").and_then(Value::as_array).cloned().unwrap_or_default();
        assert!(cannot.iter().any(|v| v.as_str() == Some("Causality")));
    }

    #[tokio::test]
    async fn learning_crosses_work_items() {
        let (_dir, service) = runtime();
        let signal = "Cross item learning fingerprint signal";
        let (first, _) = to_approval(&service, signal).await;
        service.approval(ApprovalDecision { work_item_id: first.work_item_id.clone(), action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        let second = start(&service, signal);
        let waiting = service.advance_until_blocked(&second.work_item_id, true).await.unwrap();
        let context = artifact(&waiting, "LEARNING_CONTEXT");
        let records = context.payload.get("records").and_then(Value::as_array).cloned().unwrap_or_default();
        assert!(!records.is_empty(), "second item must bind prior learning");
        let fp = learning::fingerprint(signal);
        assert!(records.iter().any(|r| r.get("opportunity_fingerprint").and_then(Value::as_str) == Some(fp.as_str())));
        assert_eq!(waiting.work_item.state, "READY_FOR_APPROVAL");
    }

    #[tokio::test]
    async fn travis_reject_persists_negative_learning() {
        let (_dir, service) = runtime();
        let signal = "Rejected duplicate opportunity must be remembered";
        let fp = learning::fingerprint(signal);
        service.db.insert_learning("seed-item", json!({
            "opportunity_fingerprint": fp,
            "cycle_decision": "STOP",
            "data_quality": "UNAVAILABLE",
            "recommended_attention": "NEGATIVE_SIGNAL",
            "constraints_for_next": {"avoid_angles": [signal], "prefer_angles": [], "require_more_evidence": false, "do_not_treat_as_causal": true}
        })).unwrap();
        let work = start(&service, signal);
        let detail = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(detail.work_item.state, "REJECTED");
        assert!(detail.artifacts.iter().any(|a| a.artifact_type == "GROWTH_DECISION" && a.payload.get("decision").and_then(Value::as_str) == Some("reject")));
        assert!(detail.artifacts.iter().all(|a| a.artifact_type != "ASSIGNMENT"));
        let learning_art = artifact(&detail, "LEARNING_RECORD");
        assert_eq!(learning_art.payload.get("cycle_decision").and_then(Value::as_str), Some("STOP"));
        assert_eq!(learning_art.payload.get("opportunity_fingerprint").and_then(Value::as_str), Some(fp.as_str()));
        let stored = service.db.recent_learning(10).unwrap();
        assert!(stored.iter().any(|r| r.source_work_item_id == work.work_item_id && r.cycle_decision == "STOP"));
    }

    #[tokio::test]
    async fn brain_invalid_json_blocks() {
        let (_dir, service) = runtime();
        service.db.set_setting("brain_force_fail", "true").unwrap();
        let work = start(&service, "Brain schema failure should block at STRATEGIZED");
        let detail = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(detail.work_item.state, "BLOCKED");
        assert_blocked_invariant(&detail.work_item);
        assert_eq!(detail.work_item.resume_state.as_deref(), Some("STRATEGIZED"));
        assert!(detail.work_item.blocked_reason.as_deref().unwrap_or("").contains("JSON"));
        service.db.set_setting("brain_force_fail", "false").unwrap();
        let resumed = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(resumed.work_item.state, "READY_FOR_APPROVAL", "{:?}", resumed.work_item.blocked_reason);
        assert_ne!(resumed.work_item.state, "NEW");
    }

    #[tokio::test]
    async fn revision_feedback_reaches_brain() {
        let (_dir, service) = runtime();
        let (work, _) = to_approval(&service, "Original hook about approval gates").await;
        let waiting = service.approval(ApprovalDecision {
            work_item_id: work.work_item_id.clone(),
            action: "REQUEST_REVISION".into(),
            feedback: Some("tighten the hook".into()),
        }, true).await.unwrap();
        assert_eq!(waiting.work_item.state, "REVISION_REQUIRED");
        let revised = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(revised.work_item.state, "READY_FOR_APPROVAL");
        let draft = artifact(&revised, "CONTENT_DRAFT");
        assert!(draft.payload.get("body").and_then(Value::as_str).unwrap_or("").contains("tighten the hook"));
        assert!(draft.version >= 2);
        assert!(revised.artifacts.iter().any(|a| a.artifact_type == "REVISION_INSTRUCTION"));
    }

    #[tokio::test]
    async fn approval_gate_unchanged() {
        let (_dir, service) = runtime();
        let (work, waiting) = to_approval(&service, "Sam remains the only public-execute authority").await;
        assert_eq!(waiting.work_item.state, "READY_FOR_APPROVAL");
        assert_eq!(waiting.work_item.current_owner, "sam");
        let rejected = service.approval(ApprovalDecision { work_item_id: work.work_item_id.clone(), action: "REJECT".into(), feedback: Some("off brand".into()) }, true).await.unwrap();
        assert_eq!(rejected.work_item.state, "REJECTED");
        assert!(rejected.artifacts.iter().all(|a| a.artifact_type != "PUBLICATION_RECEIPT"));
    }

    #[tokio::test]
    async fn schema_fixture_not_used_as_success() {
        let (_dir, service) = runtime();
        let (work, waiting) = to_approval(&service, "Fixture recovery is banned on the success path").await;
        assert!(waiting.events.iter().all(|e| e.event_type != "agent.output_schema_recovered"));
        assert!(waiting.runs.iter().all(|r| r.provider != "MOCK_PROVIDER"));
        let done = service.approval(ApprovalDecision { work_item_id: work.work_item_id, action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        assert!(done.events.iter().all(|e| e.event_type != "agent.output_schema_recovered"));
        assert!(done.runs.iter().all(|r| r.provider != "MOCK_PROVIDER"));
        let draft = latest_payload(&done, "CONTENT_DRAFT");
        assert!(draft.get("body").and_then(Value::as_str).unwrap_or("").contains("Fixture recovery is banned on the success path"));
    }

    #[tokio::test]
    async fn publish_adapter_failure_resumes_ready_to_publish() {
        let (_dir, service) = runtime();
        let (work, _) = to_approval(&service, "Adapter transient failure").await;
        service.db.set_setting("publish_force_fail", "true").unwrap();
        let blocked = service.approval(ApprovalDecision { work_item_id: work.work_item_id.clone(), action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        assert_eq!(blocked.work_item.state, "BLOCKED");
        assert_blocked_invariant(&blocked.work_item);
        assert_eq!(blocked.work_item.resume_state.as_deref(), Some("READY_TO_PUBLISH"));
        service.db.set_setting("publish_force_fail", "false").unwrap();
        let resumed = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(resumed.work_item.state, "MEASURED", "{:?}", resumed.work_item.blocked_reason);
    }

    #[tokio::test]
    async fn synthetic_complete_observe_is_mocked() {
        let (_dir, service) = runtime();
        service.db.set_setting("observe_quality", "COMPLETE").unwrap();
        let (work, _) = to_approval(&service, "Synthetic observe must be marked mocked").await;
        let done = service.approval(ApprovalDecision { work_item_id: work.work_item_id, action: "APPROVE".into(), feedback: None }, true).await.unwrap();
        let snapshot = latest_payload(&done, "PERFORMANCE_SNAPSHOT");
        assert_eq!(snapshot.get("data_quality").and_then(Value::as_str), Some("COMPLETE"));
        assert_eq!(snapshot.get("mocked"), Some(&json!(true)));
        assert_eq!(snapshot.get("provenance").and_then(Value::as_str), Some("LOCAL_ADAPTER_SYNTHETIC"));
    }

    #[tokio::test]
    async fn live_style_provider_json_error_resumes_strategized() {
        let (_dir, service) = runtime();
        service.db.set_setting("brain_force_fail", "true").unwrap();
        let work = start(&service, "Live provider JSON truncation must resume Brain only");
        let detail = service.advance_until_blocked(&work.work_item_id, false).await.unwrap();
        assert_blocked_invariant(&detail.work_item);
        assert_eq!(detail.work_item.resume_state.as_deref(), Some("STRATEGIZED"));
        let reason = detail.work_item.blocked_reason.clone().unwrap_or_default();
        assert!(reason.contains("truncated JSON"), "{reason}");
        assert!(reason.contains("done_reason=length"), "{reason}");
        assert!(reason.contains("eval_count=768"), "{reason}");
        assert_eq!(detail.artifacts.iter().filter(|a| a.artifact_type == "OPPORTUNITY_CARD").count(), 1);
        assert_eq!(detail.artifacts.iter().filter(|a| a.artifact_type == "CONTENT_BRIEF").count(), 1);
        assert!(detail.artifacts.iter().all(|a| a.artifact_type != "CONTENT_DRAFT"));

        service.db.set_setting("brain_force_fail", "false").unwrap();
        let resumed = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_eq!(resumed.work_item.state, "READY_FOR_APPROVAL", "{:?}", resumed.work_item.blocked_reason);
        assert_eq!(resumed.artifacts.iter().filter(|a| a.artifact_type == "OPPORTUNITY_CARD").count(), 1);
        assert_eq!(resumed.artifacts.iter().filter(|a| a.artifact_type == "ASSIGNMENT").count(), 1);
        assert_eq!(resumed.artifacts.iter().filter(|a| a.artifact_type == "CONTENT_BRIEF").count(), 1);
        assert!(resumed.artifacts.iter().any(|a| a.artifact_type == "CONTENT_DRAFT"));
    }

    #[tokio::test]
    async fn missing_resume_state_is_repaired_without_blocked_to_blocked() {
        let (dir, service) = runtime();
        service.db.set_setting("brain_force_fail", "true").unwrap();
        let work = start(&service, "Null resume_state must not become BLOCKED to BLOCKED");
        let blocked = service.advance_until_blocked(&work.work_item_id, false).await.unwrap();
        assert_blocked_invariant(&blocked.work_item);
        let path = dir.path().join("test.sqlite");
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute("UPDATE work_items SET resume_state=NULL WHERE work_item_id=?1", [&work.work_item_id]).unwrap();
        }
        let poisoned = service.db.work_item(&work.work_item_id).unwrap();
        assert_eq!(poisoned.state, "BLOCKED");
        assert!(poisoned.resume_state.is_none());

        service.db.set_setting("brain_force_fail", "false").unwrap();
        let resumed = service.advance_until_blocked(&work.work_item_id, true).await.unwrap();
        assert_ne!(resumed.work_item.state, "NEW");
        assert_ne!(resumed.work_item.resume_state.as_deref(), Some("BLOCKED"));
        assert_eq!(resumed.work_item.state, "READY_FOR_APPROVAL", "{:?}", resumed.work_item.blocked_reason);
        assert!(resumed.events.iter().any(|e| e.event_type == "workflow.resume_state.repaired" || e.event_type == "workflow.resumed"));
    }

    #[test]
    fn blocked_resume_state_invariant() {
        assert!(!Database::is_valid_resume_state("NEW"));
        assert!(!Database::is_valid_resume_state("BLOCKED"));
        assert!(!Database::is_valid_resume_state(""));
        assert!(!Database::is_valid_resume_state("CANCELLED"));
        assert!(Database::is_valid_resume_state("STRATEGIZED"));
        assert!(Database::is_valid_resume_state("READY_TO_PUBLISH"));
        assert_eq!(resume_target("IN_PRODUCTION"), Some("STRATEGIZED"));
        assert_eq!(resume_target("BLOCKED"), None);
        assert_eq!(resume_target("NEW"), None);
    }
}
