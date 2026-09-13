use serde_json::{json, Value};
use std::{collections::HashSet, fs, path::{Path, PathBuf}};

use crate::{
    db::Database,
    providers::{AiProvider, OllamaProvider, OpenAiProvider, ProviderRequest},
    router::{route, ProviderChoice},
    task_store::{CommandTask, TaskPreparation, TaskStore},
};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareCommandTaskResult {
    pub task: CommandTask,
    pub preparation: TaskPreparation,
}

#[derive(Clone)]
pub struct TaskPrepareEngine {
    store: TaskStore,
    db: Database,
    repository_root: PathBuf,
}

impl TaskPrepareEngine {
    pub fn new(store: TaskStore, db: Database, repository_root: PathBuf) -> Self {
        Self { store, db, repository_root }
    }

    pub async fn prepare(&self, task_id: &str, actor: &str) -> Result<PrepareCommandTaskResult, String> {
        let task = self.store.begin_preparation(task_id, actor)?;
        let previous = self.store.preparation(task_id)?;
        let operator_decision = previous.as_ref().and_then(|p| p.operator_decision.clone());
        let repository_context = collect_repository_context(&self.repository_root, &task)?;

        self.store.record_event(task_id, "prepare.explorer.deployed", "orchestrator", json!({"role":"explorer"}))?;
        let explorer = match self.run_role(
            task_id,
            "explorer",
            "classification",
            explorer_system_prompt(),
            json!({
                "task": task,
                "repository_context": repository_context,
                "operator_decision": operator_decision,
            }),
            explorer_schema(),
            1600,
        ).await {
            Ok(value) => value,
            Err(error) => {
                self.store.fail_preparation(task_id, "explorer", &error)?;
                return Err(error);
            }
        };
        self.store.save_prepare_role_output(task_id, "explorer", &explorer)?;

        self.store.record_event(task_id, "prepare.researcher.deployed", "orchestrator", json!({"role":"researcher"}))?;
        let researcher = match self.run_role(
            task_id,
            "researcher",
            "complex_reasoning",
            researcher_system_prompt(),
            json!({
                "task": self.store.get(task_id)?,
                "explorer_report": explorer,
                "operator_decision": operator_decision,
            }),
            researcher_schema(),
            1900,
        ).await {
            Ok(value) => value,
            Err(error) => {
                self.store.fail_preparation(task_id, "researcher", &error)?;
                return Err(error);
            }
        };
        self.store.save_prepare_role_output(task_id, "researcher", &researcher)?;

        if operator_decision.as_deref().unwrap_or("").trim().is_empty() {
            if let Some((question, recommendation)) = first_operator_question(&explorer, &researcher) {
                let preparation = self.store.require_operator_input(task_id, &question, recommendation.as_deref())?;
                return Ok(PrepareCommandTaskResult { task: self.store.get(task_id)?, preparation });
            }
        }

        self.store.record_event(task_id, "prepare.orchestrator.deployed", "orchestrator", json!({"role":"orchestrator"}))?;
        let orchestrator = match self.run_role(
            task_id,
            "orchestrator",
            "complex_reasoning",
            orchestrator_system_prompt(),
            json!({
                "task": self.store.get(task_id)?,
                "explorer_report": explorer,
                "researcher_report": researcher,
                "operator_decision": operator_decision,
                "command_center_rules": {
                    "prepare_is_not_start": true,
                    "task_must_remain_todo_after_prepare": true,
                    "builder_must_return_reviewable_artifact": true,
                    "completion_requires_review": true,
                    "never_invent_missing_requirements": true
                }
            }),
            orchestrator_schema(),
            2800,
        ).await {
            Ok(value) => value,
            Err(error) => {
                self.store.fail_preparation(task_id, "orchestrator", &error)?;
                return Err(error);
            }
        };
        self.store.save_prepare_role_output(task_id, "orchestrator", &orchestrator)?;

        let prompt = orchestrator.get("prompt_markdown").and_then(Value::as_str).unwrap_or("").trim();
        if prompt.is_empty() {
            let error = "Prepare orchestrator returned no prompt_markdown".to_string();
            self.store.fail_preparation(task_id, "orchestrator", &error)?;
            return Err(error);
        }
        let task = self.store.finalize_preparation(task_id, prompt, &orchestrator, "orchestrator")?;
        let preparation = self.store.preparation(task_id)?.ok_or("Preparation state disappeared after finalization")?;
        Ok(PrepareCommandTaskResult { task, preparation })
    }

