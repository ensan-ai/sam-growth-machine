use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::{collections::HashMap, time::Duration};

use crate::models::{ProviderStatus, RuntimeSettings};

pub const BRAIN_MAX_OUTPUT_TOKENS: u32 = 2048;

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub agent: String,
    pub task_type: String,
    pub expected_type: String,
    pub system_prompt: String,
    pub context: Value,
    pub output_schema: Value,
    pub output_template: Value,
    pub schema_resources: HashMap<String, Value>,
    pub max_output_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub output: Value,
    pub token_usage: Option<i64>,
    pub estimated_cost: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaStats {
    pub done: bool,
    pub done_reason: String,
    pub eval_count: i64,
    pub prompt_eval_count: i64,
    pub response_len: usize,
}

impl OllamaStats {
    pub fn from_body(body: &Value, response_len: usize) -> Self {
        Self {
            done: body.get("done").and_then(Value::as_bool).unwrap_or(false),
            done_reason: body.get("done_reason").and_then(Value::as_str).unwrap_or("").to_string(),
            eval_count: body.get("eval_count").and_then(Value::as_i64).unwrap_or(0),
            prompt_eval_count: body.get("prompt_eval_count").and_then(Value::as_i64).unwrap_or(0),
            response_len,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "done={} done_reason={} eval_count={} prompt_eval_count={} response_len={}",
            self.done, self.done_reason, self.eval_count, self.prompt_eval_count, self.response_len
        )
    }

    pub fn truncated(&self) -> bool {
        self.done_reason == "length"
    }
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn model(&self) -> &str;
    async fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, String>;
}

pub struct OllamaProvider { endpoint:String, model:String, client:Client }
impl OllamaProvider { pub fn new(settings:&RuntimeSettings)->Self{Self{endpoint:settings.ollama_endpoint.trim_end_matches('/').into(),model:settings.ollama_model.clone(),client:Client::builder().timeout(Duration::from_secs(300)).build().unwrap()}} }

#[async_trait]
impl AiProvider for OllamaProvider {
    fn name(&self)->&'static str{"OLLAMA_PROVIDER"} fn model(&self)->&str{&self.model}
    async fn generate(&self,request:&ProviderRequest)->Result<ProviderResponse,String>{
        let prompt = brain_prompt(request);
        let num_predict = if request.max_output_tokens == 0 { BRAIN_MAX_OUTPUT_TOKENS } else { request.max_output_tokens };
        let format = if request.output_schema.is_object() { request.output_schema.clone() } else { json!("json") };
        let mut body = self.complete(&prompt, &format, num_predict).await?;
        if is_format_rejection(&body) && format != json!("json") {
            body = self.complete(&prompt, &json!("json"), num_predict).await?;
        }
        interpret_ollama_body(&body)
    }
}

impl OllamaProvider {
    async fn complete(&self, prompt: &str, format: &Value, num_predict: u32) -> Result<Value, String> {
        let response = self.client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&json!({
                "model": self.model,
                "prompt": prompt,
                "stream": false,
                "think": false,
                "format": format,
                "options": { "temperature": 0.2, "num_predict": num_predict }
            }))
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {e}"))?;
        if !response.status().is_success() {
            let status = response.status();
            let detail = response.text().await.unwrap_or_default();
            return Err(format!("Ollama returned {status}: {}", detail.chars().take(240).collect::<String>()));
        }
        response.json().await.map_err(|e| format!("Invalid Ollama response: {e}"))
    }
}

pub struct OpenAiProvider { model:String, key:String, client:Client }
impl OpenAiProvider { pub fn from_env(settings:&RuntimeSettings)->Option<Self>{std::env::var("OPENAI_API_KEY").ok().filter(|k|!k.trim().is_empty()).map(|key|Self{model:settings.openai_model.clone(),key,client:Client::builder().timeout(Duration::from_secs(300)).build().unwrap()})} }

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn name(&self)->&'static str{"OPENAI_PROVIDER"} fn model(&self)->&str{&self.model}
    async fn generate(&self,request:&ProviderRequest)->Result<ProviderResponse,String>{
        let input = brain_prompt(request);
        let response=self.client.post("https://api.openai.com/v1/chat/completions").bearer_auth(&self.key).json(&json!({
            "model":self.model,
            "messages":[{"role":"system","content":request.system_prompt},{"role":"user","content":input}],
            "response_format":{"type":"json_object"},
            "max_tokens": request.max_output_tokens.max(256)
        })).send().await.map_err(|e|format!("OpenAI request failed: {e}"))?;
        if !response.status().is_success(){let status=response.status();let detail=response.text().await.unwrap_or_default();return Err(format!("OpenAI returned {status}: {}",detail.chars().take(240).collect::<String>()));}
        let body:Value=response.json().await.map_err(|e|format!("Invalid OpenAI response: {e}"))?;
        let text=body.pointer("/choices/0/message/content").and_then(Value::as_str).ok_or("OpenAI response contained no output")?;
        let usage=body.pointer("/usage/total_tokens").and_then(Value::as_i64);
        let finish = body.pointer("/choices/0/finish_reason").and_then(Value::as_str).unwrap_or("");
        parse_json_output(text).map_err(|e| {
            let kind = if finish == "length" || e.contains("EOF") { "truncated JSON" } else { "invalid JSON" };
            format!("Provider output is not JSON: {e} ({kind}) | finish_reason={finish} response_len={}", text.len())
        }).map(|output| ProviderResponse{output,token_usage:usage,estimated_cost:None})
    }
}

