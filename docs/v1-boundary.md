# Minimum V1 and Decision Boundary

Status: Phase 3A prioritization only; no runtime, provider, database, or UI choice is made here.

## Minimum V1 verdict

The smallest useful runtime proves one opportunity through one complete, traceable cycle. It should support one content asset and at least one approved destination first, while preserving the contracts needed to add the remaining approved V1 destinations without changing authority boundaries.

## Must have before any real runtime activation

1. Artifact validation against the existing employee schemas.
2. Immutable artifact storage or equivalent persistence for source inputs, outputs, and versions.
3. A minimal handoff envelope with sender, artifact reference, receiver, status, acknowledgement, rejection/blocker, causation reference, and safe resume state.
4. The non-authoritative PUBLISH_PACKAGE assembler with exact Brain/Jax/Adam bindings.
5. The explicit APPROVAL record and a human approval mechanism that records Sam's decision, versions, scope, timestamp, and revocation/supersession state.
6. Approval and version validation before Maro can publish.
7. One least-privilege publication adapter, durable idempotency identity, receipt capture, ambiguous-outcome reconciliation, and restart-safe state.
8. One read-only analytics path that creates provenance-bearing PERFORMANCE_SNAPSHOT evidence for the same destination.
9. Lara-to-Travis PERFORMANCE_INSIGHT delivery and Travis CYCLE_DECISION.
10. End-to-end correlation from insight back to opportunity, plus basic failure visibility and cancellation.
11. Minimum brand, claims, asset-rights, privacy, secret-handling, and platform-policy rules needed for the selected real cycle.
12. The ten documented integration cases executed before activation.

“At least one destination first” is a rollout order, not a permanent change to approved V1 platform scope or content targets.

## Can wait until after the first V1 cycle

- additional platform adapters and broad account coverage
- automated optimized scheduling, blackout rules, and sophisticated capacity planning
- dashboards, rich Mission Control UI, and visual workflow builders
- a universal cross-platform metric model or normalization score
- advanced attribution, experiment statistics, anomaly thresholds, and causal inference
- semantic vector search, automatic deduplication infrastructure, and large source-monitoring coverage
- sophisticated model routing, fallbacks, and per-task optimization
- long-term semantic memory beyond minimal governed history
- multi-cycle concurrency optimization and advanced queue topology
- KPI formulas, weights, benchmarks, and performance compensation logic
- localization, additional formats, paid media, comments/DM workflows, account management, and public-content deletion/correction automation
- departments, additional managers, or an eighth employee

## Relevant unresolved-decision classification

This classification does not delete or resolve the canonical decision register.

### A. Must resolve before runtime design is considered safe

| Decision group | Existing IDs | Minimum decision needed |
|---|---|---|
| Approval authority and lifecycle | U-017, U-028 | Canonical APPROVAL identity, explicit Sam action, version/scope binding, rejection/change handling, revocation/supersession, and pause authority |
| Immutable revision identity | U-005, U-028 | New-ID versus version rules for CONTENT_BRIEF and CREATIVE_PACKAGE; dependency invalidation after material change |
| Package assembly and handoff semantics | U-006, U-007 | Mechanical assembler ownership, entry/exit criteria, acknowledgement/rejection, fan-out/fan-in, blocker ownership, and resume point |
| Minimum legal/brand/safety policy | U-010, U-012, U-031 | Claims, rights, privacy, likeness, disclosure, sensitive assets, and prohibited-content rules sufficient for the first real cycle |
| Activation boundary | U-018, U-028, U-029 | Who approves definitions/runtime activation and how pause/rollback overrides execution |

### B. Can be resolved during runtime build, but before activation

| Decision group | Existing IDs | Build-time decision |
|---|---|---|
| Execution and persistence | U-023, U-024, U-027, U-028 | State store/database, event/receipt persistence, concurrency, retention, restart recovery, correlation, and audit |
| Provider access and permissions | U-013, U-019, U-020 | First publisher/analytics providers, accounts, credentials, OAuth, least-privilege scopes, and secret lifecycle |
| Retry/idempotency mechanics | U-023, U-025 | Idempotency derivation/storage, bounded retry/backoff, timeout, reconciliation, and escalation limits |
| Scheduling | U-015 | First cycle's explicit timezone, publish intent, measurement checkpoint, and safe cancellation behavior |
| Analytics semantics | U-016 | Metrics available for the first provider, their definitions, data-quality mapping, and one useful measurement window |
| Models and costs | U-021, U-022 | Minimal model/tool selection, budget ceilings, and cost observability for each active step |
| Evaluation gate | U-026 | Pass criteria and human review for the ten integration cases |