    pub fn answer_operator(&self, task_id: &str, decision: &str) -> Result<TaskPreparation, String> {
        self.store.set_operator_decision(task_id, decision)
    }

    async fn run_role(
        &self,
        task_id: &str,
        role: &str,
        task_type: &str,
        system_prompt: String,
        context: Value,
        output_schema: Value,
        max_output_tokens: u32,
    ) -> Result<Value, String> {
        let settings = self.db.settings()?;
        let openai = OpenAiProvider::from_env(&settings);
        let selected = route(task_type, &settings, openai.is_some(), false);
        let request = ProviderRequest {
            agent: role.into(),
            task_type: format!("command_center_prepare_{role}"),
            expected_type: format!("{}_PREPARE_REPORT", role.to_uppercase()),
            system_prompt,
            context,
            output_schema,
            output_template: Value::Null,
            schema_resources: Default::default(),
            max_output_tokens,
        };

        let (primary_name, primary_model) = match selected.primary {
            ProviderChoice::Ollama => ("OLLAMA_PROVIDER", settings.ollama_model.as_str()),
            ProviderChoice::OpenAi => ("OPENAI_PROVIDER", settings.openai_model.as_str()),
            ProviderChoice::Mock => return Err("Mock provider is not permitted for Command Center preparation".into()),
        };
        self.store.record_event(task_id, &format!("prepare.{role}.model_started"), role, json!({"provider":primary_name,"model":primary_model}))?;

        let mut escalated = false;
        let mut response = match selected.primary {
            ProviderChoice::Ollama => OllamaProvider::new(&settings).generate(&request).await,
            ProviderChoice::OpenAi => openai.as_ref().ok_or("OpenAI is not configured")?.generate(&request).await,
            ProviderChoice::Mock => unreachable!(),
        };
        if response.is_err() && selected.may_escalate {
            if let Some(provider) = openai.as_ref() {
                escalated = true;
                response = provider.generate(&request).await;
            }
        }

        match response {
            Ok(result) => {
                let final_provider = if escalated { "OPENAI_PROVIDER" } else { primary_name };
                let final_model = if escalated { settings.openai_model.as_str() } else { primary_model };
                self.db.record_execution(
                    None,
                    role,
                    &format!("command_center.prepare.{role}.{task_id}"),
                    "MODEL_RUN",
                    Some(final_provider),
                    Some(final_model),
                    true,
                    None,
                )?;
                self.store.record_event(task_id, &format!("prepare.{role}.model_completed"), role, json!({
                    "provider": final_provider,
                    "model": final_model,
                    "escalated": escalated,
                    "token_usage": result.token_usage,
                    "estimated_cost": result.estimated_cost
                }))?;
                Ok(result.output)
            }
            Err(error) => {
                self.db.record_execution(
                    None,
                    role,
                    &format!("command_center.prepare.{role}.{task_id}"),
                    "MODEL_RUN",
                    Some(primary_name),
                    Some(primary_model),
                    false,
                    Some(&error),
                )?;
                self.store.record_event(task_id, &format!("prepare.{role}.model_failed"), role, json!({"error":error}))?;
                Err(error)
            }
        }
    }
}

fn explorer_system_prompt() -> String {
    "You are the Explorer transient execution role inside SAM Command Center. Your job is preparation, not implementation. Inspect the supplied repository snapshot and task metadata. Identify the existing architecture, relevant files, dependencies, conventions, constraints, prior implementation that can be reused, and conflicts that a builder must know before touching code. Do not write implementation code. Do not invent files or requirements. Ask the operator only when a missing decision materially changes architecture, scope, irreversible behavior, security, cost, or the requested outcome. Return JSON only.".into()
}

fn researcher_system_prompt() -> String {
    "You are the Researcher transient execution role inside SAM Command Center. Convert the task plus Explorer report into an execution study. Define the real objective, success criteria, constraints, unknowns, likely failure modes, implementation sequence, validation checks, and safe parallelism/dependencies. Reuse existing project decisions instead of reopening them. Do not implement. Ask the operator only for genuinely material unresolved decisions. Return JSON only.".into()
}