#[allow(dead_code)]
pub struct MockProvider { model:String }
impl MockProvider { pub fn new()->Self{Self{model:"deterministic-schema-fixture-v1".into()}} }
#[async_trait]
impl AiProvider for MockProvider {
    fn name(&self)->&'static str{"MOCK_PROVIDER"} fn model(&self)->&str{&self.model}
    async fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, String> {
        let _ = request;
        Err("MOCK_PROVIDER cannot be used as a success path in the kernel slice".into())
    }
}

pub async fn detect(settings:&RuntimeSettings)->ProviderStatus{
    let client=Client::builder().timeout(Duration::from_secs(4)).build().unwrap();
    let endpoint=settings.ollama_endpoint.trim_end_matches('/');
    match client.get(format!("{endpoint}/api/tags")).send().await {
        Ok(response) if response.status().is_success()=>{
            let value:Value=response.json().await.unwrap_or(Value::Null);
            let found=model_present(&value,&settings.ollama_model);
            ProviderStatus{ollama_available:true,ollama_model_available:found,ollama_endpoint:settings.ollama_endpoint.clone(),ollama_model:settings.ollama_model.clone(),openai_configured:std::env::var("OPENAI_API_KEY").map(|v|!v.trim().is_empty()).unwrap_or(false),openai_model:settings.openai_model.clone(),message:if found{"Ollama and configured model are ready".into()}else{format!("Ollama is running but {} is missing",settings.ollama_model)}}
        }
        _=>ProviderStatus{ollama_available:false,ollama_model_available:false,ollama_endpoint:settings.ollama_endpoint.clone(),ollama_model:settings.ollama_model.clone(),openai_configured:std::env::var("OPENAI_API_KEY").map(|v|!v.trim().is_empty()).unwrap_or(false),openai_model:settings.openai_model.clone(),message:"Ollama is unavailable; paid fallback will not happen unless explicitly enabled".into()}
    }
}

fn model_present(value:&Value,model:&str)->bool{value.get("models").and_then(Value::as_array).map(|models|models.iter().any(|m|m.get("name").and_then(Value::as_str)==Some(model)||m.get("model").and_then(Value::as_str)==Some(model))).unwrap_or(false)}

fn brain_prompt(request: &ProviderRequest) -> String {
    format!(
        "{}\n\nTASK: {}\nEXPECTED ARTIFACT: {}\nINPUT:\n{}\n\nReturn one JSON object only.",
        request.system_prompt,
        request.task_type,
        request.expected_type,
        request.context
    )
}

fn is_format_rejection(body: &Value) -> bool {
    body.get("error").and_then(Value::as_str).map(|e| e.to_lowercase().contains("format") || e.to_lowercase().contains("schema")).unwrap_or(false)
}

pub fn truncated_provider_error_body() -> Value {
    json!({
        "response": "{\n  \"artifact_type\": \"CONTENT_DRAFT\",\n  \"created_by\": \"brain\",\n  \"hook\": \"Practical AI\",\n  \"body\": \"This object is incomplete",
        "done": true,
        "done_reason": "length",
        "eval_count": 768,
        "prompt_eval_count": 4100
    })
}

pub fn interpret_ollama_body(body: &Value) -> Result<ProviderResponse, String> {
    if let Some(error) = body.get("error").and_then(Value::as_str) {
        return Err(format!("Ollama error: {error}"));
    }
    let text = body.get("response").and_then(Value::as_str).unwrap_or("");
    let stats = OllamaStats::from_body(body, text.len());
    if text.trim().is_empty() {
        return Err(format!("Ollama response contained no output | {}", stats.summary()));
    }
    match parse_json_output(text) {
        Ok(output) => {
            let usage = stats.prompt_eval_count + stats.eval_count;
            Ok(ProviderResponse {
                output,
                token_usage: (usage > 0).then_some(usage),
                estimated_cost: Some(0.0),
            })
        }
        Err(error) => {
            let kind = if stats.truncated() || error.contains("EOF") { "truncated JSON" } else { "invalid JSON" };
            Err(format!("Provider output is not JSON: {error} ({kind}) | {}", stats.summary()))
        }
    }
}

fn parse_json_output(text:&str)->Result<Value,String>{
    let trimmed = text.trim();
    let trimmed = trimmed.strip_prefix("```json").or_else(|| trimmed.strip_prefix("```")).unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix("```").unwrap_or(trimmed).trim();
    serde_json::from_str(trimmed).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn detects_configured_ollama_model_from_tags(){
        let tags=json!({"models":[{"name":"qwen3:14b"}]});
        assert!(model_present(&tags,"qwen3:14b"));
        assert!(!model_present(&tags,"other"));
    }

    #[test]
    fn truncated_json_is_not_recovered() {
        let err = interpret_ollama_body(&truncated_provider_error_body()).unwrap_err();
        assert!(err.contains("Provider output is not JSON"), "{err}");
        assert!(err.contains("truncated JSON"), "{err}");
        assert!(err.contains("done_reason=length"), "{err}");
        assert!(err.contains("eval_count=768"), "{err}");
        assert!(err.contains("response_len="), "{err}");
        assert!(!err.to_lowercase().contains("fixture"), "{err}");
    }

    #[test]
    fn complete_json_parses_with_stats_usage() {
        let body = json!({
            "response": "{\"artifact_type\":\"CONTENT_DRAFT\",\"body\":\"ok\"}",
            "done": true,
            "done_reason": "stop",
            "eval_count": 40,
            "prompt_eval_count": 200
        });
        let result = interpret_ollama_body(&body).unwrap();
        assert_eq!(result.output["artifact_type"], "CONTENT_DRAFT");
        assert_eq!(result.token_usage, Some(240));
    }
}
