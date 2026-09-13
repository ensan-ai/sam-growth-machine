use serde_json::{json, Value};

use crate::capabilities::{EXECUTION_TARGET, INTENDED_PLATFORM};
use crate::learning::{excerpt, fingerprint};
use crate::models::LearningRecord;

pub fn signal_text(payload: &Value) -> String {
    payload
        .get("signal")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub fn stamp(record_id: &str, version: i64, mut payload: Value) -> Value {
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("artifact_id".into(), json!(record_id));
        obj.insert("version".into(), json!(version));
    }
    payload
}

pub fn package_opportunity(work_item_id: &str, signal: &Value, learning: &[LearningRecord]) -> (Value, Value) {
    let text = signal_text(signal);
    let fp = fingerprint(&text);
    let stopped = learning.iter().any(|r| r.opportunity_fingerprint == fp && r.cycle_decision == "STOP");
    let card_id = format!("opportunity-{work_item_id}");
    let card = json!({
        "schema_version": "1.0.0",
        "artifact_type": "OPPORTUNITY_CARD",
        "created_by": "saly",
        "source_input_references": [signal.get("artifact_id").and_then(Value::as_str).unwrap_or(work_item_id)],
        "opportunity_id": card_id,
        "working_title": format!("Practical AI: {text}"),
        "audience_problem": text,
        "core_idea": text,
        "recommendation": if stopped { "WATCH" } else { "NOMINATE" },
        "opportunity_fingerprint": fp,
        "evidence": [{
            "evidence_id": format!("evidence-{work_item_id}"),
            "statement": text,
            "evidence_type": "INFERENCE",
            "source_ids": [format!("src-{work_item_id}")],
            "verification_status": "NOT_APPLICABLE"
        }],
        "sources": [{
            "source_id": format!("src-{work_item_id}"),
            "source_type": "HUMAN_SIGNAL",
            "source_class": "INTERNAL",
            "title": "Sam research signal",
            "locator": format!("governed://work/{work_item_id}/signal"),
            "relevance_note": "Human-provided research signal; not a primary web source."
        }],
        "semantic_duplicate": stopped,
        "intended_platforms": [INTENDED_PLATFORM],
    });
    let batch = json!({
        "schema_version": "1.0.0",
        "artifact_type": "OPPORTUNITY_BATCH",
        "created_by": "saly",
        "source_input_references": [card_id],
        "opportunity_ids": [card_id],
        "cards": [card_id],
        "opportunity_fingerprint": fp,
        "intended_platforms": [INTENDED_PLATFORM],
    });
    (card, batch)
}

pub fn prioritize_and_assign(
    work_item_id: &str,
    card: &Value,
    learning: &[LearningRecord],
) -> Result<(Value, Option<Value>), String> {
    let fp = card
        .get("opportunity_fingerprint")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let idea = card.get("core_idea").and_then(Value::as_str).unwrap_or_default();
    let card_id = card.get("opportunity_id").and_then(Value::as_str).unwrap_or_default();
    let reject = crate::learning::should_reject(learning, &fp);
    let double = crate::learning::should_double_down(learning, &fp);
    let need_more = learning.iter().any(|r| {
        r.opportunity_fingerprint == fp && r.recommended_attention.as_deref() == Some("NEED_MORE_DATA")
    });
    if reject {
        let decision = json!({
            "schema_version": "1.0.0",
            "artifact_type": "GROWTH_DECISION",
            "created_by": "travis",
            "source_input_references": [card_id],
            "opportunity_reference": card_id,
            "decision": "reject",
            "rationale": format!("Rejected by learning policy for fingerprint {fp}"),
            "priority": "none",
            "intended_outcome": "Do not rediscover this opportunity without new evidence",
            "confidence": {"assessment": "high", "basis": "negative LEARNING_RECORD"},
            "next_destination": "none",
            "opportunity_fingerprint": fp,
        });
        return Ok((decision, None));
    }
    let priority = if double { "high" } else { "normal" };
    let context = if need_more {
        "measurement immature; do not claim results"
    } else {
        "pursue within Practical AI for Real Work"
    };
    let decision = json!({
        "schema_version": "1.0.0",
        "artifact_type": "GROWTH_DECISION",
        "created_by": "travis",
        "source_input_references": [card_id],
        "opportunity_reference": card_id,
        "decision": "pursue",
        "rationale": format!("Pursue signal: {idea}"),
        "priority": priority,
        "intended_outcome": "Produce one LinkedIn-ready package for Sam approval",
        "confidence": {"assessment": "medium", "basis": "deterministic policy"},
        "next_destination": "adam",
        "opportunity_fingerprint": fp,
    });
    let assignment = json!({
        "schema_version": "1.0.0",
        "artifact_type": "ASSIGNMENT",
        "created_by": "travis",
        "source_input_references": [card_id],
        "recipient": "adam",
        "objective": format!("Turn this opportunity into a LinkedIn brief: {idea}"),
        "source_artifact_reference": card_id,
        "expected_deliverable": "CONTENT_BRIEF",
        "priority": priority,
        "deadline_or_window": Value::Null,
        "success_condition": "Brief preserves the research signal and intended LinkedIn surface",
        "approval_requirement": {"required": true, "approver": "sam"},
        "context": format!("{context} work_item={work_item_id}"),
        "opportunity_fingerprint": fp,
        "intended_platforms": [INTENDED_PLATFORM],
    });
    Ok((decision, Some(assignment)))
}

