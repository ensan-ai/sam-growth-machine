use chrono::Utc;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

pub fn generate_for_type(schema: &Value, desired_type: &str) -> Value {
    generate_for_type_with_resources(schema, desired_type, &HashMap::new())
}

pub fn generate_for_type_with_resources(schema: &Value, desired_type: &str, resources: &HashMap<String, Value>) -> Value {
    generate_node(schema, choose_branch(schema, schema, desired_type, resources), desired_type, "root", 0, resources)
}

fn choose_branch<'a>(root: &'a Value, node: &'a Value, desired: &str, resources: &'a HashMap<String, Value>) -> &'a Value {
    let resolved = resolve_ref(root, node, resources).unwrap_or(node);
    if let Some(branches) = resolved.get("oneOf").or_else(|| resolved.get("anyOf")).and_then(Value::as_array) {
        for branch in branches {
            let candidate = resolve_ref(root, branch, resources).unwrap_or(branch);
            if serde_json::to_string(candidate).unwrap_or_default().contains(desired) {
                let external = branch.get("$ref").and_then(Value::as_str).map(|r| !r.starts_with('#')).unwrap_or(false);
                return if external { branch } else { candidate };
            }
        }
        if let Some(first) = branches.first() {
            let external = first.get("$ref").and_then(Value::as_str).map(|r| !r.starts_with('#')).unwrap_or(false);
            return if external { first } else { resolve_ref(root, first, resources).unwrap_or(first) };
        }
    }
    resolved
}

fn resolve_ref<'a>(root: &'a Value, node: &'a Value, resources: &'a HashMap<String, Value>) -> Option<&'a Value> {
    let reference = node.get("$ref")?.as_str()?;
    if let Some(pointer) = reference.strip_prefix('#') { return root.pointer(pointer); }
    let (uri, fragment) = reference.split_once('#').unwrap_or((reference, ""));
    let resource = resources.get(uri)?;
    if fragment.is_empty() { Some(resource) } else { resource.pointer(fragment) }
}

fn generate_node(root: &Value, raw: &Value, desired: &str, key: &str, depth: usize, resources: &HashMap<String, Value>) -> Value {
    if depth > 20 { return Value::Null; }
    if let Some(reference) = raw.get("$ref").and_then(Value::as_str).filter(|r| !r.starts_with('#')) {
        let (uri, fragment) = reference.split_once('#').unwrap_or((reference, ""));
        if let Some(resource) = resources.get(uri) {
            let target = if fragment.is_empty() { resource } else { resource.pointer(fragment).unwrap_or(resource) };
            return generate_node(resource, target, desired, key, depth + 1, resources);
        }
    }
    let node = resolve_ref(root, raw, resources).unwrap_or(raw);
    if let Some(reference) = node.get("$ref").and_then(Value::as_str).filter(|r| !r.starts_with('#')) {
        let (uri, fragment) = reference.split_once('#').unwrap_or((reference, ""));
        if let Some(resource) = resources.get(uri) {
            let target = if fragment.is_empty() { resource } else { resource.pointer(fragment).unwrap_or(resource) };
            return generate_node(resource, target, desired, key, depth + 1, resources);
        }
    }
    if desired == "PUBLICATION_RECEIPT" {
        if key == "status" { return json!("PUBLISHED"); }
        if key == "confirmation_basis" { return json!("PLATFORM_CONFIRMED_WITHOUT_ID"); }
        if key == "published_at" { return json!(Utc::now().to_rfc3339()); }
        if key == "error_category" { return Value::Null; }
    }
    if desired == "DISTRIBUTION_RESULT" {
        if key == "overall_status" || key == "status" { return json!("PUBLISHED"); }
        if key == "error_category" { return Value::Null; }
        if key == "partial_failure_detected" || key == "retryable" { return json!(false); }
        if key == "published_count" || key == "target_count" { return json!(1); }
        if key == "failed_or_blocked_count" { return json!(0); }
    }
    if desired == "PUBLISH_PACKAGE" {
        if key == "package_state" { return json!("APPROVED"); }
        if key == "cancelled" { return json!(false); }
        if key == "approval_version" { return json!("1.0.0"); }
        if key == "content_type" { return json!("LINKEDIN_POST"); }
    }
    if let Some(v) = node.get("const") { return v.clone(); }
    if let Some(v) = node.get("default") { return v.clone(); }
    if let Some(v) = node.get("examples").and_then(Value::as_array).and_then(|a| a.first()) { return v.clone(); }
    if let Some(v) = node.get("enum").and_then(Value::as_array).and_then(|a| a.first()) { return v.clone(); }
    if let Some(branches) = node.get("oneOf").or_else(|| node.get("anyOf")).and_then(Value::as_array) {
        let branch = branches.iter().find(|b| {
            let candidate = resolve_ref(root, b, resources).unwrap_or(b);
            serde_json::to_string(candidate).unwrap_or_default().contains(desired)
        }).or_else(|| branches.iter().find(|b| type_name(resolve_ref(root, b, resources).unwrap_or(b)) != "null")).unwrap_or(&branches[0]);
        return generate_node(root, branch, desired, key, depth + 1, resources);
    }
    if node.get("properties").is_none() && node.get("type").is_none() {
      if let Some(parts) = node.get("allOf").and_then(Value::as_array) {
        let mut merged = Map::new();
        for part in parts {
            if let Value::Object(values) = generate_node(root, part, desired, key, depth + 1, resources) {
                merge_objects(&mut merged, values);
            }
        }
        return Value::Object(merged);
      }
    }
    if node.get("type").and_then(Value::as_array).map(|types| types.iter().any(|v| v.as_str() == Some("null"))).unwrap_or(false) {
        return Value::Null;
    }
    match type_name(node) {
        "object" => {
            let mut out = Map::new();
            let properties = node.get("properties").and_then(Value::as_object);
            let required = node.get("required").and_then(Value::as_array).cloned().unwrap_or_default();
            for required_key in required.iter().filter_map(Value::as_str) {
                if let Some(prop) = properties.and_then(|p| p.get(required_key)) {
                    let value = if required_key == "artifact_type" || required_key == "input_type" || required_key == "output_type" {
                        choose_type_value(prop, desired).unwrap_or_else(|| generate_node(root, prop, desired, required_key, depth + 1, resources))
                    } else {
                        generate_node(root, prop, desired, required_key, depth + 1, resources)
                    };
                    out.insert(required_key.to_string(), value);
                }
            }
            if let Some(properties) = properties {
                for (property_key, property_schema) in properties {
                    if !out.contains_key(property_key) && property_schema.get("const").is_some() {
                        out.insert(property_key.clone(), generate_node(root, property_schema, desired, property_key, depth + 1, resources));
                    } else if !out.contains_key(property_key) && property_schema.get("required").is_some() {
                        out.insert(property_key.clone(), generate_node(root, property_schema, desired, property_key, depth + 1, resources));
                    }
                }
            }
            Value::Object(out)
        }
        "array" => {
            let conditional_min = if key.contains("platform") || key == "scenes" { 1 } else { 0 };
            let min = node.get("minItems").and_then(Value::as_u64).unwrap_or(0).max(conditional_min);
            let item = node.get("items").unwrap_or(&Value::Null);
            if desired == "PUBLISH_PACKAGE" && key == "authoritative_artifact_bindings" {
                let mut brief = generate_node(root, item, desired, key, depth + 1, resources);
                let mut draft = brief.clone();
                if let Value::Object(values) = &mut brief { values.insert("artifact_type".into(), json!("CONTENT_BRIEF")); values.insert("artifact_id".into(), json!("brief-fixture-001")); }
                if let Value::Object(values) = &mut draft { values.insert("artifact_type".into(), json!("CONTENT_DRAFT")); values.insert("artifact_id".into(), json!("draft-fixture-001")); }
                return json!([brief, draft]);
            }
            Value::Array((0..min).map(|_| generate_node(root, item, desired, key, depth + 1, resources)).collect())
        }
        "integer" => json!(node.get("minimum").and_then(Value::as_i64).unwrap_or(1)),
        "number" => json!(node.get("minimum").and_then(Value::as_f64).unwrap_or(1.0)),
        "boolean" => json!(key != "required"),
        "null" => Value::Null,
        _ => Value::String(string_value(node, key, desired)),
    }
}

