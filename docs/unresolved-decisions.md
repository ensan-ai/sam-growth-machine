# Decision Register

This register separates approved facts from decisions that must not be guessed. Phase 2 should resolve only the items necessary to create complete employee contracts; implementation decisions remain deferred to the appropriate later phase.

## Decided

- Product name: SAM Growth Machine.
- V1 business scope: SAM PERSONAL BRAND only.
- Sam is Founder, CEO, Human Authority, and Final Approval Layer; Sam is not an AI employee.
- V1 contains exactly seven AI employees: Travis, Saly, Adam, Brain, Jax, Maro, and Lara, with the approved titles recorded in the org chart.
- The formal reporting hierarchy is: Travis reports to Sam; Saly, Adam, Maro, and Lara report to Travis; Brain and Jax report to Adam.
- Reporting hierarchy and workflow handoffs are independent. A handoff does not imply a management relationship.
- Travis's Phase 2A mission, ownership boundaries, four input categories, four output artifacts, bounded internal decision rights, Sam approval boundary, operating rhythm, KPI signals, conceptual memory policy, and failure behavior are defined in his version 1.0.0 employee package.
- Saly's Phase 2B mission, ownership boundaries, research-source policy, discovery categories, brand relevance gate, semantic deduplication behavior, 1-to-5 score scale, OPPORTUNITY_CARD, curated OPPORTUNITY_BATCH, research authority, KPI signals, conceptual memory policy, and quality behavior are defined in her version 1.0.0 employee package.
- Adam's Phase 2C mission, ownership boundaries, REACH/AUTHORITY/BOTH roles, V1 format strategy, nine-point quality gate, CONTENT_BRIEF, REPURPOSING_MAP, STRATEGIC_CONCERN, Travis boundary, Brain/Jax strategy handoffs, KPI signals, conceptual memory policy, and failure behavior are defined in his version 1.0.0 employee package.
- Brain's Phase 2D mission, ownership boundaries, base voice profile, format-specific writing behavior, claim-safety rules, revision behavior, CONTENT_DRAFT and issue artifacts, Adam and Jax boundaries, twelve-point quality gate, KPI signals, conceptual memory policy, and failure behavior are defined in his version 1.0.0 employee package.
- Jax's Phase 2E mission, ownership boundaries, proof-first policy, EASY/MEDIUM/HIGH production philosophy, controlled visual taxonomy, CREATIVE_PACKAGE, ASSET_REQUEST, GENERATION_BRIEF, NO_VISUAL_REQUIRED, conflict behavior, Adam and Brain boundaries, fourteen-point quality gate, KPI signals, conceptual memory policy, and failure behavior are defined in his version 1.0.0 employee package.
- Maro's Phase 2F mission, ownership boundaries, version-bound approval gate, PUBLISH_PACKAGE, conceptual publication states, per-platform operations and receipts, multi-platform results, duplicate protection, failure classes, scheduling behavior, least-privilege boundaries, KPI signals, operational memory policy, and failure behavior are defined in his version 1.0.0 employee package.
- Lara's Phase 2G mission, ownership boundaries, objective-aware metric philosophy, qualified-audience principle, five evidence inputs, five analytical outputs, data-quality states, comparison and causality rules, Maro and Travis boundaries, KPI signals, conceptual memory policy, and read-oriented permission posture are defined in her version 1.0.0 employee package.
- The current high-level growth-loop sequence is the sequence in workflows/personal-brand-growth-loop.yaml.
- A Sam approval gate occurs after the Brain + Jax stage and before Maro in that sequence.
- Human approval outcomes must eventually include APPROVE, REQUEST CHANGES, REJECT, and PAUSE.
- Autonomy must be earned.
- Handoffs must eventually be formal machine-readable events with the minimum fields listed in docs/handoff-philosophy.md.
- Meaningful actions must eventually leave operational events or receipts.
- Static configuration belongs in the repository; live runtime state does not.
- The current weekly content target is recorded in docs/product-vision.md.
- One approved master Short distributed to four short-video platforms remains one original content asset and creates four platform publication records.
- Phase 1 is specifications only and excludes application and infrastructure implementation.

## Unresolved

