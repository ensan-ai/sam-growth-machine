use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeSummary {
    pub employee_id: String,
    pub name: String,
    pub title: String,
    pub reports_to: String,
    pub version: String,
    pub definition_status: String,
    pub runtime_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub ollama_available: bool,
    pub ollama_model_available: bool,
    pub ollama_endpoint: String,
    pub ollama_model: String,
    pub openai_configured: bool,
    pub openai_model: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSettings {
    pub ollama_endpoint: String,
    pub ollama_model: String,
    pub openai_model: String,
    pub allow_openai_escalation: bool,
    pub company_control: String,
    #[serde(default = "default_observe")]
    pub observe_quality: String,
    #[serde(default)]
    pub brain_force_fail: bool,
    #[serde(default)]
    pub publish_force_fail: bool,
}

fn default_observe() -> String { "UNAVAILABLE".into() }

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            ollama_endpoint: "http://localhost:11434".into(),
            ollama_model: "qwen3:14b".into(),
            openai_model: "gpt-5.6".into(),
            allow_openai_escalation: false,
            company_control: "RUNNING".into(),
            observe_quality: "UNAVAILABLE".into(),
            brain_force_fail: false,
            publish_force_fail: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemSummary {
    pub work_item_id: String,
    pub title: String,
    pub state: String,
    pub current_owner: String,
    pub created_at: String,
    pub updated_at: String,
    pub blocked_reason: Option<String>,
    pub resume_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRecord {
    pub artifact_id: String,
    pub artifact_type: String,
    pub work_item_id: String,
    pub version: i64,
    pub producer: String,
    pub created_at: String,
    pub supersedes: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffRecord {
    pub handoff_id: String,
    pub work_item_id: String,
    pub sender: String,
    pub receiver: String,
    pub artifact_id: String,
    pub status: String,
    pub created_at: String,
    pub acknowledged_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRecord {
    pub approval_id: String,
    pub work_item_id: String,
    pub approver: String,
    pub content_artifact_id: String,
    pub content_version: i64,
    pub creative_artifact_id: String,
    pub creative_version: i64,
    pub package_artifact_id: String,
    pub package_version: i64,
    pub platform_scope: Vec<String>,
    pub status: String,
    pub feedback: Option<String>,
    pub created_at: String,
    pub superseded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunRecord {
    pub run_id: String,
    pub work_item_id: String,
    pub agent: String,
    pub task_type: String,
    pub provider: String,
    pub model: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub success: bool,
    pub escalation_occurred: bool,
    pub token_usage: Option<i64>,
    pub estimated_api_cost: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRecord {
    pub execution_id: String,
    pub work_item_id: Option<String>,
    pub role: String,
    pub capability: String,
    pub kind: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningRecord {
    pub learning_id: String,
    pub source_work_item_id: String,
    pub opportunity_fingerprint: String,
    pub cycle_decision: String,
    pub data_quality: String,
    pub recommended_attention: Option<String>,
    pub payload: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemEventRecord {
    pub event_id: String,
    pub work_item_id: Option<String>,
    pub event_type: String,
    pub actor: String,
    pub detail: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemDetail {
    pub work_item: WorkItemSummary,
    pub artifacts: Vec<ArtifactRecord>,
    pub handoffs: Vec<HandoffRecord>,
    pub approvals: Vec<ApprovalRecord>,
    pub runs: Vec<AgentRunRecord>,
    pub events: Vec<SystemEventRecord>,
    #[serde(default)]
    pub executions: Vec<ExecutionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub company_state: String,
    pub provider_status: ProviderStatus,
    pub active_agents: Vec<String>,
    pub waiting_approvals: i64,
    pub blocked_work: i64,
    pub work_items: Vec<WorkItemSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartWorkItemRequest {
    pub title: String,
    pub research_signal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDecision {
    pub work_item_id: String,
    pub action: String,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlRequest {
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSettingsRequest {
    pub ollama_endpoint: String,
    pub ollama_model: String,
    pub openai_model: String,
    pub allow_openai_escalation: bool,
}
