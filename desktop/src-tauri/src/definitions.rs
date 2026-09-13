use crate::models::EmployeeSummary;
use serde_json::Value;
use std::{collections::HashMap, fs, path::{Path, PathBuf}};

#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub summary: EmployeeSummary,
    pub contract: String,
    pub prompt: String,
    pub input_schema: Value,
    pub output_schema: Value,
}

#[derive(Debug, Clone)]
pub struct DefinitionStore {
    agents: HashMap<String, AgentDefinition>,
}

impl DefinitionStore {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref().to_path_buf();
        let mut agents = HashMap::new();
        for id in ["travis", "saly", "adam", "brain", "jax", "maro", "lara"] {
            let dir = root.join("agents").join(id);
            let manifest_text = fs::read_to_string(dir.join("manifest.yaml"))
                .map_err(|e| format!("Cannot load {id} manifest: {e}"))?;
            let manifest: serde_yaml::Value = serde_yaml::from_str(&manifest_text)
                .map_err(|e| format!("Invalid {id} manifest: {e}"))?;
            let identity = manifest.get("identity").ok_or_else(|| format!("{id} identity missing"))?;
            let organization = manifest.get("organization").ok_or_else(|| format!("{id} organization missing"))?;
            let string_at = |parent: &serde_yaml::Value, key: &str| {
                parent.get(key).and_then(|v| v.as_str()).unwrap_or_default().to_string()
            };
            let summary = EmployeeSummary {
                employee_id: string_at(identity, "employee_id"),
                name: string_at(identity, "name"),
                title: string_at(identity, "title"),
                reports_to: string_at(organization, "reports_to"),
                version: string_at(identity, "version"),
                definition_status: manifest.get("definition_status").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                runtime_status: "IDLE".into(),
            };
            if summary.employee_id != id || summary.version != "1.0.0" || summary.definition_status != "reviewable" {
                return Err(format!("Employee {id} is not a reviewable 1.0.0 definition"));
            }
            let contract = fs::read_to_string(dir.join("contract.md"))
                .map_err(|e| format!("Cannot load {id} contract: {e}"))?;
            let prompt = fs::read_to_string(dir.join("prompt.md"))
                .map_err(|e| format!("Cannot load {id} prompt: {e}"))?;
            let input_schema = read_json(dir.join("schemas/input.schema.json"))?;
            let output_schema = read_json(dir.join("schemas/output.schema.json"))?;
            agents.insert(id.to_string(), AgentDefinition { summary, contract, prompt, input_schema, output_schema });
        }
        Ok(Self { agents })
    }

    pub fn get(&self, id: &str) -> Result<&AgentDefinition, String> {
        self.agents.get(id).ok_or_else(|| format!("Unknown employee: {id}"))
    }

    pub fn employees(&self) -> Vec<EmployeeSummary> {
        let order = ["travis", "saly", "adam", "brain", "jax", "maro", "lara"];
        order.iter().filter_map(|id| self.agents.get(*id).map(|d| d.summary.clone())).collect()
    }

    pub fn schema_resources(&self) -> HashMap<String, Value> {
        let mut resources = HashMap::new();
        for definition in self.agents.values() {
            for schema in [&definition.input_schema, &definition.output_schema] {
                if let Some(id) = schema.get("$id").and_then(Value::as_str) {
                    resources.insert(id.to_string(), schema.clone());
                }
            }
        }
        resources
    }

}

fn read_json(path: PathBuf) -> Result<Value, String> {
    let text = fs::read_to_string(&path).map_err(|e| format!("Cannot load {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("Invalid JSON {}: {e}", path.display()))
}

pub fn repository_root() -> Result<PathBuf, String> {
    if let Ok(root) = std::env::var("SAM_GROWTH_MACHINE_ROOT") {
        let path = PathBuf::from(root);
        if path.join("agents").is_dir() { return Ok(path); }
    }
    let from_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    from_manifest.canonicalize().map_err(|e| format!("Cannot resolve repository root: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_authoritative_employees() {
        let store = DefinitionStore::load(repository_root().unwrap()).unwrap();
        assert_eq!(store.employees().len(), 7);
        assert_eq!(store.get("brain").unwrap().summary.reports_to, "adam");
        assert_eq!(store.get("saly").unwrap().summary.reports_to, "travis");
    }
}
