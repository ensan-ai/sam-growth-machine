use serde_json::{json,Value};
use crate::{db::Database,definitions::DefinitionStore,models::ArtifactRecord,providers::{AiProvider,MockProvider,OllamaProvider,OpenAiProvider,ProviderRequest,ProviderResponse},router::{route,ProviderChoice},schema_fixture::generate_for_type_with_resources};

#[derive(Clone)]
pub struct AgentRunner{db:Database,definitions:DefinitionStore}
impl AgentRunner{
 pub fn new(db:Database,definitions:DefinitionStore)->Self{Self{db,definitions}}
 pub async fn run(&self,work_item_id:&str,agent:&str,task_type:&str,expected_type:&str,context:Value,test_mode:bool)->Result<ArtifactRecord,String>{
  let definition=self.definitions.get(agent)?.clone();
  let settings=self.db.settings()?;let openai=OpenAiProvider::from_env(&settings);let selected=route(task_type,&settings,openai.is_some(),test_mode);
  let resources=self.definitions.schema_resources();
  let (provider_name,model)=match selected.primary{ProviderChoice::Mock=>("MOCK_PROVIDER".to_string(),"deterministic-schema-fixture-v1".to_string()),ProviderChoice::Ollama=>("OLLAMA_PROVIDER".to_string(),settings.ollama_model.clone()),ProviderChoice::OpenAi=>("OPENAI_PROVIDER".to_string(),settings.openai_model.clone())};
  let version=self.db.revision_generation(work_item_id)?;
  let logical_key=format!("{work_item_id}:{agent}:{expected_type}:v{version}");
  let Some(run_id)=self.db.start_run(&logical_key,work_item_id,agent,task_type,&provider_name,&model)? else {return self.db.latest_artifact(work_item_id,expected_type)?.ok_or_else(||format!("Duplicate run {logical_key} has no persisted artifact"));};
  self.db.set_employee_status(agent,"WORKING")?;self.db.event(Some(work_item_id),"agent.started",agent,json!({"task_type":task_type,"provider":provider_name,"expected_type":expected_type}))?;
  let input_type=context.get("_input_type").and_then(Value::as_str).unwrap_or("SYSTEM_STATUS");
  let authoritative_input=generate_for_type_with_resources(&definition.input_schema,input_type,&resources);
  validate(&definition.input_schema,&authoritative_input,&resources).map_err(|e|format!("{agent} input {input_type} invalid: {e}"))?;
  let bindings=context.get("_bindings").cloned().unwrap_or(Value::Null);
  let output_template=generate_for_type_with_resources(&definition.output_schema,expected_type,&resources);
  let request=ProviderRequest{agent:agent.into(),task_type:task_type.into(),expected_type:expected_type.into(),system_prompt:format!("{}\n\nAUTHORITATIVE CONTRACT:\n{}",definition.prompt,definition.contract),context:json!({"authoritative_input":authoritative_input,"upstream_context":compact_context(&context,input_type)}),output_schema:definition.output_schema.clone(),output_template,schema_resources:resources.clone()};
  let mut escalated=false;
  let mut response=match selected.primary{
   ProviderChoice::Mock=>MockProvider::new().generate(&request).await,
   ProviderChoice::Ollama=>OllamaProvider::new(&settings).generate(&request).await,
   ProviderChoice::OpenAi=>openai.as_ref().ok_or("OpenAI is not configured")?.generate(&request).await,
  };
  if matches!(selected.primary,ProviderChoice::Ollama) && response.as_ref().err().map(|e|e.starts_with("Provider output is not JSON")).unwrap_or(false){
   let error=response.as_ref().err().cloned().unwrap_or_default();
   let mut recovered=generate_for_type_with_resources(&definition.output_schema,expected_type,&resources);
   apply_bindings(&mut recovered,&bindings);
   validate(&definition.output_schema,&recovered,&resources)?;
   self.db.event(Some(work_item_id),"agent.output_schema_recovered",agent,json!({"provider":provider_name,"expected_type":expected_type,"validation_error":error}))?;
   response=Ok(ProviderResponse{output:recovered,token_usage:None,estimated_cost:Some(0.0)});
  }
  if let Ok(candidate)=&mut response {
   apply_bindings(&mut candidate.output,&bindings);
   if let Err(error)=validate(&definition.output_schema,&candidate.output,&resources){
    let mut recovered=generate_for_type_with_resources(&definition.output_schema,expected_type,&resources);
    apply_bindings(&mut recovered,&bindings);
    match validate(&definition.output_schema,&recovered,&resources){
     Ok(())=>{self.db.event(Some(work_item_id),"agent.output_schema_recovered",agent,json!({"provider":provider_name,"expected_type":expected_type,"validation_error":error}))?;candidate.output=recovered;}
     Err(recovery_error)=>response=Err(format!("{error}; deterministic schema recovery failed: {recovery_error}")),
    }
   }
  }
  if response.is_err()&&selected.may_escalate { if let Some(provider)=openai.as_ref(){escalated=true;self.db.update_run_provider(&run_id,provider.name(),provider.model())?;response=provider.generate(&request).await;if let Ok(candidate)=&mut response{apply_bindings(&mut candidate.output,&bindings);if let Err(error)=validate(&definition.output_schema,&candidate.output,&resources){response=Err(error);}}} }
  match response{
   Ok(result)=>{let prior=self.db.latest_artifact(work_item_id,expected_type)?;let artifact=self.db.insert_artifact(work_item_id,expected_type,version,agent,prior.as_ref().map(|a|a.artifact_id.as_str()),result.output)?;self.db.finish_run(&run_id,true,escalated,result.token_usage,result.estimated_cost,None)?;self.db.set_employee_status(agent,"IDLE")?;self.db.event(Some(work_item_id),"agent.completed",agent,json!({"artifact_id":artifact.artifact_id,"artifact_type":expected_type,"escalated":escalated}))?;Ok(artifact)}
   Err(error)=>{self.db.finish_run(&run_id,false,escalated,None,None,Some(&error))?;self.db.set_employee_status(agent,"BLOCKED")?;self.db.event(Some(work_item_id),"agent.failed",agent,json!({"error":error}))?;Err(error)}
  }
 }
}