| ID | Area | Decision required |
|---|---|---|
| U-001 | Organization | Exact departments for all seven employees. |
| U-002 | Organization | Exact direct collaborators and manager operating responsibilities beyond the approved formal reporting hierarchy. |
| U-005 | Interfaces | Broader cross-version compatibility and any completed-employee additions beyond approved minimums. Maro's Publish Package conceptually binds Adam, Brain, and Jax artifacts; Lara's analytical inputs bind those artifacts and Maro publication artifacts without redesigning completed employees. |
| U-006 | Handoffs | Detailed handoff infrastructure and payload behavior beyond declared static interfaces; Brain + Jax fan-out/fan-in and assembly behavior; validation, rejection, rework, and escalation mechanics. |
| U-007 | Workflow | Exact meaning of each high-level stage, entry/exit criteria, cycle boundaries, and how NEXT DECISION / NEXT CYCLE is chosen. |
| U-008 | Research | Exact configurable source allowlist, selected newsletters/creators/accounts/competitors, research cadence, geographic/language scope, access methods, and source-specific operating rules. Saly's source classes and quality policy are approved. |
| U-009 | Strategy | Exact planning horizon, capacity model, format-selection rubric, content-role measurement, and repurposing thresholds. Adam's current decision rights and quality gate are approved. |
| U-010 | Brand | Audience definition, prohibited topics, and any voice or claims rules beyond Brain's approved high-level voice and evidence-preservation behavior. |
| U-011 | Writing | Exact length constraints and any writing rules beyond Brain's approved Short, LinkedIn, X, Facebook, hook, CTA, quality-gate, and platform-adaptation behavior. |
| U-012 | Creative | Exact visual identity, production specifications, asset-rights/licensing policy, generation/editing tools, and quality thresholds beyond Jax's approved proof-first planning rules. |
| U-013 | Distribution | Publishing providers/APIs, account mapping, provider-specific metadata, credential handling, exact permissions, and runtime enforcement beyond Maro's approved Phase 2F behavior. |
| U-014 | Platform adaptation | Copy/creative adaptation rules beyond Brain's Phase 2D and Jax's Phase 2E behavior and provider-specific operational transformations. One master Short may create four platform records without becoming four originals. |
| U-015 | Scheduling | Exact days, times, time zone, blackout rules, and scheduling details; Travis's weekly, daily, and event-driven rhythm is approved without clock times. |
| U-016 | Analytics | Analytics providers, exact measurement and attribution windows, metric definitions, KPI formulas, weights, benchmarks, targets, and feedback implementation; Lara's objective-aware measurement behavior and Travis's qualitative North Star are approved. |
| U-017 | Approval | Approval production/validation infrastructure, detailed acceptance criteria, delegation if any, rework routing, response times, revocation, and emergency pause behavior. Maro requires explicit version- and scope-bound Sam approval before publication. |
| U-018 | Autonomy | System-wide promotion, demotion, and revocation rules; all seven reviewable employees have bounded internal authority only. |
| U-019 | Capabilities | Exact tool assignments for all seven employees, plus the difference between declared, permitted, and required tools. |
| U-020 | Permissions | Future enforcement mechanics for all employees; the seven reviewable employees' current allow/deny boundaries are approved. |
| U-021 | Models | Model routing, provider policy, fallback policy, and use-case constraints. |
| U-022 | Cost | Per-task and periodic budgets, spend approval thresholds, and cost allocation rules. Travis, Maro, and Lara have no independent spending authority. |
| U-023 | Execution | Future execution environment, isolation, concurrency, exact retry/timeout values, and idempotency storage/algorithm. Maro's stable publication identity and reconcile-before-retry behavior are conceptually defined. |
| U-024 | Memory | Storage implementation, retention, deletion, correction, privacy, and access policy; the seven reviewable employees' conceptual memory categories and raw-data prohibitions are approved. |
| U-025 | Failure handling | Exact retry limits, reassignment rules, escalation service levels, safe fallback mechanics, and incident severity; Travis's, Jax's, Maro's, and Lara's required failure postures are approved for their current phases. |
| U-026 | Quality | KPI formulas, evaluation scoring, pass thresholds, regression policy, and human review sampling; the seven reviewable employees' guardrails and semantic cases are defined. |
| U-027 | Observability | System-wide event/receipt schema, correlation/causation rules, storage, ordering, retention, redaction, and tracing depth. Maro's publication receipt and Lara's analytical artifact trace requirements are defined. |
| U-028 | Lifecycle | Final lifecycle states, transition authority, contract approval process, version compatibility, rollback, pause, retirement, and reactivation rules. |
| U-029 | Definition governance | Who may propose/edit employee definitions and how conflicts among vision, manifest, contract, prompt, schemas, and evals are resolved operationally. |
| U-030 | Content target | How missed or extra original assets and platform publications are treated against weekly targets. Cross-posting one approved Short remains one original plus four publication records. |
| U-031 | Legal and safety | Claims, disclosure, copyright, likeness, privacy, moderation, regulated-topic, and platform-policy guardrails. |
| U-032 | Localization | Supported languages, locale conventions, and whether content variants are in V1. |

