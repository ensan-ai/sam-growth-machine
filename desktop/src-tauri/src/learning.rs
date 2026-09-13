use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::models::LearningRecord;

pub fn fingerprint(signal: &str) -> String {
    let normalized: String = signal
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

pub fn excerpt(signal: &str) -> String {
    signal.chars().take(200).collect()
}

pub fn should_reject(records: &[LearningRecord], fp: &str) -> bool {
    records.iter().any(|r| r.opportunity_fingerprint == fp && r.cycle_decision == "STOP")
}

pub fn should_double_down(records: &[LearningRecord], fp: &str) -> bool {
    records
        .iter()
        .any(|r| r.opportunity_fingerprint == fp && r.cycle_decision == "DOUBLE_DOWN" && r.data_quality == "COMPLETE")
}

pub fn context_payload(records: &[LearningRecord]) -> Value {
    json!({
        "artifact_type": "LEARNING_CONTEXT",
        "created_by": "system",
        "records": records.iter().map(|r| json!({
            "learning_id": r.learning_id,
            "source_work_item_id": r.source_work_item_id,
            "opportunity_fingerprint": r.opportunity_fingerprint,
            "cycle_decision": r.cycle_decision,
            "data_quality": r.data_quality,
            "recommended_attention": r.recommended_attention,
            "constraints_for_next": r.payload.get("constraints_for_next"),
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_signal_same_fingerprint() {
        assert_eq!(fingerprint("Hello World"), fingerprint("hello   world"));
        assert_ne!(fingerprint("Hello World"), fingerprint("other"));
    }
}
