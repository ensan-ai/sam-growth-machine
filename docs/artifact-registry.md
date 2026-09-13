# Canonical Artifact Registry

Status: Phase 3A conceptual registry; existing schemas remain authoritative.

## Registry

“Versioned” below means business-content revision identity, not the common schema_version field.

| Artifact | Producer | Primary consumer | Purpose | Business version/revision | Triggers work? |
|---|---|---|---|---|---|
| OPPORTUNITY_CARD | Saly | Travis through batch; Adam through Opportunity Context | Full research evidence, relevance, score, provenance, and recommendation | No explicit version; revised evidence needs a new immutable card identity or future revision rule | Indirectly, when included in a batch |
| OPPORTUNITY_BATCH | Saly | Travis | Curated decision intake compatible with Travis | New input_id for each emitted batch | Yes: Travis prioritization |
| GROWTH_DECISION | Travis | Adam, Saly, or no next owner as declared | Authoritative pursue/defer/reject/request-more-evidence decision | New artifact_id for a changed decision | Pursue may trigger Assignment |
| ASSIGNMENT | Travis | Named employee; Adam in the canonical cycle | Work commitment, objective, source, deliverable, priority, success condition, and approval requirement | New artifact_id for materially changed work | Yes: Adam strategy work |
| CONTENT_BRIEF | Adam | Brain and Jax | Authoritative strategy, audience, role, proof, format, constraints, and success signals | No explicit business version; immutable brief identity/revision rule required before runtime | Yes: Brain and Jax |
| REPURPOSING_MAP | Adam | Brain, Jax, and future assembly | Declares how one insight becomes distinct platform assets | New map_id for a changed map | When applicable |
| CONTENT_DRAFT | Brain | Jax, Sam review, and package assembly | Authoritative public wording | Yes: draft_id plus semantic version | Yes: draft-dependent Jax work and assembly |
| CREATIVE_PACKAGE | Jax | Sam review and package assembly | Authoritative creative plan, assets, proof treatment, and adaptations | creative_package_id exists; intrinsic business version is missing | Yes: assembly when ready |
| ASSET_REQUEST | Jax | Declared destination: Sam, Adam, or system | Requests missing real/generated/designed material without substitution | New asset_request_id per request/revision | Yes: asset fulfillment; blocks readiness when declared |
| NO_VISUAL_REQUIRED | Jax | Sam review and package assembly | Explicit decision that text-only treatment best supports strategy | New decision_id if changed | Yes: permits assembly without Creative Package |
| PUBLISH_PACKAGE | System mechanically; schema also permits Sam or Travis | Sam while awaiting approval; Maro when approved | Binds exact brief, draft, creative snapshot, platforms, metadata, schedule, and approval | Contains content_version, creative_version, and approval_version; new package identity for material reassembly | Yes: Sam review or Maro validation |
| APPROVAL | Sam; system records | Package assembly and Maro | Explicit public authorization for exact versions and scope | approval_id plus approval_version; superseded/revoked state retained | Yes: permits approved package delivery to Maro |
| PUBLICATION_RECEIPT | Maro | Lara and Travis when operationally relevant | Authoritative per-platform attempt/outcome and publication identity | Immutable receipt per recorded outcome; captures content/creative versions | Yes: measurement eligibility or recovery |
| DISTRIBUTION_RESULT | Maro | Lara and Travis | Aggregate multi-platform truth, including partial failure | New distribution_result_id for an updated aggregation | Yes: measurement and target-risk review |
| PERFORMANCE_SNAPSHOT | Future read-only system evidence capture | Lara | Raw observed metrics for one publication and explicit window with provenance/quality | New snapshot_id per window/retrieval; never overwrite historical evidence silently | Yes: Lara analysis |
| PERFORMANCE_INSIGHT | Lara | Travis | Decision-useful interpretation directly compatible with Travis intake | New input_id/insight_reference when evidence or conclusion materially changes | Yes: Travis cycle decision |
| CONTENT_PERFORMANCE_REPORT | Lara | Lara evidence store; Travis/Adam through a compatible PERFORMANCE_INSIGHT or PERFORMANCE_CONTEXT projection | Asset/family-level objective-aware analysis | New report_id for a revised window/evidence set | Indirectly through a compatible envelope |
| CYCLE_PERFORMANCE_REPORT | Lara | Lara evidence store; Travis through PERFORMANCE_INSIGHT | Cycle-level evidence package | cycle_id plus analysis window; new report artifact for revisions | Indirectly: cycle review |
| ANOMALY_ALERT | Lara | Lara evidence store; Travis through PERFORMANCE_INSIGHT or SYSTEM_STATUS | Unusual behavior with evidence, uncertainty, and next check | New alert_id; status may be tracked by future state | Indirectly when material |
| EXPERIMENT_RESULT | Lara | Lara evidence store; Travis through PERFORMANCE_INSIGHT | Evidence-bounded experiment result | experiment_id plus new result artifact when evidence matures | Indirectly: Travis decides next action |
| CYCLE_DECISION | Travis | Relevant specialist(s) | Authoritative DOUBLE_DOWN/CONTINUE/MODIFY/PAUSE/STOP/TEST decision | New artifact_id for a changed decision | Yes: next cycle |

