use serde_json::{json, Value};

use crate::{
    db::Database,
    definitions::DefinitionStore,
    providers::{AiProvider, OllamaProvider, OpenAiProvider, ProviderRequest, BRAIN_MAX_OUTPUT_TOKENS},
    router::{route, ProviderChoice},
};

/// Brain-only model runner. Deterministic roles never enter this path.
/// Schema-fixture recovery is forbidden on the success path.
#[derive(Clone)]
pub struct AgentRunner {
    db: Database,
    definitions: DefinitionStore,
}

impl AgentRunner {
    pub fn new(db: Database, definitions: DefinitionStore) -> Self {
        Self { db, definitions }
    }

    pub async fn generate_content_draft(
        &self,
        work_item_id: &str,
        brief: &Value,
        revision: Option<&Value>,
    ) -> Result<Value, String> {
        let settings = self.db.settings()?;
        let _ = self.definitions.get("brain")?;
        let openai = OpenAiProvider::from_env(&settings);
        let selected = route("public_writing", &settings, openai.is_some(), false);
        if matches!(selected.primary, ProviderChoice::Mock) {
            return Err("MOCK_PROVIDER cannot be used as a success path in the kernel slice".into());
        }
        let (provider_name, model) = match selected.primary {
            ProviderChoice::Ollama => ("OLLAMA_PROVIDER".to_string(), settings.ollama_model.clone()),
            ProviderChoice::OpenAi => ("OPENAI_PROVIDER".to_string(), settings.openai_model.clone()),
            ProviderChoice::Mock => unreachable!(),
        };
        let version = self.db.revision_generation(work_item_id)?;
        let logical_key = format!("{work_item_id}:brain:CONTENT_DRAFT:v{version}");
        let Some(run_id) = self.db.start_run(
            &logical_key,
            work_item_id,
            "brain",
            "public_writing",
            &provider_name,
            &model,
        )?
        else {
            return self
                .db
                .latest_artifact(work_item_id, "CONTENT_DRAFT")?
                .map(|a| a.payload)
                .ok_or_else(|| format!("Duplicate run {logical_key} has no persisted artifact"));
        };
        self.db.set_employee_status("brain", "WORKING")?;
        self.db.event(
            Some(work_item_id),
            "model.started",
            "brain",
            json!({
                "capability": "write_public_copy",
                "provider": provider_name,
                "expected_type": "CONTENT_DRAFT",
                "max_output_tokens": BRAIN_MAX_OUTPUT_TOKENS,
                "structured_output": true
            }),
        )?;

        let request = ProviderRequest {
            agent: "brain".into(),
            task_type: "public_writing".into(),
            expected_type: "CONTENT_DRAFT".into(),
            system_prompt: brain_system_prompt(),
            context: brain_context(brief, revision),
            output_schema: brain_output_schema(),
            output_template: Value::Null,
            schema_resources: Default::default(),
            max_output_tokens: BRAIN_MAX_OUTPUT_TOKENS,
        };

        let mut escalated = false;
        let mut response = match selected.primary {
            ProviderChoice::Ollama => OllamaProvider::new(&settings).generate(&request).await,
            ProviderChoice::OpenAi => openai.as_ref().ok_or("OpenAI is not configured")?.generate(&request).await,
            ProviderChoice::Mock => Err("MOCK_PROVIDER cannot be used as a success path in the kernel slice".into()),
        };

        if response.is_err() && selected.may_escalate {
            if let Some(provider) = openai.as_ref() {
                escalated = true;
                self.db.update_run_provider(&run_id, provider.name(), provider.model())?;
                response = provider.generate(&request).await;
            }
        }

        match response {
            Ok(result) => match validate_draft(&result.output) {
                Ok(output) => {
                    self.db.finish_run(&run_id, true, escalated, result.token_usage, result.estimated_cost, None)?;
                    self.db.set_employee_status("brain", "IDLE")?;
                    self.db.event(
                        Some(work_item_id),
                        "model.completed",
                        "brain",
                        json!({"artifact_type": "CONTENT_DRAFT", "escalated": escalated, "token_usage": result.token_usage}),
                    )?;
                    Ok(output)
                }
                Err(error) => {
                    self.fail_run(&run_id, work_item_id, &error)?;
                    Err(error)
                }
            },
            Err(error) => {
                self.fail_run(&run_id, work_item_id, &error)?;
                Err(error)
            }
        }
    }

