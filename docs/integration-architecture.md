# Phase 3A Lean Integration Architecture

Status: conceptual integration review only; no runtime implementation.

## Integration verdict

The seven employee contracts form a coherent company loop if two coordination boundaries are made explicit before runtime:

1. a non-authoritative system assembly step builds PUBLISH_PACKAGE from the ready, immutable Brain and Jax outputs; and
2. Sam's decision is stored as an explicit APPROVAL record and projected into Maro's existing approval_reference.

Neither boundary requires an eighth employee or a completed-employee schema change. The system may assemble, validate, route, and record; it may not invent content, creative, approval, or strategy.

## End-to-end workflow

~~~mermaid
flowchart LR
    S[Saly<br/>Opportunity evidence] --> T1[Travis<br/>Pursue decision + assignment]
    T1 --> A[Adam<br/>Content Brief]
    A --> B[Brain<br/>Content Draft]
    A --> J0[Jax<br/>Early proof and asset planning]
    B --> J1[Jax<br/>Final Creative Package]
    J0 --> J1
    B --> X[System assembly<br/>Publish Package]
    J1 --> X
    X --> H{Sam approval}
    H -->|Approve exact versions and scope| M[Maro<br/>Publish]
    H -->|Request changes| R[Targeted revision]
    R --> B
    R --> J0
    M --> P[Publication Receipt<br/>Distribution Result]
    P --> D[System evidence capture<br/>Performance Snapshot]
    D --> L[Lara<br/>Performance Insight]
    L --> T2[Travis<br/>Cycle Decision]
    T2 --> S
~~~

## Reporting hierarchy

~~~mermaid
flowchart TD
    SAM[Sam<br/>Founder / CEO / Human Authority] --> TRAVIS[Travis<br/>Main Agent / Growth Director]
    TRAVIS --> SALY[Saly]
    TRAVIS --> ADAM[Adam]
    TRAVIS --> MARO[Maro]
    TRAVIS --> LARA[Lara]
    ADAM --> BRAIN[Brain]
    ADAM --> JAX[Jax]
~~~

Workflow arrows are handoffs, not management relationships.

## Minimum lifecycle for one opportunity

| Stage | Owner | Trigger | Required input | Output | Next recipient | Blocks when | Sam required? |
|---|---|---|---|---|---|---|---|
| Discover | Saly | Research assignment, discovery event, or approved discovery window | Research scope and evidence sources | OPPORTUNITY_CARD; curated OPPORTUNITY_BATCH | Travis | Evidence, relevance, provenance, or novelty is insufficient | No |
| Commit | Travis | Valid OPPORTUNITY_BATCH | Opportunity evidence plus current goal/capacity | GROWTH_DECISION and ASSIGNMENT for Adam | Adam | Evidence is insufficient, goal is ambiguous, or priority conflicts | Only for material direction outside Travis's authority |
| Strategize | Adam | Pursue decision plus Assignment and full Opportunity Context | Selected opportunity and constraints | CONTENT_BRIEF | Brain and Jax | Weak opportunity, unclear proof, duplicate angle, or strategy conflict | No |
| Create words | Brain | Quality-gated Content Brief | Brief and necessary source context | CONTENT_DRAFT | Jax and package assembly | Unsupported claim, unclear brief, or strategic contradiction | Only for optional direction or unclear Sam feedback |
| Plan/create visual treatment | Jax | Content Brief, then Content Draft when copy-dependent | Brief, available assets, and applicable draft | CREATIVE_PACKAGE or NO_VISUAL_REQUIRED; ASSET_REQUEST when blocked | Package assembly | Missing real asset, unsafe proof, copy/visual conflict, or unjustified complexity | Only for human capture, approval-sensitive ambiguity, or explicit feedback |
| Assemble review package | System, mechanically | Required Brain/Jax outputs are ready | Brief, exact draft version, creative snapshot or no-visual decision, platform metadata | PUBLISH_PACKAGE in AWAITING_APPROVAL | Sam | Missing output, version, asset, metadata, or scope | No authority is exercised; Sam is next |
| Approve | Sam | Complete review package | Exact content, creative, platforms, constraints, and versions | APPROVAL or rejection/change request | System assembly; then Maro if approved | Rejected, paused, superseded, expired, or incomplete approval | Yes |
| Distribute | Maro | APPROVED PUBLISH_PACKAGE is valid and schedule-eligible | Version-bound approval package | PUBLICATION_RECEIPT and DISTRIBUTION_RESULT | Lara via publication context; Travis for blockers | Approval/version mismatch, missing dependency, ambiguity, rejection, or permission failure | No routine intervention |
| Capture performance evidence | System, mechanically | Confirmed publication reaches a measurement checkpoint | Publication truth and read-only metrics | PERFORMANCE_SNAPSHOT | Lara | Publication unclear, analytics unavailable, immature, or definitions unknown | No |
| Interpret | Lara | Publication context plus a valid snapshot/window | Publication Record, Performance Snapshot, Content Context, optional history | PERFORMANCE_INSIGHT and optional reports/alerts | Travis | Data is immature, missing, conflicting, incomparable, or weak | No |
| Decide next cycle | Travis | Decision-useful Lara insight | PERFORMANCE_INSIGHT and current goal/capacity | CYCLE_DECISION and, when needed, new assignments | Relevant specialists | Evidence or authority is insufficient | Only for material brand/goal/target changes |