pub fn compile_brief(assignment: &Value, card: &Value, revision: Option<&Value>) -> Value {
    let idea = card.get("core_idea").and_then(Value::as_str).unwrap_or_default();
    let assignment_id = assignment
        .get("artifact_id")
        .and_then(Value::as_str)
        .or_else(|| assignment.get("source_artifact_reference").and_then(Value::as_str))
        .unwrap_or_default();
    let card_id = card.get("opportunity_id").and_then(Value::as_str).unwrap_or_default();
    let mut core = idea.to_string();
    if let Some(rev) = revision {
        if let Some(fb) = rev.get("feedback").and_then(Value::as_str) {
            core = format!("{core} [revision: {fb}]");
        }
    }
    json!({
        "schema_version": "1.0.0",
        "artifact_type": "CONTENT_BRIEF",
        "created_by": "adam",
        "source_input_references": [assignment_id, card_id],
        "brief_id": format!("brief-{card_id}"),
        "source_assignment_id": assignment_id,
        "source_opportunity_id": card_id,
        "working_title": card.get("working_title").cloned().unwrap_or(json!(idea)),
        "target_audience": "Operators building AI-assisted workflows",
        "audience_problem": idea,
        "strategic_objective": "Explain a practical AI working pattern",
        "content_role": "AUTHORITY",
        "core_idea": core,
        "core_takeaway": idea,
        "sam_unique_angle": "Show the workflow with a human approval gate",
        "why_this_matters_now": idea,
        "evidence_requirements": [{"requirement_id": "ev-1", "requirement": idea, "mandatory": true, "source_references": [card_id]}],
        "demonstration_pattern": "Contrast ungoverned automation with a version-bound approval gate",
        "format": {"type": "LINKEDIN_POST", "extension": Value::Null},
        "target_platforms": [INTENDED_PLATFORM],
        "intended_platforms": [INTENDED_PLATFORM],
        "hook_direction": format!("Lead with: {idea}"),
        "key_points": [idea, "Human approval belongs at consequential decisions"],
        "limitation_or_caveat": "Slice uses a local execution adapter, not a live LinkedIn API",
        "cta_objective": "Inspect one workflow for its approval boundary",
        "creative_requirements": {"creative_objective": "Text-first LinkedIn post", "proof_or_demo_requirement": "No visual required in this slice"},
        "writing_requirements": {"voice": "SAM_BASE", "preserve_uncertainty": true},
        "success_signals": ["qualified conversation", "save"],
        "priority": assignment.get("priority").cloned().unwrap_or(json!("normal")),
        "confidence": {"assessment": "medium", "basis": "compiled from opportunity"},
        "risks": ["local adapter is not public proof"],
        "source_references": [card_id],
        "quality_gate": {
            "positioning_fit": true,
            "audience_problem_clear": true,
            "takeaway_specific": true,
            "sam_adds_unique_value": true,
            "non_duplicate_angle": true,
            "format_matches_idea": true,
            "evidence_requirements_clear": true,
            "intended_outcome_clear": true,
            "reason_beyond_quota": true
        }
    })
}