No unresolved item has been encoded as active employee behavior in Phase 1.

## Remaining Travis decisions after Phase 2A

- Department and exact direct-collaborator designation.
- Formal runtime delivery of Saly OpportunityCard and Lara PERFORMANCE_INSIGHT artifacts beyond the compatible static interfaces.
- Detailed handoff routing, rejection, rework, and escalation payloads beyond the declared source/destination boundary.
- Exact priority-label and qualitative-confidence taxonomies.
- Exact model providers, models, routing, and fallbacks.
- Exact direct tool assignments and access-enforcement mechanics.
- Model/task budgets and cost-allocation rules; Travis cannot spend independently.
- Execution environment, isolation, idempotency, concurrency, and runtime implementation.
- Exact retry counts, timeout values, reassignment rules, and escalation service levels.
- Exact days, clock times, time zone, and blackout rules for weekly and daily activity.
- KPI formulas, weights, attribution rules, benchmarks, targets, and numerical thresholds.
- Memory storage, retention, access, privacy, correction, and deletion rules.
- Exact evaluation rubric, weighting, pass threshold, and human review sampling.
- Event/receipt schemas, storage, ordering, retention, and redaction.
- Detailed approval-interface mechanics, response expectations, delegation rules if any, and rework routing.

These remain unresolved. No choice above is implied by Travis's reviewable specification.

## Remaining Saly decisions after Phase 2B

- Department and exact direct-collaborator designation.
- Exact source allowlist, selected newsletters, creators/accounts, competitors, geographic scope, and language scope.
- Exact research tools, APIs, access methods, provider integrations, and permissions enforcement.
- Exact discovery cadence, research windows, monitoring intervals, days, clock times, and time zone.
- Exact per-score rubrics, confidence criteria, recommendation thresholds, and any future weighting; no weighted formula is approved.
- Exact curated batch-size limits and rules for when a WATCH item belongs in a batch.
- Exact semantic-similarity thresholds, comparison windows, implementation method, and duplicate-review escalation.
- Exact promotion rules from WATCH to PURSUE_CANDIDATE or REJECT.
- Model providers, models, routing, fallbacks, and context limits.
- Research/model budgets, rate limits, and cost-allocation rules; Saly cannot spend independently.
- Execution environment, isolation, retry counts, timeout values, idempotency, and runtime implementation.
- Raw-source capture limits, copyright/licensing handling, memory storage, retention, privacy, access, correction, and deletion.
- KPI formulas, attribution, benchmarks, targets, and numerical thresholds for QUALIFIED OPPORTUNITY YIELD.
- Evaluation scoring rubric, weighting, pass threshold, regression policy, and human review sampling.
- Event and receipt schemas, rejected-card storage, ordering, retention, redaction, and trace presentation.
- Version-negotiation policy for Saly's namespaced batch extension and any future Travis interface revision.

These remain unresolved. No choice above is implied by Saly's reviewable specification.

## Remaining Adam decisions after Phase 2C

- Department and exact direct-collaborator designation.
- Exact priority labels and qualitative-confidence taxonomy.
- Whether a future Travis ASSIGNMENT version adds a separate expected_outcome field; Phase 2C uses objective plus success_condition without changing Travis.
- Exact content-role and format-selection rubrics beyond the approved qualitative definitions.
- Exact repurposing thresholds, capacity limits, adaptation-count limits, and when a separate asset is strategically justified.
- Final creative adaptation rules and any copy rules beyond Brain's approved Phase 2D behavior; Adam defines strategy only.
- Exact content-catalog comparison method, semantic-duplicate threshold, recent-work window, and implementation.
- Weekly capacity model, allocation mechanics, and conflict-resolution procedure when targets exceed feasible quality work.
- Exact evidence-sufficiency criteria and operational handling for each quality-gate failure.
- Adam CONTENT_BRIEF compatibility with Brain 1.0.0 and Jax 1.0.0 is resolved. Formal runtime delivery, acceptance orchestration, and Adam-originated revision transport remain unresolved.
- Brand voice, editorial style, claims policy, and format-specific constraints beyond Brain's approved high-level Phase 2D rules.
- Model providers, models, routing, fallbacks, and context limits.
- Exact read tools, artifact/catalog access, permissions enforcement, budgets, and cost-allocation rules.
- Execution environment, isolation, retry counts, timeout values, idempotency, and runtime implementation.
- Work scheduling, service levels, revision timing, and escalation response expectations.
- Memory storage, retention, access, privacy, correction, and deletion.
- KPI formulas, attribution, benchmarks, targets, and numerical thresholds for CONTENT STRATEGY YIELD.
- Evaluation scoring rubric, weighting, pass threshold, regression policy, and human review sampling.
- Event and receipt schemas, storage, ordering, retention, redaction, and trace presentation.
- Formal runtime handoff mechanics for CONTENT_BRIEF, REPURPOSING_MAP, and STRATEGIC_CONCERN.

