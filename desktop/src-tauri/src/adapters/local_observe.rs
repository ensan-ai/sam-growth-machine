use serde_json::{json, Value};

/// Default slice observe is UNAVAILABLE. COMPLETE is tests-only and must be mocked.
pub fn snapshot(receipt_id: &str, quality: &str) -> Value {
    let complete = quality == "COMPLETE";
    json!({
        "artifact_type": "PERFORMANCE_SNAPSHOT",
        "publication_receipt_id": receipt_id,
        "measurement_window": "EARLY",
        "data_quality": if complete { "COMPLETE" } else { "UNAVAILABLE" },
        "metrics": if complete {
            json!({"impressions": 12, "qualified_engagements": 1, "profile_visits": 1, "saves": 0, "comments": 0})
        } else {
            json!({})
        },
        "provenance": if complete { "LOCAL_ADAPTER_SYNTHETIC" } else { "LOCAL_ADAPTER" },
        "mocked": complete,
        "execution_target": "LOCAL_LEDGER",
        "intended_platforms": ["LINKEDIN"],
    })
}
