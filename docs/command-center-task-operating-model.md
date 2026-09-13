# SAM Command Center — Task Operating Model

Status: **approved target operating model for the Command Center task layer** on `feature/task-command-center-v1`.

This document captures the exact working method SAM Growth should follow for task execution. It defines the control-plane behavior around tasks; it does **not** replace the frozen company/content lifecycle in `docs/slice-v1-kernel-loop.md` or the seven approved V1 employees.

The seven V1 employees remain Travis, Saly, Adam, Brain, Jax, Maro, and Lara. Explorer, Researcher, Reviewer, Security Reviewer, Validator, and similar names below are **transient execution roles/sub-agents**, not new company employees.

---

## 1. Core principle

The operator must never manage serious AI work as disconnected chat/terminal sessions with no shared state.

Every task must live inside one Command Center that answers, at a glance:

- what is running now
- what is blocked and why
- what must happen first
- what can run in parallel
- who/what owns each task
- whether the task is Agent, Human, or Pair work
- what research was performed before execution
- what prompt was actually generated for the builder
- what reviewers inspected the result
- what changed, when, and by whom
- what task is unlocked next

The Command Center is the project map and execution ledger for humans and agents together.

---

## 2. Command Center domains

The Command Center has three mandatory operational views.

### 2.1 Swimlane

Purpose: project-level roadmap visibility.

The Swimlane is generated from project planning documents such as `MANIFESTO.md`, `ROADMAP.md`, approved milestones, and task definitions.

It shows:

- domains/major workstreams
- milestones
- planned start/end
- weeks or execution windows
- task counts per milestone
- completed/total task count
- percentage complete
- project status: ahead / on time / late
- project-specific compliance/checklists where needed

The Swimlane answers: **Where is the entire project going?**

### 2.2 Task Board

Purpose: execution order and dependency control.

Canonical lanes:

`BLOCKED → TODO → IN_PROGRESS → REVIEW → DONE`

Each task must display at minimum:

- task id
- title
- milestone/domain
- owner
- execution mode
- dependencies
- priority/order
- current status
- blocker reason when blocked

The Task Board is the operational translation of the Swimlane.

### 2.3 Agent Hub

Purpose: live execution visibility.

The Agent Hub shows transient execution roles and/or company employees currently participating in work, including:

- role/agent name
- provider/model when applicable
- task id
- current phase
- status
- start/finish timestamps
- latest action
- result/error
- handoff/reviewer relationship

Every meaningful execution action must be visible in the Agent Hub/event stream.

---

## 3. Task graph and dependency discipline

A task is never selected only because it exists on the board.

Before work starts, the system must determine:

1. which tasks are ready now
2. which tasks depend on unfinished tasks
3. which tasks can execute in parallel
4. which tasks require a strict sequence

If task `SG-014` depends on `SG-013`, `SG-014` must remain `BLOCKED` until `SG-013 = DONE`.

When the final dependency completes, the system performs **Auto-Unblock**:

`BLOCKED → TODO`

No human should have to move the dependent card manually.

Parallel work is permitted only when the task graph says the tasks are independent enough to execute safely.

---

## 4. Execution modes

Every task has exactly one execution mode:

- `AGENT` — AI/system can execute without routine human work.
- `HUMAN` — the operator/human must perform the task.
- `PAIR` — AI and human collaborate; operator input is part of execution.

The system may recommend the mode by inspecting the task and project, but the mode must be visible and auditable.

Examples:

- scrape/analyze public creator content → AGENT
- create an external account requiring Sam identity/action → HUMAN
- choose a strategically sensitive final content angle → PAIR

---

## 5. Operator command surface

The intended operator experience is deliberately small.

For normal tasks, Sam should primarily use:

`PREPARE TASK <id>`

`START TASK <id>`

`COMPLETE TASK <id>`

Review is a visible system phase between START and COMPLETE. The operator must not need to manually write the full builder prompt.

The system does the preparation, routing, logging, dependency checks, and review orchestration in the background.

---

## 6. Phase 1 — PREPARE TASK

`PREPARE TASK` does **not** execute the task.

Its job is to study the task and generate the strongest execution package possible before building begins.

### 6.1 Prepare sequence

~~~text
Operator
  ↓
PREPARE TASK SG-xxx
  ↓
Orchestrator
  ├─ spawn Explorer
  └─ spawn Researcher
        ↓
Explorer result + Researcher result
        ↓
Orchestrator synthesis
        ↓
Generated task prompt / execution package
        ↓
Persist prompt + update shared state + log events
        ↓
Task remains TODO/PREPARED
~~~

### 6.2 Explorer role

Explorer studies the implementation environment rather than inventing a solution immediately.

Depending on task type it may inspect:

- repository/codebase
- relevant files and previous implementations
- project architecture and conventions
- existing artifacts
- available tools and APIs
- current task/dependency state
- prior work on the same subject
- data/schema/source locations

For coding work, Explorer performs a codebase sweep focused on the task.

For growth/content work, Explorer inspects the relevant creator/source/content/data environment.