These remain unresolved. No choice above is implied by Adam's reviewable specification.

## Remaining Brain decisions after Phase 2D

- Department and exact direct-collaborator designation.
- Exact language-specific, platform-specific, and format-specific voice profiles beyond the approved base voice and current writing behavior.
- Supported languages, locale conventions, translation ownership, and localization review.
- Exact length, duration, character-count, and structure constraints for each format and platform.
- Exact evidence-sufficiency thresholds, claim-verification ownership, controlled verification access, and legal/editorial claims policy.
- Exact handling when Sam's writing direction conflicts with an approved Content Brief, including the formal Adam review and resolution path.
- Formal Adam-originated revision-request transport. Phase 2D supports a namespaced `CONTENT_BRIEF.extensions.brain_revision` object without changing Adam's schema.
- Brain CONTENT_DRAFT compatibility with Jax 1.0.0 is resolved. Combined-artifact assembly, fan-out/fan-in mechanics, and runtime acceptance orchestration remain unresolved.
- Exact model providers, models, routing, fallbacks, and context limits.
- Exact tools, source-access controls, permissions enforcement, budgets, and cost-allocation rules; Brain has no independent spending authority or broad web-search entitlement.
- Execution environment, isolation, retry counts, timeout values, idempotency, concurrency, and runtime implementation.
- Work scheduling, service levels, revision timing, and escalation response expectations.
- Memory storage, retention, access, privacy, correction, deletion, and the evidence threshold for generalizing a style preference.
- KPI formulas, attribution, benchmarks, targets, and numerical thresholds for WRITING EXECUTION QUALITY.
- Evaluation scoring rubric, weighting, pass threshold, regression policy, and human review sampling.
- Event and receipt schemas, version-history storage, ordering, retention, redaction, and trace presentation.
- Detailed approval-interface mechanics, combined Brain/Jax review, response expectations, and rework routing; Sam remains final approval authority.

These remain unresolved. No choice above is implied by Brain's reviewable specification.

## Remaining Jax decisions after Phase 2E

- Department and exact direct-collaborator designation.
- Exact visual identity, brand system, thumbnail system, reusable component library, and rules for maintaining them.
- Exact technical production specifications, including dimensions, safe areas, duration, frame rate, codecs, export profiles, accessibility, and platform-specific variants.
- Exact criteria and rolling measurement window for the approximate EASY/MEDIUM production mix; HIGH remains rare and justified without a fixed quota.
- Exact platform conditions that justify separate Short edits instead of one reusable master.
- Exact asset-rights, licensing, logo, screenshot, likeness, privacy, disclosure, and sensitive-information policies.
- Exact meaning and enforcement of asset approval, provenance verification, and rights-or-usage status.
- Exact thresholds for when generated content could be mistaken for real evidence and the required labeling or disclosure treatment.
- Exact generation, capture, design, editing, metadata-inspection, export, and asset-library tools; providers and models remain unresolved.
- Formal asset storage, naming, versioning, deduplication, reuse, retention, deletion, and access policy.
- Formal Brain/Jax parallel-work, assembly, conflict-resolution, and combined approval mechanics.
- Formal ASSET_REQUEST ownership, response routing, service levels, fulfillment confirmation, and rework behavior.
- Exact approval-sensitive review procedure, including Sam response expectations and treatment of unresolved items.
- Exact model routing, context limits, fallbacks, permissions enforcement, budgets, and cost allocation; Jax has no independent spending authority.
- Execution environment, isolation, retry counts, timeout values, idempotency, concurrency, and runtime implementation.
- Memory storage, retention, access, privacy, correction, deletion, and the evidence threshold for generalizing a visual preference.
- KPI formulas, attribution, benchmarks, targets, and numerical thresholds for CREATIVE EXECUTION VALUE.
- Evaluation scoring rubric, weighting, pass threshold, regression policy, and human review sampling.
- Event and receipt schemas, production-history storage, ordering, retention, redaction, and trace presentation.
- Downstream Publish Package assembly, immutable creative snapshot versioning, and actual handoff mechanics after Sam approval. Maro's required input is now defined.