pub fn write_public_copy(brief: &Value, revision: Option<&Value>) -> Value {
    let idea = brief.get("core_idea").and_then(Value::as_str).unwrap_or_default();
    let takeaway = brief.get("core_takeaway").and_then(Value::as_str).unwrap_or(idea);
    let mut body = format!("{idea}\n\n{takeaway}\n\nPrepared for LinkedIn. Execution will use a local ledger until a LinkedIn adapter exists.");
    if let Some(rev) = revision {
        if let Some(fb) = rev.get("feedback").and_then(Value::as_str) {
            body = format!("{body}\n\nRevision guidance: {fb}");
        }
    }
    let brief_id = brief.get("brief_id").and_then(Value::as_str).unwrap_or("brief");
    json!({
        "schema_version": "1.0.0",
        "artifact_type": "CONTENT_DRAFT",
        "created_by": "brain",
        "source_input_references": [brief_id],
        "content_brief_id": brief_id,
        "title": brief.get("working_title"),
        "hook": brief.get("hook_direction"),
        "body": body,
        "cta": brief.get("cta_objective"),
        "core_idea": idea,
        "intended_platforms": [INTENDED_PLATFORM],
        "format": {"type": "LINKEDIN_POST"},
        "claims_requiring_verification": [],
    })
}

pub fn decide_creative(brief: &Value, draft: &Value) -> Value {
    let brief_id = brief.get("brief_id").and_then(Value::as_str).unwrap_or("brief");
    let draft_id = draft
        .get("artifact_id")
        .and_then(Value::as_str)
        .or_else(|| draft.get("content_brief_id").and_then(Value::as_str));
    json!({
        "schema_version": "1.0.0",
        "artifact_type": "NO_VISUAL_REQUIRED",
        "created_by": "jax",
        "source_input_references": [brief_id],
        "decision_id": format!("novisual-{brief_id}"),
        "brief_id": brief_id,
        "draft_id": draft_id,
        "target_platforms": [INTENDED_PLATFORM],
        "intended_platforms": [INTENDED_PLATFORM],
        "visual_decision": "NO_VISUAL_REQUIRED",
        "reasoning": "Slice has no captured proof assets; text-only LinkedIn treatment preserves REAL PROOF > DECORATION.",
        "strategy_supported": true,
        "approval_status": "REQUIRES_SAM_APPROVAL",
    })
}

pub fn interpret_performance(snapshot: &Value, draft: &Value, receipt: &Value) -> Value {
    let quality = snapshot.get("data_quality").and_then(Value::as_str).unwrap_or("UNAVAILABLE");
    let unavailable = quality != "COMPLETE";
    let attention = if unavailable { "NEED_MORE_DATA" } else { "WEAK_SIGNAL" };
    json!({
        "schema_version": "1.0.0",
        "artifact_type": "PERFORMANCE_INSIGHT",
        "created_by": "lara",
        "source_input_references": [
            snapshot.get("publication_receipt_id").cloned().unwrap_or(json!("snapshot")),
        ],
        "input_type": "PERFORMANCE_INSIGHT",
        "source": "lara",
        "data_quality": quality,
        "recommended_attention": attention,
        "intended_platforms": [INTENDED_PLATFORM],
        "execution_target": EXECUTION_TARGET,
        "core_idea": draft.get("core_idea"),
        "payload": {
            "extensions": {
                "lara_analysis": {
                    "intended_objective": draft.get("cta").cloned().unwrap_or(json!("qualified conversation")),
                    "actual_outcome": if unavailable {
                        "No platform measurement is available; local ledger execution is not LinkedIn analytics."
                    } else {
                        "Synthetic local observation only; not a real platform measurement."
                    },
                    "recommended_attention": attention,
                    "what_can_be_concluded": [if unavailable { "Publication was recorded on LOCAL_LEDGER" } else { "Synthetic metrics are test-only" }],
                    "what_cannot_be_concluded": ["Causality", "LinkedIn audience quality"],
                    "uncertainty": ["Observe adapter is not a social analytics provider"],
                    "evidence": [{
                        "evidence_id": "obs-1",
                        "statement": "Observation only; no causal claim.",
                        "evidence_type": "OBSERVATION",
                        "source_reference": receipt.get("external_reference").cloned().unwrap_or(json!("ledger")),
                        "data_quality": quality
                    }]
                }
            }
        }
    })
}