Explorer returns evidence and context to the Orchestrator.

### 6.3 Researcher role

Researcher studies the task requirements and the correct way to satisfy them.

It determines:

- what the task really requires
- success/acceptance criteria
- standards/policies that apply
- likely implementation approach
- edge cases and risks
- missing decisions
- facts requiring verification
- dependencies the original task description may have missed

Researcher returns evidence and recommendations to the Orchestrator.

### 6.4 Human input during PREPARE

Explorer/Researcher must not guess a material operator decision.

If a decision is needed, the system pauses preparation and returns:

- the question
- available options
- recommended option
- reason for recommendation
- consequence of each meaningful option when relevant

Example:

~~~text
OPERATOR INPUT REQUIRED

Option A: analyze last 30 posts
Option B: analyze top 20 outliers from six months

Recommendation: Option B
Reason: stronger pattern signal for this task
~~~

After Sam responds, PREPARE continues from the same task state.

Routine technical choices inside already-approved constraints should not interrupt Sam.

### 6.5 Generated task prompt

The Orchestrator combines:

- original task
- Explorer evidence
- Researcher evidence
- project rules/contracts
- relevant prior artifacts/state
- dependencies
- allowed tools
- constraints
- success criteria
- validation/review requirements

and writes a detailed prompt/execution package for the Builder.

The generated prompt may be long. That is intentional. Sam should not be expected to hand-write it.

Recommended logical artifact:

`tasks/prompts/<task-id>.md`

or the runtime equivalent stored in the task database/artifact store.

Minimum generated prompt sections:

1. Task identity
2. Objective
3. Why this task exists
4. Current project context
5. Relevant discovered files/data/sources
6. Dependencies and prerequisites
7. Required implementation/output
8. Constraints and forbidden assumptions
9. Acceptance criteria
10. Required tests/checks
11. Review chain
12. Operator decisions already made
13. Expected deliverables/artifacts

At the end of PREPARE:

- `prepared_at` is recorded
- the generated prompt is persisted
- preparation events are logged
- the task stays in `TODO` (prepared, not yet executing)

`PREPARED ≠ STARTED`.

---

## 7. Phase 2 — START TASK

`START TASK <id>` is allowed only when:

- all hard dependencies are DONE
- the task is not BLOCKED
- PREPARE has completed successfully
- required operator decisions are resolved
- an execution package exists

On START:

`TODO → IN_PROGRESS`

The Orchestrator loads the generated task prompt and deploys the required Builder/execution role.

The Builder executes the task from the prepared package rather than from a one-line card title.

---

## 8. Builder and specialist execution

“Builder” means the primary executor for the task, not necessarily a coding agent.

Possible builders include:

- code builder
- scraper/collector
- transcript processor
- creator analyst
- content analyst
- growth strategist
- script writer
- adapter/runtime worker

The Orchestrator selects the appropriate execution capability for the task.

The execution layer may use Claude, Codex, Gemini, local models, deterministic code, APIs, or other approved providers. The Command Center must be provider-neutral.

Provider/model choice is an implementation decision beneath the task contract and must be logged when a model is used.

---

## 9. Phase 3 — review chain

Builder success does **not** mean task completion.

After the Builder returns, the Orchestrator deploys the review roles required by that task type.

Reference coding chain:

~~~text
Builder
  ↓
Reviewer / Code Quality Reviewer
  ↓
Security Reviewer
  ↓
Validator
  ↓
Project-specific Compliance Reviewer (only when applicable)
  ↓
REVIEW
~~~

Rules:

- Reviewers inspect the Builder output independently enough to find errors the Builder may miss.
- Security review is required only where security concerns are relevant.
- Compliance is project/task-specific, not a universal employee.
- Validator judges the findings/results and prevents a reviewer warning from being silently ignored.
- Failed review sends work back for correction rather than marking it DONE.
- Review findings, corrections, and decisions are logged.

For SAM Growth tasks the specialist chain changes by task type. Examples:

Creator intelligence:

`Analyst → Source/Evidence Reviewer → Pattern Validator → Brand/Relevance Review`

Script/content:

`Writer → Fact/Source Review → Brand Review → Duplicate/Novelty Review`

The principle remains the same: **build, then independently review, then expose to Sam when human review is required.**

---

## 10. Phase 4 — COMPLETE TASK

A task may be completed only after required reviews have passed and any required operator review has been performed.

Operator command:

`COMPLETE TASK <id>`

Transition:

`REVIEW → DONE`

Completion must:

- persist final result/artifact references
- persist reviewer outcomes
- write final task events
- mark completion timestamp
- update project progress
- evaluate every task that depends on this task
- Auto-Unblock newly eligible tasks

A model saying “done” is not sufficient evidence of completion.

---

## 11. Shared state — the central coordination mechanism

The reference workflow succeeds because every execution role reads/writes one shared operational state instead of relying on isolated session memory.

SAM Growth must preserve that behavior.

### 11.1 Canonical runtime truth

The current implementation persists runtime truth in SQLite. This remains the canonical durable store unless explicitly changed by an approved architecture decision.