fn orchestrator_system_prompt() -> String {
    "You are the Prepare Orchestrator for SAM Command Center. You receive a task, Explorer report, Researcher report, and optional operator decision. Produce the strongest possible builder execution prompt. The prompt must be specific to this repository and task, name relevant files/context, state what to preserve, list ordered implementation steps, acceptance criteria, validation/tests, review expectations, dependency rules, and explicit non-goals. PREPARE never executes the task; the task remains TODO until START TASK. Do not invent requirements or claim work has been completed. Return JSON only.".into()
}

fn explorer_schema() -> Value {
    json!({
        "type":"object","additionalProperties":false,
        "required":["summary","relevant_paths","existing_components","dependencies","constraints","risks","operator_questions","confidence"],
        "properties":{
            "summary":{"type":"string"},
            "relevant_paths":{"type":"array","items":{"type":"string"}},
            "existing_components":{"type":"array","items":{"type":"string"}},
            "dependencies":{"type":"array","items":{"type":"string"}},
            "constraints":{"type":"array","items":{"type":"string"}},
            "risks":{"type":"array","items":{"type":"string"}},
            "operator_questions":{"type":"array","items":question_schema()},
            "confidence":{"type":"string"}
        }
    })
}

fn researcher_schema() -> Value {
    json!({
        "type":"object","additionalProperties":false,
        "required":["objective","success_criteria","constraints","unknowns","execution_steps","parallelizable_work","review_checks","failure_modes","operator_questions","confidence"],
        "properties":{
            "objective":{"type":"string"},
            "success_criteria":{"type":"array","items":{"type":"string"}},
            "constraints":{"type":"array","items":{"type":"string"}},
            "unknowns":{"type":"array","items":{"type":"string"}},
            "execution_steps":{"type":"array","items":{"type":"string"}},
            "parallelizable_work":{"type":"array","items":{"type":"string"}},
            "review_checks":{"type":"array","items":{"type":"string"}},
            "failure_modes":{"type":"array","items":{"type":"string"}},
            "operator_questions":{"type":"array","items":question_schema()},
            "confidence":{"type":"string"}
        }
    })
}

fn orchestrator_schema() -> Value {
    json!({
        "type":"object","additionalProperties":false,
        "required":["prompt_markdown","execution_plan","review_chain","acceptance_criteria","non_goals","can_start"],
        "properties":{
            "prompt_markdown":{"type":"string"},
            "execution_plan":{"type":"array","items":{"type":"string"}},
            "review_chain":{"type":"array","items":{"type":"string"}},
            "acceptance_criteria":{"type":"array","items":{"type":"string"}},
            "non_goals":{"type":"array","items":{"type":"string"}},
            "can_start":{"type":"boolean"}
        }
    })
}

fn question_schema() -> Value {
    json!({
        "type":"object","additionalProperties":false,
        "required":["question","options","recommendation","reason"],
        "properties":{
            "question":{"type":"string"},
            "options":{"type":"array","items":{"type":"string"}},
            "recommendation":{"type":"string"},
            "reason":{"type":"string"}
        }
    })
}

fn first_operator_question(explorer: &Value, researcher: &Value) -> Option<(String, Option<String>)> {
    for source in [explorer, researcher] {
        let question = source.get("operator_questions")?.as_array()?.first()?;
        let text = question.get("question").and_then(Value::as_str)?.trim();
        if text.is_empty() { continue; }
        let options = question.get("options").and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_default();
        let reason = question.get("reason").and_then(Value::as_str).unwrap_or("").trim();
        let rendered = if options.is_empty() {
            if reason.is_empty() { text.to_string() } else { format!("{text}\n\nWhy this matters: {reason}") }
        } else {
            format!("{text}\n\nOptions:\n{}{}", options.iter().enumerate().map(|(i,o)| format!("{}. {o}", i+1)).collect::<Vec<_>>().join("\n"), if reason.is_empty(){String::new()}else{format!("\n\nWhy this matters: {reason}")})
        };
        let recommendation = question.get("recommendation").and_then(Value::as_str).map(str::to_string).filter(|s| !s.trim().is_empty());
        return Some((rendered, recommendation));
    }
    None
}