## Minimum APPROVAL artifact

The conceptual canonical record is the same information Maro already requires in approval_reference:

| Field | Rule |
|---|---|
| approval_id | Stable identity for this recorded decision |
| approver | Must be Sam for public content in V1 |
| approved_item | PUBLISH_PACKAGE or consolidated review-package identity |
| approved_content_version | Exact Brain draft version |
| approved_creative_version | Exact immutable creative snapshot version, or null for approved no-visual treatment |
| approved_platforms | Exact allowed destinations |
| approval_scope_id | Stable scope identity covering item, platforms, and approval-sensitive constraints |
| approved_at | Explicit timestamp |
| approval_version | Version of the approval record |
| decision/status | APPROVE, REQUEST_CHANGES, REJECT, or PAUSE; only APPROVE can authorize Maro |
| validity_status | VALID, REVOKED, EXPIRED, or SUPERSEDED |
| supersedes / superseded_by | Optional links preserving history when a later approval replaces this one |

For Maro, APPROVE maps to decision=APPROVE and approver maps to approved_by=sam. REQUEST_CHANGES, REJECT, and PAUSE remain approval-workflow outcomes and must never be projected as a valid Maro approval_reference.

Approval is explicit stored system state. Chat history, memory, silence, prior preference, or Travis direction cannot substitute for it.

## Minimum traceability identity model

Use existing identities and source references; do not add a universal ID to every record.

~~~mermaid
flowchart RL
    I[PERFORMANCE_INSIGHT<br/>insight_reference] --> S[PERFORMANCE_SNAPSHOT<br/>snapshot_id]
    S --> R[PUBLICATION_RECEIPT<br/>receipt_id]
    R --> P[PUBLISH_PACKAGE<br/>publish_package_id]
    P --> AP[APPROVAL<br/>approval_id + approval_version]
    P --> C[CREATIVE_PACKAGE<br/>creative_package_id + snapshot version]
    P --> D[CONTENT_DRAFT<br/>draft_id + version]
    P --> B[CONTENT_BRIEF<br/>brief_id]
    B --> A[ASSIGNMENT<br/>artifact_id]
    A --> G[GROWTH_DECISION<br/>artifact_id]
    G --> O[OPPORTUNITY_CARD<br/>opportunity_id]
~~~

Minimum required chain:

1. opportunity_id
2. growth-decision artifact_id
3. assignment artifact_id
4. brief_id
5. draft_id + draft version
6. creative_package_id + immutable snapshot version, or no-visual decision_id
7. approval_id + approval_version
8. publish_package_id
9. receipt_id; distribution_result_id when aggregate context matters
10. snapshot_id
11. insight_reference

publication_operation_id remains inside the receipt for idempotency and operational diagnosis; it need not be duplicated at every analytical layer. cycle_id groups work but does not replace source links.

## Gaps and ownership findings

- Missing canonical artifact: standalone APPROVAL. Phase 3A defines its minimum concept without adding a schema.
- Missing owned coordination step: PUBLISH_PACKAGE assembly. Assign it to a future non-authoritative system function, initiated by workflow state—not to a new employee.
- External/system-produced evidence: PERFORMANCE_SNAPSHOT is not an employee judgment artifact. The future read-only analytics adapter records observations; Lara interprets them.
- No unavoidable orphan among the reviewed artifacts: OPPORTUNITY_CARD is reachable through OPPORTUNITY_BATCH and Adam's Opportunity Context. Detailed Lara reports/alerts/results must be referenced and summarized through Travis-compatible PERFORMANCE_INSIGHT, or through Adam's PERFORMANCE_CONTEXT where appropriate; they are not direct Travis inputs.
- ASSET_REQUEST declares a destination but has no corresponding employee input or fulfillment receipt. Treat it as a future system/human task until that minimal acceptance/closure contract is defined.
- Maro outputs do not enter Lara directly. A mechanical projection must create Lara's PUBLICATION_RECORD from PUBLICATION_RECEIPT/DISTRIBUTION_RESULT without changing publication truth.
- Unclear revision identity: CONTENT_BRIEF and CREATIVE_PACKAGE lack intrinsic business-version fields. Runtime must either guarantee immutable IDs for every revision or introduce a reviewed compatibility version before approval/package assembly.
- PUBLISH_PACKAGE may be created_by Sam, Travis, or system. The lean canonical path uses system for mechanical assembly, Sam for approval, and Travis for priority—not Travis or Sam manually building payloads.