## Who wakes whom

| Employee | Trigger categories | Minimum wake rule |
|---|---|---|
| Saly | SCHEDULED, HUMAN_REQUEST, EVENT_DRIVEN, SYSTEM_RECOVERY | Start on an approved discovery window, Travis research assignment, material monitored signal, WATCH update, or recovery of interrupted research. |
| Travis | SCHEDULED, HUMAN_REQUEST, UPSTREAM_ARTIFACT, EVENT_DRIVEN, SYSTEM_RECOVERY | Start weekly/daily review, on Sam goal change, valid Saly batch, Lara insight/anomaly, critical blocker, target risk, or recovered company state requiring a decision. |
| Adam | UPSTREAM_ARTIFACT, HUMAN_REQUEST, EVENT_DRIVEN, SYSTEM_RECOVERY | Start only from a pursue Assignment plus opportunity context, applicable Sam direction, performance context, or a downstream strategic concern. |
| Brain | UPSTREAM_ARTIFACT, HUMAN_REQUEST, SYSTEM_RECOVERY | Start from a quality-gated Content Brief; resume from source context or a targeted Adam/Sam revision instruction. |
| Jax | UPSTREAM_ARTIFACT, HUMAN_REQUEST, EVENT_DRIVEN, SYSTEM_RECOVERY | Start early proof/asset planning from the Content Brief; continue when the Brain draft or source assets arrive; resume from targeted feedback or a resolved asset request. |
| Maro | UPSTREAM_ARTIFACT, SCHEDULED, EVENT_DRIVEN, SYSTEM_RECOVERY | Start only when a PUBLISH_PACKAGE is valid and eligible, a platform update arrives, an explicit control event occurs, or an interrupted publication is reconciled. |
| Lara | UPSTREAM_ARTIFACT, SCHEDULED, HUMAN_REQUEST, EVENT_DRIVEN, SYSTEM_RECOVERY | Start on publication context plus an approved measurement checkpoint/snapshot, a cycle review, a material anomaly, Sam/Travis question, or recovered incomplete measurement. |

Exact clock times remain unresolved.

## Brain and Jax dependency

The correct rule is hybrid parallelism:

- Brain and Jax both start from Adam's CONTENT_BRIEF.
- Jax may immediately plan proof, identify real assets, assess complexity, and issue ASSET_REQUEST.
- Brain independently writes the CONTENT_DRAFT.
- Jax must consume the applicable Brain draft before finalizing any copy-dependent scenes, overlays, timing, platform adaptation, or READY_FOR_SAM_REVIEW Creative Package.
- Text-only/no-visual decisions and draft-independent asset discovery need not wait for completed copy.
- Package assembly waits for the final applicable Brain and Jax outputs.

Fully sequential work wastes asset lead time; fully parallel finalization risks copy/visual mismatch.

## Sam intervention map

### Must intervene

- final public approval of exact content/creative versions and platform scope
- material change to Sam's positioning, business goal, permanent target, or brand boundary
- spending, paid campaigns, external contact, account/system changes, or another action outside delegated authority

### May intervene

- initial direction and constraints
- targeted writing/creative feedback
- exceptional brand, factual, legal, rights, or safety ambiguity
- pause, rejection, or revocation of approval

### Should not intervene

- routine research filtering, prioritization within approved goals, strategy drafting, ordinary specialist handoffs
- normal clarification between employees
- asset retrieval that does not require Sam, safe system retry, publication reconciliation, or analytics collection
- deciding whether evidence is complete, interpreting routine metrics, or operating every workflow transition

Sam should receive one consolidated review package, not separately approve Saly, Travis, Adam, Brain, and Jax work.

## Major handoffs

| Sender → artifact → receiver | Required status | Acknowledgement | Rejection/revision path | Failure path |
|---|---|---|---|---|
| Saly → OPPORTUNITY_BATCH → Travis | Curated; no REJECT items; evidence/provenance retained | Required: accepted, deferred, rejected, or more evidence requested | Saly updates evidence/card and emits a new batch identity | WATCH, reject, or blocked research |
| Travis → GROWTH_DECISION + ASSIGNMENT → Adam | Decision=pursue; assignment addressed to Adam | Required: accepted or strategic concern | Adam returns STRATEGIC_CONCERN; Travis revises decision/assignment | Travis pauses or requests more evidence |
| Adam → CONTENT_BRIEF → Brain + Jax | Nine-point quality gate passed | Required independently from Brain and Jax | Clarification/strategic or creative concern returns to Adam; corrected brief is a new immutable revision identity | Production blocked |
| Brain → CONTENT_DRAFT → Jax + assembly | READY_FOR_SAM_REVIEW; claims resolved | Required when draft-dependent creative or assembly begins | Adam/Sam targeted revision creates a new draft version | Claim/clarification/strategy issue blocks |
| Jax → CREATIVE_PACKAGE or NO_VISUAL_REQUIRED → assembly | READY_FOR_SAM_REVIEW or explicit no-visual decision | Required | Concern to Adam/Brain; targeted revision creates a new creative snapshot | ASSET_REQUEST or safety/rights blocker |
| Assembly → PUBLISH_PACKAGE → Sam | AWAITING_APPROVAL; exact source bindings complete | Sam decision is the acknowledgement | Change request routes to the owning specialist; material change creates a new version/snapshot | Package remains blocked |
| Sam → APPROVAL → assembly/Maro | Explicit, current, version- and scope-bound APPROVE | Maro validation acknowledges acceptance or blocker | Any change, revocation, expiry, or supersession invalidates prior approval | No publication |
| Maro → PUBLICATION_RECEIPT + DISTRIBUTION_RESULT → Lara/Travis | Truthful platform-confirmed or explicit failure/ambiguity status | Lara acknowledges ingest; Travis only needs blocker/target-risk notice | Upstream content defect returns to its owner and requires reapproval when material | Reconcile ambiguity; bounded retry only when safe |
| System → PERFORMANCE_SNAPSHOT → Lara | Window and provenance present; quality classified | Lara accepts, rejects, or defers | Correct retrieval/definition; create a new snapshot for a later window | UNAVAILABLE/PARTIAL/STALE/INCONSISTENT |
| Lara → PERFORMANCE_INSIGHT → Travis | Travis-compatible envelope; evidence/confidence/limitations present | Required: consumed into decision or marked insufficient | Lara reanalyzes only with new/corrected evidence | Travis defers decision or requests measurement |

