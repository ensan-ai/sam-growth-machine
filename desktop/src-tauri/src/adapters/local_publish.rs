use serde_json::{json, Value};

/// Local ledger adapter. execution_target = LOCAL_LEDGER. Does not publish to LinkedIn.
pub fn idempotency_key(work_item_id: &str, package_id: &str, package_version: i64) -> String {
    format!("{work_item_id}:{package_id}:{package_version}:LOCAL_LEDGER")
}

pub fn receipt(
    work_item_id: &str,
    package_id: &str,
    approval_id: &str,
    content_id: &str,
    visual_id: &str,
    key: &str,
) -> Value {
    json!({
        "schema_version": "1.0.0",
        "artifact_type": "PUBLICATION_RECEIPT",
        "created_by": "maro",
        "work_item_id": work_item_id,
        "publish_package_id": package_id,
        "approval_id": approval_id,
        "content_artifact_id": content_id,
        "visual_artifact_id": visual_id,
        "intended_platforms": ["LINKEDIN"],
        "execution_target": "LOCAL_LEDGER",
        "status": "PUBLISHED",
        "idempotency_key": key,
        "external_reference": format!("ledger://local/{key}"),
        "confirmation_basis": "LOCAL_LEDGER_WRITE",
        "external_api": false,
    })
}

pub fn distribution_result(work_item_id: &str, receipt_id: &str, key: &str) -> Value {
    json!({
        "schema_version": "1.0.0",
        "artifact_type": "DISTRIBUTION_RESULT",
        "created_by": "maro",
        "work_item_id": work_item_id,
        "receipt_id": receipt_id,
        "intended_platforms": ["LINKEDIN"],
        "execution_target": "LOCAL_LEDGER",
        "overall_status": "PUBLISHED",
        "published_count": 1,
        "target_count": 1,
        "failed_or_blocked_count": 0,
        "partial_failure_detected": false,
        "idempotency_key": key,
        "external_api": false,
    })
}
