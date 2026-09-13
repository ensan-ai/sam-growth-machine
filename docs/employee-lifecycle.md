# Employee Lifecycle

This document defines lifecycle states conceptually. It does not implement hiring, execution, scheduling, or state storage.

## Proposed lifecycle gates

1. Shell: the required repository package exists and known identity data is recorded.
2. Draft contract: Phase 2 defines mission, responsibilities, interfaces, capabilities, controls, handoffs, runtime policy, memory, operations, quality, and observability.
3. Review: the contract is checked for business accuracy, schema validity, permission boundaries, conflicts, and unresolved decisions.
4. Evaluation candidate: approved evaluation cases exist and can later be run in a controlled environment.
5. Approved for activation: Sam explicitly approves the employee definition and applicable autonomy boundary.
6. Active: a future runtime may execute the approved definition. This state is not available in Phase 1.
7. Change review: material static-definition changes are versioned and re-reviewed.
8. Paused or retired: future execution is disabled while historical definitions and receipts remain governed by a future retention policy.

## Current state

Travis, Saly, Adam, Brain, Jax, Maro, and Lara have progressed from Shell to complete Phase 2A, Phase 2B, Phase 2C, Phase 2D, Phase 2E, Phase 2F, and Phase 2G definitions with status reviewable. This corresponds to the draft-contract/review boundary; it is not activation approval.

No employee is active, runnable, hired into a runtime, or approved for autonomous execution.

## Unresolved lifecycle policy

Exact state names, transition authority, versioning rules, evaluation thresholds, rollback behavior, and retirement/retention policy require approval before implementation.