fn collect_repository_context(root: &Path, task: &CommandTask) -> Result<Value, String> {
    let mut paths = Vec::new();
    collect_paths(root, root, &mut paths, 0)?;
    paths.sort();
    paths.truncate(300);

    let keywords = task_keywords(task);
    let mandatory = [
        "README.md",
        "docs/command-center-task-operating-model.md",
        "docs/product-vision.md",
        "docs/slice-v1-kernel-loop.md",
        "docs/integration-architecture.md",
        "docs/company-state-machine.md",
        "desktop/src-tauri/src/task_store.rs",
        "desktop/src-tauri/src/lib.rs",
    ];
    let mut selected: Vec<String> = mandatory.iter().filter(|p| root.join(p).is_file()).map(|s| s.to_string()).collect();
    if !task.owner.trim().is_empty() {
        for suffix in ["manifest.yaml", "contract.md", "prompt.md"] {
            let path = format!("agents/{}/{suffix}", task.owner.to_lowercase());
            if root.join(&path).is_file() { selected.push(path); }
        }
    }
    let mut scored: Vec<(usize, String)> = paths.iter().map(|path| {
        let lower = path.to_lowercase();
        let score = keywords.iter().filter(|word| lower.contains(word.as_str())).count();
        (score, path.clone())
    }).filter(|(score,_)| *score > 0).collect();
    scored.sort_by(|a,b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    selected.extend(scored.into_iter().take(10).map(|(_,path)| path));
    let mut seen = HashSet::new();
    selected.retain(|p| seen.insert(p.clone()));
    selected.truncate(14);

    let files = selected.into_iter().filter_map(|path| {
        let full = root.join(&path);
        let text = fs::read_to_string(full).ok()?;
        let snippet: String = text.chars().take(5000).collect();
        Some(json!({"path":path,"snippet":snippet}))
    }).collect::<Vec<_>>();

    Ok(json!({
        "repository_files_sample": paths,
        "selected_file_context": files,
        "selection_keywords": keywords
    }))
}

fn collect_paths(root: &Path, dir: &Path, output: &mut Vec<String>, depth: usize) -> Result<(), String> {
    if depth > 7 || output.len() >= 600 { return Ok(()); }
    let entries = fs::read_dir(dir).map_err(|e| format!("Cannot scan {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if matches!(name.as_str(), ".git" | "node_modules" | "target" | "dist" | "build" | ".next") { continue; }
            collect_paths(root, &path, output, depth + 1)?;
        } else if is_text_context_file(&path) {
            if let Ok(relative) = path.strip_prefix(root) {
                output.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
        if output.len() >= 600 { break; }
    }
    Ok(())
}

fn is_text_context_file(path: &Path) -> bool {
    matches!(path.extension().and_then(|v| v.to_str()).unwrap_or(""),
        "md" | "rs" | "ts" | "tsx" | "js" | "jsx" | "json" | "yaml" | "yml" | "toml" | "sql")
}

fn task_keywords(task: &CommandTask) -> Vec<String> {
    let raw = format!("{} {} {} {}", task.title, task.description, task.owner, task.milestone.clone().unwrap_or_default()).to_lowercase();
    let stop = ["this","that","with","from","into","task","make","build","create","have","will","your","about","على","عمل","محتاج","عاوز"];
    let mut words = raw.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .map(str::trim).filter(|w| w.len() >= 4 && !stop.contains(w)).map(str::to_string).collect::<Vec<_>>();
    words.sort(); words.dedup(); words.truncate(20); words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_question_renders_options_and_recommendation() {
        let explorer = json!({"operator_questions":[{"question":"Choose scope","options":["A","B"],"recommendation":"B","reason":"B preserves the current boundary"}]});
        let researcher = json!({"operator_questions":[]});
        let (question, recommendation) = first_operator_question(&explorer, &researcher).unwrap();
        assert!(question.contains("1. A"));
        assert!(question.contains("Why this matters"));
        assert_eq!(recommendation.as_deref(), Some("B"));
    }
}