These remain unresolved. No choice above is implied by Jax's reviewable specification.

## Remaining Maro decisions after Phase 2F

- Department and exact direct-collaborator designation.
- Which component assembles and signs the PUBLISH_PACKAGE and how it obtains immutable upstream artifact snapshots.
- Formal approval creation, cryptographic or authoritative validation, revocation, expiry, delegation, and scope-amendment mechanics.
- Canonical creative snapshot versioning because Jax's current CREATIVE_PACKAGE has an identity but no intrinsic version field; Phase 2F carries a package-level creative version without changing Jax.
- Exact publishing providers, APIs, account identifiers, account-to-platform mapping, credentials, OAuth, and token lifecycle.
- Exact least-privilege scopes and enforcement for create, upload, schedule, and read-status actions.
- Provider-specific required metadata, upload settings, validation rules, file specifications, and safe mechanical transformations.
- Exact rules distinguishing mechanical formatting from material copy or creative change.
- Exact days, times, approved windows, blackout rules, time-zone policy, and any future optimized scheduling policy.
- Exact publication-state persistence, transition authority, concurrency control, and recovery behavior.
- Exact idempotency-key derivation, storage, uniqueness scope, retention, and reconciliation algorithm.
- Exact retry counts, backoff, retry windows, rate-limit handling, and provider-specific retry safety.
- Exact methods for reconciling ambiguous outcomes and already-existing platform content.
- Exact cancellation authority, race handling, and behavior when cancellation overlaps an in-flight or completed operation.
- Exact treatment and authorization for public-content deletion or correction after publication; deletion is not currently permitted.
- Exact handling for comments, direct messages, profile settings, account creation, ads, boosts, and paid campaigns; all remain outside current scope.
- Publication receipt/result storage, event correlation, ordering, retention, privacy, redaction, and audit presentation.
- Runtime delivery, storage, and reconciliation of Maro publication artifacts consumed by Lara; static compatibility is defined in Phase 2G.
- KPI formulas, attribution, benchmarks, targets, and numerical thresholds for DISTRIBUTION RELIABILITY beyond zero conceptual tolerance for unauthorized publication.
- Evaluation scoring rubric, weighting, pass threshold, regression policy, and human review sampling.
- Runtime environment, isolation, timeouts, queues, schedulers, browser automation, and implementation; none exists in Phase 2F.

These remain unresolved. No choice above is implied by Maro's reviewable specification.

## Remaining Lara decisions after Phase 2G

- Department and exact direct-collaborator designation.
- Exact analytics providers, APIs, accounts, authentication, retrieval methods, rate limits, and provider contracts.
- Exact provider-specific metric definitions, availability, naming, transformations, and controlled extension governance.
- Exact EARLY, INTERMEDIATE, and MATURE measurement durations, refresh cadence, cycle timing, attribution windows, and cutoff rules.
- Exact rules for qualified audience, qualified profile activity, meaningful conversations, professional interest, and website attribution.
- Exact cross-platform normalization policy; Phase 2G prohibits naive equivalence but defines no universal normalization.
- Exact comparable-cohort selection, historical baseline construction, minimum sample sizes, pattern-promotion thresholds, and pattern expiry.
- Experiment design standards, setup-quality rubric, sample requirements, statistical methods if any, and approval process.
- Exact anomaly thresholds, severity, deduplication, notification, resolution, and escalation service levels.
- Runtime publication-record assembly and validation from Maro receipts/results, including ambiguous-state reconciliation.
- Formal access to Adam, Brain, and Jax artifacts and handling of inconsistent or superseded source context.
- Exact KPI formulas, weights, benchmarks, targets, confidence calibration, and usefulness measurement for DECISION-USEFUL MEASUREMENT.
- Memory storage, retention, privacy, access, correction, deletion, baseline governance, and raw-payload treatment.
- Exact read permissions and enforcement for social analytics, website analytics, content catalogs, historical reports, and any sensitive audience data.
- Event and analytical-artifact storage, correlation, ordering, lineage, retention, redaction, and audit presentation.
- Evaluation scoring rubric, weighting, pass threshold, regression policy, and human review sampling.
- Model providers, models, routing, fallbacks, context limits, budgets, and cost allocation; Lara cannot spend independently.
- Runtime environment, isolation, timeouts, retries, scheduling, orchestration, dashboards, browser automation, and implementation; none exists in Phase 2G.

These remain unresolved. No choice above is implied by Lara's reviewable specification.
