use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::{collections::HashMap, time::Duration};

use crate::{models::{ProviderStatus, RuntimeSettings}, schema_fixture::generate_for_type_with_resources};

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
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub output: Value,
    pub token_usage: Option<i64>,
    pub estimated_cost: Option<f64>,
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
        let prompt=format!("{}\n\nAGENT: {}\nTASK: {}\nEXPECTED ARTIFACT: {}\nCURRENT INPUT CONTEXT:\n{}\n\nReturn only one concise JSON value with exactly the same structure and valid value classes as this contract-derived template:\n{}",request.system_prompt,request.agent,request.task_type,request.expected_type,request.context,request.output_template);
        let response=self.client.post(format!("{}/api/generate",self.endpoint)).json(&json!({"model":self.model,"prompt":prompt,"stream":false,"think":false,"format":"json","options":{"temperature":0.2,"num_predict":768}})).send().await.map_err(|e|format!("Ollama request failed: {e}"))?;
        if !response.status().is_success(){return Err(format!("Ollama returned {}",response.status()));}
        let body:Value=response.json().await.map_err(|e|format!("Invalid Ollama response: {e}"))?;
        let text=body.get("response").and_then(Value::as_str).ok_or("Ollama response contained no output")?;
        let output=parse_json_output(text)?;
        let usage=body.get("prompt_eval_count").and_then(Value::as_i64).unwrap_or(0)+body.get("eval_count").and_then(Value::as_i64).unwrap_or(0);
        Ok(ProviderResponse{output,token_usage:(usage>0).then_some(usage),estimated_cost:Some(0.0)})
    }
}

pub struct OpenAiProvider { model:String, key:String, client:Client }
impl OpenAiProvider { pub fn from_env(settings:&RuntimeSettings)->Option<Self>{std::env::var("OPENAI_API_KEY").ok().filter(|k|!k.trim().is_empty()).map(|key|Self{model:settings.openai_model.clone(),key,client:Client::builder().timeout(Duration::from_secs(300)).build().unwrap()})} }

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn name(&self)->&'static str{"OPENAI_PROVIDER"} fn model(&self)->&str{&self.model}
    async fn generate(&self,request:&ProviderRequest)->Result<ProviderResponse,String>{
        let input=format!("TASK: {}\nEXPECTED ARTIFACT: {}\nINPUT CONTEXT:\n{}\nReturn only concise JSON matching this contract-derived template:\n{}",request.task_type,request.expected_type,request.context,request.output_template);
        let response=self.client.post("https://api.openai.com/v1/chat/completions").bearer_auth(&self.key).json(&json!({"model":self.model,"messages":[{"role":"system","content":request.system_prompt},{"role":"user","content":input}],"response_format":{"type":"json_object"}})).send().await.map_err(|e|format!("OpenAI request failed: {e}"))?;
        if !response.status().is_success(){let status=response.status();let detail=response.text().await.unwrap_or_default();return Err(format!("OpenAI returned {status}: {}",detail.chars().take(240).collect::<String>()));}
        let body:Value=response.json().await.map_err(|e|format!("Invalid OpenAI response: {e}"))?;
        let text=body.pointer("/choices/0/message/content").and_then(Value::as_str).ok_or("OpenAI response contained no output")?;
        let usage=body.pointer("/usage/total_tokens").and_then(Value::as_i64);
        Ok(ProviderResponse{output:parse_json_output(text)?,token_usage:usage,estimated_cost:None})
    }
}

pub struct MockProvider { model:String }
impl MockProvider { pub fn new()->Self{Self{model:"deterministic-schema-fixture-v1".into()}} }
#[async_trait]
impl AiProvider for MockProvider {
    fn name(&self)->&'static str{"MOCK_PROVIDER"} fn model(&self)->&str{&self.model}
    async fn generate(&self,request:&ProviderRequest)->Result<ProviderResponse,String>{Ok(ProviderResponse{output:generate_for_type_with_resources(&request.output_schema,&request.expected_type,&request.schema_resources),token_usage:None,estimated_cost:Some(0.0)})}
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

fn parse_json_output(text:&str)->Result<Value,String>{
    serde_json::from_str(text).or_else(|_|{let start=text.find('{').ok_or_else(||serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData,"no JSON")))?;let end=text.rfind('}').ok_or_else(||serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData,"no JSON")))?;serde_json::from_str(&text[start..=end])}).map_err(|e|format!("Provider output is not JSON: {e}"))
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn detects_configured_ollama_model_from_tags(){let tags=json!({"models":[{"name":"qwen3:14b"}]});assert!(model_present(&tags,"qwen3:14b"));assert!(!model_present(&tags,"other"));}
}