### 11.2 Shared-state projection

To preserve the same agent-to-agent operating pattern, the runtime should expose/generate one unified shared-state projection that all approved execution roles can read through the Command Center/MCP boundary.

Logical form:

`shared_state.json`

This is a **runtime projection**, not repository configuration and not a replacement for the database.

It should contain operational references/state such as:

- project status and progress
- milestones
- task states
- task dependencies
- current owners/execution roles
- active runs
- prepared prompt references
- recent events
- blockers
- review state
- latest completed actions
- timestamps

Large business datasets, transcripts, scraped content, analytics history, and immutable artifacts stay in their proper durable stores; shared state contains references and current coordination state.

Every execution role must be able to know what relevant roles did previously without trusting ephemeral chat/session memory.

---

## 12. MCP / command boundary

Terminals, CLIs, desktop UI, and agents should interact with the Command Center through explicit tools/commands, not by editing runtime state files/database rows directly.

Target MCP/tool surface includes:

- `get_task(task_id)`
- `list_ready_tasks()`
- `get_dependencies(task_id)`
- `prepare_task(task_id)`
- `request_operator_input(task_id, ...)`
- `record_operator_decision(task_id, ...)`
- `start_task(task_id)`
- `update_task_progress(task_id, ...)`
- `report_blocker(task_id, ...)`
- `submit_task_artifact(task_id, ...)`
- `request_review(task_id, ...)`
- `record_review(task_id, ...)`
- `complete_task(task_id)`
- `get_agent_activity()`
- `get_shared_state()`

The tool layer enforces validation, permissions, transitions, logging, and idempotency.

No agent receives authority merely because it can access the database or filesystem.

---

## 13. Event/audit rule

Every meaningful breath of the project must be reconstructable from the audit trail.

Required examples:

~~~text
task.created
task.blocked
task.auto_unblocked
task.prepare_started
explorer.deployed
explorer.completed
researcher.deployed
researcher.completed
operator_input.requested
operator_input.resolved
task.prompt_generated
task.prepared
task.started
builder.deployed
builder.completed
reviewer.deployed
reviewer.finding
validator.completed
task.review_requested
task.completed
~~~

Each event records at minimum:

- event id
- task id
- actor/role
- event type
- timestamp
- structured detail/reference payload

The history must support answering: **who did what, when, based on what, and what happened next?**

---

## 14. Parallel terminal / multi-model operation

The system should support many ready tasks executing in parallel when dependencies permit.

Parallelism must be controlled by:

- dependency graph
- file/resource conflicts where relevant
- provider rate/token limits
- cost/budget policies
- concurrency limits
- human review capacity

The intended architecture is not Claude-only, Codex-only, or tied to one IDE.

A prepared task can be routed to an approved executor while the Command Center remains the stable control plane.

Example:

~~~text
Command Center
  ├─ Task A → Claude/CLI
  ├─ Task B → Codex/CLI
  ├─ Task C → Gemini
  └─ Task D → deterministic/API worker
~~~

All four still update the same task state, event history, and shared-state projection.

---

## 15. Task record minimum schema

Every Command Center task needs at least:

~~~text
task_id
title
description
project/domain
milestone
priority
status
execution_mode
owner
dependency_ids
blocked_reason
prepared_at
prompt_artifact_ref
started_at
review_at
completed_at
created_at
updated_at
~~~

Future additions may include estimates, planned dates, actual duration, token/cost metrics, risk, retries, and acceptance-test references.

---

## 16. Relationship to SAM's company lifecycle

The Task Operating Model and the SAM company/content lifecycle are separate layers.

Example:

A company lifecycle stage such as `STRATEGIZED → IN_PRODUCTION` may require several Command Center tasks.

The company state answers:

**What business/content lifecycle state is this opportunity in?**

The task state answers:

**What concrete work is being prepared/executed/reviewed right now?**

Do not collapse those two state machines into one enum.

The frozen kernel loop remains authoritative for its V1 slice. This document defines how concrete tasks supporting that loop should be controlled and observed.

---

## 17. Definition of done for the Command Center task system

The implementation is not considered faithful to this operating model until a real task can demonstrate all of the following:

1. Task exists on the board with ID and execution mode.
2. Dependencies can block it automatically.
3. Completion of prerequisites Auto-Unblocks it.
4. PREPARE deploys Explorer + Researcher or equivalent task-specific preparation roles.
5. Material missing operator decisions are surfaced with a recommendation.
6. Orchestrator generates and persists the detailed execution prompt.
7. START uses the generated prompt, not merely the task title.
8. Builder executes and produces a reviewable result.
9. Required independent review roles run.
10. Review findings can force correction.
11. COMPLETE is impossible before review requirements pass.
12. Every phase is recorded in the event history.
13. Shared state allows another approved session/agent to understand the task without relying on the first session's chat memory.
14. Dependent work is automatically reconsidered after completion.
15. Sam can understand project/task status from the Command Center without opening every AI session.

That is the target workflow for SAM Growth Command Center.