fn merge_objects(target: &mut Map<String, Value>, incoming: Map<String, Value>) {
    for (key, value) in incoming {
        if let (Some(Value::Object(existing)), Value::Object(additional)) = (target.get_mut(&key), &value) {
            merge_objects(existing, additional.clone());
        } else {
            target.insert(key, value);
        }
    }
}

fn choose_type_value(schema: &Value, desired: &str) -> Option<Value> {
    if schema.get("const").and_then(Value::as_str) == Some(desired) { return Some(json!(desired)); }
    if schema.get("enum").and_then(Value::as_array).map(|a| a.iter().any(|v| v.as_str() == Some(desired))).unwrap_or(false) {
        return Some(json!(desired));
    }
    None
}

fn type_name(node: &Value) -> &str {
    if let Some(value) = node.get("type").and_then(Value::as_str) { return value; }
    if node.get("properties").is_some() { return "object"; }
    "string"
}

fn string_value(node: &Value, key: &str, desired: &str) -> String {
    if node.get("format").and_then(Value::as_str) == Some("date-time") { return Utc::now().to_rfc3339(); }
    if node.get("format").and_then(Value::as_str) == Some("date") { return Utc::now().date_naive().to_string(); }
    if key.ends_with("_id") || key == "id" { return format!("{key}-fixture-001"); }
    if key.contains("version") { return "1.0.0".into(); }
    if key.contains("url") { return "https://example.com/evidence".into(); }
    if key.contains("platform") { return "LINKEDIN".into(); }
    let min = node.get("minLength").and_then(Value::as_u64).unwrap_or(1) as usize;
    let mut value = format!("Deterministic {desired} {key}");
    while value.len() < min { value.push_str(" evidence"); }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::{repository_root, DefinitionStore};

    #[test]
    fn generated_outputs_validate_for_each_employee() {
        let store = DefinitionStore::load(repository_root().unwrap()).unwrap();
        let resources = store.schema_resources();
        let cases = [
            ("saly", "OPPORTUNITY_BATCH"), ("travis", "GROWTH_DECISION"),
            ("adam", "CONTENT_BRIEF"), ("brain", "CONTENT_DRAFT"),
            ("jax", "CREATIVE_PACKAGE"), ("maro", "PUBLICATION_RECEIPT"),
            ("lara", "PERFORMANCE_INSIGHT")
        ];
        for (agent, kind) in cases {
            let definition = store.get(agent).unwrap();
            let fixture = generate_for_type_with_resources(&definition.output_schema, kind, &resources);
            let pairs = resources.iter().map(|(uri, value)| (uri.clone(), jsonschema::Resource::from_contents(value.clone()).unwrap()));
            let validator = jsonschema::options().with_resources(pairs).build(&definition.output_schema).unwrap();
            let errors: Vec<String> = validator.iter_errors(&fixture).map(|e| e.to_string()).collect();
            assert!(errors.is_empty(), "{agent}/{kind} invalid: {} :: {fixture}", errors.join("; "));
        }
    }
}