Acknowledgement is logical state, not transport technology.

Lara's CONTENT_PERFORMANCE_REPORT, CYCLE_PERFORMANCE_REPORT, ANOMALY_ALERT, and EXPERIMENT_RESULT are detailed evidence artifacts, not direct Travis inputs. Any one that should wake Travis must be referenced and summarized in a Travis-compatible PERFORMANCE_INSIGHT (or SYSTEM_STATUS for a purely operational alert). Likewise, Maro's outputs reach Lara through a truth-preserving PUBLICATION_RECORD projection.

## Source of truth

| Domain | Authority |
|---|---|
| Research evidence and provenance | Saly's OPPORTUNITY_CARD |
| Growth decision and work commitment | Travis's GROWTH_DECISION and ASSIGNMENT |
| Content strategy and intended success signals | Adam's CONTENT_BRIEF |
| Public wording | Brain's versioned CONTENT_DRAFT |
| Creative plan/assets | Jax's immutable Creative Package snapshot |
| Public approval | Sam's explicit APPROVAL record |
| Publication status and identifiers | Maro's PUBLICATION_RECEIPT; DISTRIBUTION_RESULT for aggregate status |
| Raw observed performance | PERFORMANCE_SNAPSHOT and its provider provenance |
| Interpreted performance truth | Lara's PERFORMANCE_INSIGHT/report, bounded by data quality |
| Next-cycle company action | Travis's CYCLE_DECISION |

## Main failure and revision routes

~~~mermaid
flowchart LR
    S[Saly weak/unverified evidence] -->|WATCH, reject, or request evidence| T[Travis]
    A[Adam finds opportunity weak] -->|STRATEGIC_CONCERN| T
    B[Brain strategy contradiction] -->|STRATEGIC_CONCERN| AD[Adam]
    J[Jax missing real asset] -->|ASSET_REQUEST| AS[System / Adam / Sam only when required]
    H[Sam rejects or requests changes] -->|targeted feedback| O{Owning layer}
    O -->|strategy| AD
    O -->|words| B
    O -->|creative| J
    M[Maro ambiguous result] -->|RECONCILIATION_REQUEST; no blind retry| SYS[System recovery]
    L[Lara missing analytics] -->|UNAVAILABLE / defer| SYS
    SYS -->|material decision blocked| T
~~~

Normal operational failures go to the owning specialist or system recovery. Sam receives only decisions that require human authority.

## Revision rules

- New evidence may create a new Opportunity Card/Batch; Travis then issues a new decision or assignment when commitment changes.
- Strategic changes create a new Content Brief revision identity and invalidate dependent draft/creative/package work.
- Brain revisions increment CONTENT_DRAFT.version.
- Jax revisions require a new immutable creative snapshot identity/version before runtime; the current Creative Package has no intrinsic business-version field.
- Sam feedback routes to Adam for strategy, Brain for words, and Jax for creative.
- Any material change after approval invalidates approval and resumes at AWAITING_APPROVAL after new versions are assembled.
- Maro may make only approved mechanical changes; material upstream fixes resume with the owning specialist and require reapproval.
- Lara corrections create new snapshots or analytical artifact IDs; Lara never rewrites strategy. Travis decides the next action.

## Integration eval plan

Document for later implementation testing:

1. successful discovery-to-cycle-decision path
2. weak opportunity rejected before production
3. Brain strategic contradiction and revision
4. missing Jax asset and resumed package
5. Sam rejection followed by targeted revision and new approval
6. approval/content or creative version mismatch blocked by Maro
7. partial multi-platform publication preserved through Lara analysis
8. immature measurement deferred
9. duplicate handoff/publication event remains idempotent
10. interrupted workflow resumes from persisted state without duplicate work

No Phase 3A integration eval suite or runtime was implemented.
