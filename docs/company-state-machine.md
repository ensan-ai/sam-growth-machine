# Lean Company State Machine

Status: conceptual Phase 3A integration state only. It coordinates existing artifacts; it does not replace employee-specific states or implement storage.

Runtime kernel slice states are defined in `docs/slice-v1-kernel-loop.md`:
`NEW → RESEARCHED → SELECTED → STRATEGIZED → IN_PRODUCTION → READY_FOR_APPROVAL → APPROVED → READY_TO_PUBLISH → PUBLISHED → MEASURING → MEASURED`,
plus `REVISION_REQUIRED`, `REJECTED`, `BLOCKED`, `CANCELLED`.
`BLOCKED` always stores `resume_state` and never semantically resumes to `NEW`.

This document remains the conceptual company lifecycle. Do not treat its names
(`CANDIDATE`, `AWAITING_APPROVAL`, `PUBLISHING`) as the SQLite runtime enum.

## States

| State | Meaning |
|---|---|
| CANDIDATE | Saly has a traceable opportunity that may enter Travis review. |
| SELECTED | Travis issued a pursue decision and active Assignment. |
| IN_PRODUCTION | Adam, Brain, and Jax are producing the strategy, words, and creative package. |
| AWAITING_APPROVAL | The system assembled exact review-package versions for Sam. |
| REVISION_REQUIRED | A named owner must revise strategy, words, creative, metadata, or assets. |
| REJECTED | Sam rejected the item or Travis rejected the opportunity; no publication is allowed. |
| APPROVED | Sam explicitly approved exact versions and platform scope. |
| SCHEDULED | Maro accepted the package and recorded an approved future execution time/window. |
| PUBLISHING | One or more approved platform operations are unresolved in flight. |
| PUBLISHED | Every required target for this lifecycle is confirmed published. |
| PARTIALLY_PUBLISHED | At least one target published and at least one failed or blocked. |
| MEASUREMENT_WAITING | Publication truth exists, but the required measurement window is not ready or data is unavailable/immature. |
| MEASURED | Lara produced a decision-useful insight or explicitly bounded result for Travis. |
| BLOCKED | Work cannot advance because a dependency, permission, evidence, validation, or recoverable operational condition is unresolved. |
| CANCELLED | Authorized cancellation stops every not-yet-completed operation. |

PUBLISHED and PARTIALLY_PUBLISHED are company-level projections of Maro's platform-specific truth. MEASURED may contain NEED_MORE_DATA; it means the available evidence was interpreted truthfully, not that every metric exists.

## Normal and exceptional transitions

~~~mermaid
stateDiagram-v2
    [*] --> CANDIDATE
    CANDIDATE --> SELECTED: Travis pursues + assigns
    CANDIDATE --> REJECTED: Travis rejects
    SELECTED --> IN_PRODUCTION: Adam accepts
    IN_PRODUCTION --> AWAITING_APPROVAL: exact draft + creative/no-visual ready
    AWAITING_APPROVAL --> APPROVED: Sam approves exact versions/scope
    AWAITING_APPROVAL --> REVISION_REQUIRED: Sam requests changes
    AWAITING_APPROVAL --> REJECTED: Sam rejects
    REVISION_REQUIRED --> IN_PRODUCTION: owner accepts revision
    IN_PRODUCTION --> BLOCKED: dependency/evidence/asset conflict
    BLOCKED --> IN_PRODUCTION: dependency resolved
    APPROVED --> SCHEDULED: Maro accepts future schedule
    APPROVED --> PUBLISHING: publish-now operation starts
    SCHEDULED --> PUBLISHING: schedule becomes eligible
    PUBLISHING --> PUBLISHED: all targets confirmed
    PUBLISHING --> PARTIALLY_PUBLISHED: mixed confirmed outcomes
    PUBLISHING --> BLOCKED: recoverable or ambiguous failure
    BLOCKED --> PUBLISHING: safe retry/reconciliation resolves
    PUBLISHED --> MEASUREMENT_WAITING: measurement checkpoint pending
    PARTIALLY_PUBLISHED --> MEASUREMENT_WAITING: published targets eligible
    MEASUREMENT_WAITING --> MEASURED: Lara emits bounded insight
    MEASUREMENT_WAITING --> BLOCKED: analytics unavailable/conflicting
    BLOCKED --> MEASUREMENT_WAITING: evidence recovery succeeds
    SELECTED --> CANCELLED: authorized cancellation
    IN_PRODUCTION --> CANCELLED: authorized cancellation
    AWAITING_APPROVAL --> CANCELLED: authorized cancellation
    APPROVED --> CANCELLED: cancellation before execution
    SCHEDULED --> CANCELLED: cancellation before execution
    MEASURED --> [*]
    REJECTED --> [*]
    CANCELLED --> [*]
~~~

## Transition ownership

| From → trigger → next | Owner |
|---|---|
| CANDIDATE → pursue Assignment → SELECTED | Travis |
| CANDIDATE → reject/defer → REJECTED or remains CANDIDATE/WATCH | Travis, informed by Saly |
| SELECTED → accepted strategy work → IN_PRODUCTION | Adam |
| IN_PRODUCTION → ready immutable bundle assembled → AWAITING_APPROVAL | System assembly after Brain/Jax readiness |
| AWAITING_APPROVAL → explicit APPROVE → APPROVED | Sam |
| AWAITING_APPROVAL → REQUEST_CHANGES → REVISION_REQUIRED | Sam; revision routed to Adam, Brain, or Jax |
| AWAITING_APPROVAL → REJECT → REJECTED | Sam |
| REVISION_REQUIRED → new applicable version accepted → IN_PRODUCTION | Owning specialist |
| APPROVED → valid future schedule → SCHEDULED | Maro |
| APPROVED/SCHEDULED → operation starts → PUBLISHING | Maro |
| PUBLISHING → all targets confirmed → PUBLISHED | Maro |
| PUBLISHING → mixed outcomes → PARTIALLY_PUBLISHED | Maro |
| PUBLISHED/PARTIALLY_PUBLISHED → window not ready → MEASUREMENT_WAITING | System evidence capture / Lara |
| MEASUREMENT_WAITING → insight emitted → MEASURED | Lara |
| MEASURED → cycle decision/new Assignment | Travis; starts a new lifecycle rather than mutating the old one |
| Any eligible active state → authorized cancel → CANCELLED | Future control policy; Sam remains final pause authority |
| Active state → recoverable failure → BLOCKED | Current owner or system |
| BLOCKED → cause resolved/reconciled → last safe prior state | Current owner or system; never infer completion |

## Blocking discipline

- BLOCKED always records the blocked state, owner, reason, required resolution, and safe resume state.
- A recoverable failure does not create a new content version unless content, strategy, creative, or approval-bound metadata changes.
- An ambiguous publication never transitions directly to FAILED or retries blindly; Maro reconciliation determines the safe next state.
- A material revision after APPROVED invalidates approval and returns the lifecycle to IN_PRODUCTION, then AWAITING_APPROVAL.
- Cancellation cannot reverse already confirmed publication. It only stops unpublished work; correction/deletion remains a separate unresolved authority decision.