    fn fail_run(&self, run_id: &str, work_item_id: &str, error: &str) -> Result<(), String> {
        self.db.finish_run(run_id, false, false, None, None, Some(error))?;
        self.db.set_employee_status("brain", "BLOCKED")?;
        self.db.event(Some(work_item_id), "model.failed", "brain", json!({"error": error, "fixture_recovery": false}))?;
        Ok(())
    }
}

pub fn brain_system_prompt() -> String {
    "You are Brain, writer for SAM SHERIF | PRACTICAL AI.\n\
     Write a LinkedIn post. Preserve the brief core_idea. Do not invent metrics, facts, tests, or visuals.\n\
     Do not publish. Sam is the only approver.\n\
     Return one JSON object with keys: artifact_type, created_by, hook, body, cta, core_idea, intended_platforms, format.\n\
     artifact_type=CONTENT_DRAFT, created_by=brain, format=LINKEDIN_POST, intended_platforms=[\"LINKEDIN\"].\n\
     body is the full post, under 1900 characters. hook is one sentence. cta is one concrete next step."
        .into()
}

pub fn brain_output_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["artifact_type", "created_by", "hook", "body", "cta", "core_idea", "intended_platforms", "format"],
        "properties": {
            "artifact_type": { "type": "string" },
            "created_by": { "type": "string" },
            "hook": { "type": "string" },
            "body": { "type": "string" },
            "cta": { "type": "string" },
            "core_idea": { "type": "string" },
            "intended_platforms": { "type": "array", "items": { "type": "string" } },
            "format": { "type": "string" }
        }
    })
}

fn brain_context(brief: &Value, revision: Option<&Value>) -> Value {
    json!({
        "brief_id": brief.get("brief_id"),
        "core_idea": brief.get("core_idea"),
        "audience_problem": brief.get("audience_problem"),
        "core_takeaway": brief.get("core_takeaway"),
        "cta_objective": brief.get("cta_objective"),
        "hook_direction": brief.get("hook_direction"),
        "limitation_or_caveat": brief.get("limitation_or_caveat"),
        "intended_platforms": ["LINKEDIN"],
        "revision": revision.and_then(|v| v.get("feedback")).cloned()
    })
}

fn validate_draft(output: &Value) -> Result<Value, String> {
    if !output.is_object() {
        return Err("Provider output is not JSON: expected a CONTENT_DRAFT object".into());
    }
    let body = output.get("body").and_then(Value::as_str).unwrap_or("").trim().to_string();
    let idea = output.get("core_idea").and_then(Value::as_str).unwrap_or("").trim().to_string();
    if body.is_empty() && idea.is_empty() {
        return Err("Provider output is not JSON: CONTENT_DRAFT missing body".into());
    }
    let mut output = output.clone();
    if let Some(obj) = output.as_object_mut() {
        obj.entry("artifact_type").or_insert(json!("CONTENT_DRAFT"));
        obj.entry("created_by").or_insert(json!("brain"));
        obj.entry("intended_platforms").or_insert(json!(["LINKEDIN"]));
        obj.entry("format").or_insert(json!("LINKEDIN_POST"));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_validation_rejects_non_objects_and_empty_bodies() {
        assert!(validate_draft(&json!("not-an-object")).is_err());
        assert!(validate_draft(&json!({"artifact_type": "CONTENT_DRAFT"})).is_err());
        assert!(validate_draft(&json!({"body": "A preserved research signal about approval gates"})).is_ok());
    }

    #[test]
    fn compact_schema_is_small_and_has_no_defs() {
        let schema = brain_output_schema();
        let encoded = schema.to_string();
        assert!(encoded.len() < 1200, "compact schema too large: {}", encoded.len());
        assert!(schema.get("$defs").is_none());
        assert!(schema.get("oneOf").is_none());
        assert_eq!(schema["required"].as_array().unwrap().len(), 8);
    }
}
