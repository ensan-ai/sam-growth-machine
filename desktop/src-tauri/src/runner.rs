use serde_json::{json, Value};

use crate::{
    db::Database,
    definitions::DefinitionStore,
    providers::{AiProvider, OllamaProvider, OpenAiProvider, ProviderRequest},
    router::{route, ProviderChoice},
    schema_fixture::generate_for_type_with_resources,
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
        let definition = self.definitions.get("brain")?.clone();
        let settings = self.db.settings()?;
        let openai = OpenAiProvider::from_env(&settings);
        let selected = route("public_writing", &settings, openai.is_some(), false);
        if matches!(selected.primary, ProviderChoice::Mock) {
            return Err("MOCK_PROVIDER cannot be used as a success path in the kernel slice".into());
        }
        let resources = self.definitions.schema_resources();
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
            json!({"capability": "write_public_copy", "provider": provider_name, "expected_type": "CONTENT_DRAFT"}),
        )?;

        let output_template = generate_for_type_with_resources(&definition.output_schema, "CONTENT_DRAFT", &resources);
        let context = json!({
            "content_brief": brief,
            "revision": revision,
            "intended_platforms": ["LINKEDIN"],
            "instruction": "Write a LinkedIn post that preserves the brief core_idea. Return JSON only. Do not invent platform metrics."
        });
        let request = ProviderRequest {
            agent: "brain".into(),
            task_type: "public_writing".into(),
            expected_type: "CONTENT_DRAFT".into(),
            system_prompt: format!("{}\n\nAUTHORITATIVE CONTRACT:\n{}", definition.prompt, definition.contract),
            context,
            output_schema: definition.output_schema.clone(),
            output_template,
            schema_resources: resources,
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
                        json!({"artifact_type": "CONTENT_DRAFT", "escalated": escalated}),
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
        self.db.event(Some(work_item_id), "model.failed", "brain", json!({"error": error}))?;
        Ok(())
    }
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
}
