use crate::models::RuntimeSettings;

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ProviderChoice{Ollama,OpenAi,Mock}

#[derive(Debug,Clone)]
pub struct Route{pub primary:ProviderChoice,pub may_escalate:bool}

pub fn route(task_type:&str,settings:&RuntimeSettings,openai_configured:bool,test_mode:bool)->Route{
    if test_mode{return Route{primary:ProviderChoice::Mock,may_escalate:false};}
    let quality_critical=matches!(task_type,"strategy"|"public_writing"|"complex_reasoning"|"conflicting_evidence"|"brand_critical");
    if quality_critical&&settings.allow_openai_escalation&&openai_configured { Route{primary:ProviderChoice::OpenAi,may_escalate:false} }
    else { Route{primary:ProviderChoice::Ollama,may_escalate:settings.allow_openai_escalation&&openai_configured} }
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn local_first_and_explicit_paid_escalation(){let mut s=RuntimeSettings::default();assert_eq!(route("classification",&s,true,false).primary,ProviderChoice::Ollama);assert_eq!(route("public_writing",&s,true,false).primary,ProviderChoice::Ollama);s.allow_openai_escalation=true;assert_eq!(route("public_writing",&s,true,false).primary,ProviderChoice::OpenAi);let fallback=route("classification",&s,true,false);assert_eq!(fallback.primary,ProviderChoice::Ollama);assert!(fallback.may_escalate);assert_eq!(route("classification",&s,true,true).primary,ProviderChoice::Mock);}
}