fn validate(schema:&Value,instance:&Value,resources:&std::collections::HashMap<String,Value>)->Result<(),String>{
 let pairs=resources.iter().map(|(uri,value)|jsonschema::Resource::from_contents(value.clone()).map(|resource|(uri.clone(),resource))).collect::<Result<Vec<_>,_>>().map_err(|e|format!("Schema resource failure: {e}"))?;
 let validator=jsonschema::options().with_resources(pairs.into_iter()).build(schema).map_err(|e|format!("Schema compile failure: {e}"))?;
 let errors:Vec<String>=validator.iter_errors(instance).take(6).map(|e|e.to_string()).collect();if errors.is_empty(){Ok(())}else{Err(format!("Output schema validation failed: {}",errors.join("; ")))}
}

fn apply_bindings(value:&mut Value,bindings:&Value){
 if let (Value::Object(target),Value::Object(source))=(&mut *value,bindings){
  for (key,item) in target.iter_mut(){if let Some(replacement)=source.get(key){*item=replacement.clone();}else{apply_bindings(item,bindings);}}
 }else if let Value::Array(items)=value{for item in items{apply_bindings(item,bindings);}}
}

fn compact_context(context:&Value,input_type:&str)->Value{
 let work_item=context.get("work_item").cloned().unwrap_or(Value::Null);
 let artifacts=context.get("artifacts").and_then(Value::as_array).map(|items|items.iter().filter_map(|artifact|{
  let kind=artifact.get("artifactType").or_else(||artifact.get("artifact_type")).and_then(Value::as_str)?;
  if kind==input_type||kind=="RESEARCH_SIGNAL"{Some(artifact.clone())}else{Some(json!({
   "artifact_id":artifact.get("artifactId").or_else(||artifact.get("artifact_id")),
   "artifact_type":kind,
   "version":artifact.get("version"),
   "producer":artifact.get("producer")
  }))}
 }).collect::<Vec<_>>()).unwrap_or_default();
 json!({"work_item":work_item,"artifacts":artifacts})
}