pub fn decide_next_cycle(insight: &Value, card: &Value) -> Value {
    let attention = insight.get("recommended_attention").and_then(Value::as_str).unwrap_or("NEED_MORE_DATA");
    let quality = insight.get("data_quality").and_then(Value::as_str).unwrap_or("UNAVAILABLE");
    let decision = if quality == "COMPLETE" && attention == "STRONG_SIGNAL" {
        "DOUBLE_DOWN"
    } else {
        "CONTINUE"
    };
    json!({
        "schema_version": "1.0.0",
        "artifact_type": "CYCLE_DECISION",
        "created_by": "travis",
        "source_input_references": [insight.get("artifact_id").cloned().unwrap_or(json!("insight"))],
        "performance_insight_reference": insight.get("artifact_id"),
        "decision": decision,
        "evidence": [format!("lara:{attention}")],
        "rationale": format!("Cycle decision from {attention} / {quality}"),
        "next_action": if decision == "CONTINUE" { "Run the next signal with learning context applied" } else { "Increase attention on this fingerprint" },
        "opportunity_fingerprint": card.get("opportunity_fingerprint"),
        "data_quality": quality,
        "recommended_attention": attention,
    })
}

fn normalize_cycle_decision(raw: &str) -> &'static str {
    match raw {
        "reject" | "STOP" | "stop" => "STOP",
        "DOUBLE_DOWN" => "DOUBLE_DOWN",
        "MODIFY" => "MODIFY",
        "PAUSE" => "PAUSE",
        "TEST" => "TEST",
        _ => "CONTINUE",
    }
}

pub fn learning_record(
    work_item_id: &str,
    signal: &str,
    cycle: &Value,
    insight: Option<&Value>,
    extra_refs: &[&str],
) -> Value {
    let fp = cycle
        .get("opportunity_fingerprint")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| fingerprint(signal));
    let decision = normalize_cycle_decision(cycle.get("decision").and_then(Value::as_str).unwrap_or("STOP"));
    let quality = insight
        .and_then(|i| i.get("data_quality").and_then(Value::as_str))
        .or_else(|| cycle.get("data_quality").and_then(Value::as_str))
        .unwrap_or("UNAVAILABLE");
    let attention = insight
        .and_then(|i| i.get("recommended_attention").and_then(Value::as_str))
        .or_else(|| cycle.get("recommended_attention").and_then(Value::as_str))
        .unwrap_or(if decision == "STOP" { "NEGATIVE_SIGNAL" } else { "NEED_MORE_DATA" });
    let refs: Vec<Value> = if extra_refs.is_empty() {
        vec![json!(work_item_id)]
    } else {
        extra_refs.iter().map(|r| json!(*r)).collect()
    };
    json!({
        "schema_version": "1.0.0",
        "artifact_type": "LEARNING_RECORD",
        "created_by": "system",
        "owner_role": "travis",
        "source_work_item_id": work_item_id,
        "source_input_references": refs,
        "opportunity_fingerprint": fp,
        "signal_excerpt": excerpt(signal),
        "cycle_decision": decision,
        "data_quality": quality,
        "recommended_attention": attention,
        "constraints_for_next": {
            "avoid_angles": if decision == "STOP" { vec![excerpt(signal)] } else { Vec::<String>::new() },
            "prefer_angles": if decision == "DOUBLE_DOWN" { vec![excerpt(signal)] } else { Vec::<String>::new() },
            "require_more_evidence": attention == "NEED_MORE_DATA",
            "do_not_treat_as_causal": quality != "COMPLETE"
        },
        "next_action": cycle.get("next_action").cloned().unwrap_or(json!("apply learning context")),
    })
}

pub fn revision_instruction(feedback: &str, target: &str) -> Value {
    json!({
        "artifact_type": "REVISION_INSTRUCTION",
        "created_by": "system",
        "feedback": feedback,
        "target_owner": target,
        "intended_platforms": [INTENDED_PLATFORM],
    })
}

pub fn publish_package(id: &str, brief: &Value, draft: &Value, visual: &Value, execution_target: &str) -> Value {
    json!({
        "artifact_type": "PUBLISH_PACKAGE",
        "work_item_id": id,
        "content_brief": {"artifact_id": brief.get("artifact_id"), "version": brief.get("version")},
        "content_draft": {"artifact_id": draft.get("artifact_id"), "version": draft.get("version")},
        "creative_package": Value::Null,
        "no_visual_decision": {
            "artifact_id": visual.get("artifact_id"),
            "version": visual.get("version"),
            "compatible_draft_id": draft.get("artifact_id")
        },
        "intended_platforms": [INTENDED_PLATFORM],
        "target_platforms": [INTENDED_PLATFORM],
        "execution_target": execution_target,
        "asset_references": [],
        "required_metadata": {"title": "Practical AI for Real Work"},
        "approval_sensitive_flags": ["PUBLIC_CONTENT"],
        "status": "AWAITING_APPROVAL"
    })
}