### C. Defer until after V1

- U-001 departments and most of U-002 direct-collaborator formalization
- broad research allowlists/cadence and advanced score thresholds in U-008
- advanced planning, repurposing, and capacity rubrics in U-009 and U-014
- detailed multi-language/format rules in U-011 and U-032
- universal metric normalization, advanced attribution, full KPI formulas, and statistical experimentation in U-016 and U-026
- sophisticated autonomy promotion/demotion, long-term semantic memory, rich observability, and full lifecycle governance beyond safe activation
- missed/extra target accounting under U-030 beyond simple truthful reporting

### D. Obsolete or duplicate

No canonical register row is wholly obsolete. Several descriptions should eventually be consolidated:

- U-006, U-007, U-023, U-025, U-027, and U-028 repeat transport, state, recovery, lifecycle, and observability concerns.
- U-019 and U-020 overlap tool assignment with permission enforcement.
- U-015 is repeated in employee-specific scheduling backlogs.
- U-026 is repeated in every employee's remaining evaluation decisions.
- Employee-specific “runtime implementation” bullets duplicate U-023 and should later reference it instead of restating it.

Consolidation can wait; changing the register is unnecessary for Phase 3A.

## Travis bottleneck check

Verdict: MANAGEABLE BUT WATCH.

Travis correctly owns two high-leverage decisions: commitment before strategy and the next-cycle decision after learning. He becomes a bottleneck only if routine handoff acknowledgements, normal retries, asset collection, package assembly, or every employee blocker is routed through him.

Guardrail: specialists resolve local quality issues; the system handles mechanical state/recovery; Travis receives priority conflicts, cross-owner blockers, target risk, and decision-useful learning.

## Sam workload check

Verdict: HEALTHY IF APPROVAL IS CONSOLIDATED.

Sam should primarily handle important direction changes, one final version-bound public approval per release package, and exceptional brand/safety/authority escalations. Brain and Jax may receive targeted Sam feedback, but routine iteration must stay with Adam and the specialists.

The main workload risk is separate approval requests for copy, creative, metadata, and each platform. The canonical rule is one consolidated approval covering exact versions and scope, followed by new approval only after a material change.

## Eighth employee

No.

The missing responsibilities are mechanical coordination capabilities:

- package assembly
- approval recording
- artifact persistence/routing
- publication and analytics adapters
- recovery and trace correlation

These belong to the future system/runtime, not an independent employee with new judgment authority.

## Integration risks to carry forward

1. CONTENT_BRIEF and CREATIVE_PACKAGE lack intrinsic business-version fields; immutable revision identity must be settled before approval binding.
2. PUBLISH_PACKAGE assembly has no implemented owner; the lean conceptual owner is a non-authoritative system function.
3. APPROVAL is not a standalone schema; only Maro's APPROVE-shaped reference is currently machine-readable.
4. Brain/Jax fan-out/fan-in, acknowledgements, rejection, and resume semantics are not implemented.
5. PERFORMANCE_SNAPSHOT has a conceptual system/provider producer but no adapter or storage.
6. Lara's detailed reports/alerts are not direct Travis inputs and must travel through a referenced PERFORMANCE_INSIGHT; Maro outputs likewise require a truth-preserving PUBLICATION_RECORD projection.
7. ASSET_REQUEST has a destination but no defined acceptance/fulfillment receipt.
8. Mixed artifact envelope conventions (artifact_type versus input_type) require routing discipline, not employee redesign.
9. Travis can become a queue if local failures are escalated instead of routed to their owners.
10. Sam can become a workflow operator if approval is not consolidated.

## Recommended next phase

Phase 3B should resolve the five category-A decisions and write a runtime-ready integration contract: immutable revision rules, handoff envelope, approval lifecycle, package assembly contract, and safe activation boundary.

Do not select providers, build infrastructure, or activate employees until those decisions are approved.
