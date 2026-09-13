# SAM AI Employee Standard v1

## Purpose

An AI employee is a governed company role with formal static definitions, not merely a prompt. This standard defines the minimum specification surface every SAM Growth Machine AI employee must eventually satisfy.

Phase 1 creates conforming shells. Unknown business values remain visibly unconfigured until Phase 2.

## Required employee package

Every AI employee uses this directory structure:

    agents/<employee-slug>/
        manifest.yaml
        contract.md
        prompt.md
        schemas/
            input.schema.json
            output.schema.json
        evals/
            cases.yaml

The manifest is the primary machine-readable static definition. The contract is the human-readable agreement. The prompt is reserved for eventual runtime behavioral instructions. The JSON Schemas define strict interfaces. Evaluation cases are reserved for pre-employment and regression evaluation.

## Required specification domains

### Identity

- employee id
- name
- title
- department
- version

The employee id is a stable lowercase slug. A definition version changes when the static contract changes.

### Organization

- reports_to
- direct collaborators
- organizational position

Workflow adjacency must not be mistaken for a management relationship.

### Mission

- primary mission
- business reason for existence

### Responsibilities

- owns
- does_not_own

Explicit exclusions are part of the contract and help prevent silent role expansion.

### Contracts

- accepted input types
- required output types

Inputs and outputs must be backed by strict versioned schemas before an employee can be activated.

### Capabilities

- skills
- tools

A declared tool is not automatically permitted. Capability and permission are separate concerns.

### Control

- permissions
- approval requirements
- autonomy level

Sam remains the ultimate human authority. Autonomy is earned and may be revised or revoked.

### Handoffs

- receives_from
- sends_to

Handoffs must use formal contracts and must not depend on unstructured free-form chat.

### Runtime policy

- model policy
- cost/budget policy
- execution environment
- retry policy
- timeout policy

These are declared policies, not live consumption or status data.

### Memory

- working memory policy
- long-term memory policy
- information that must not be stored

Live memory infrastructure and retention details are not part of Phase 1.

### Operations

- triggers
- schedules
- failure and escalation behavior

### Quality

- guardrails
- KPIs
- evals

An employee must not be treated as ready solely because a prompt exists. Contract, schema, approval, and evaluation readiness are separate gates.

### Observability

- events
- receipts
- tracing requirements

Meaningful future actions must be attributable and traceable.

## Configuration rules

1. Static employee configuration belongs in the repository.
2. Live runtime state never belongs in employee manifests, contracts, prompts, or evaluation definitions.
3. Unapproved values use explicit unconfigured markers and are tracked in docs/unresolved-decisions.md.
4. The same structural shape applies to all seven V1 employee definitions.
5. A prompt cannot override the manifest, contract, approval policy, or schema.
6. Contract changes require versioning and review before activation.

## Definition and configuration status

- shell identifies a Phase 1 employee package that has not received its detailed contract.
- reviewable identifies a completed employee specification awaiting review; it does not authorize runtime execution.
- unconfigured_phase_2 marks a section whose business values remain unresolved.
- partially_configured_phase_2a marks a section with approved values and explicitly unresolved details.
- configured_phase_2a marks a section whose required Phase 2A decisions are represented.
- partially_configured_phase_2b marks a Saly section with approved values and explicitly unresolved details.
- configured_phase_2b marks a Saly section whose required Phase 2B decisions are represented.
- partially_configured_phase_2c marks an Adam section with approved values and explicitly unresolved details.
- configured_phase_2c marks an Adam section whose required Phase 2C decisions are represented.
- partially_configured_phase_2d marks a Brain section with approved values and explicitly unresolved details.
- configured_phase_2d marks a Brain section whose required Phase 2D decisions are represented.
- partially_configured_phase_2e marks a Jax section with approved values and explicitly unresolved details.
- configured_phase_2e marks a Jax section whose required Phase 2E decisions are represented.
- partially_configured_phase_2f marks a Maro section with approved values and explicitly unresolved details.
- configured_phase_2f marks a Maro section whose required Phase 2F decisions are represented.
- partially_configured_phase_2g marks a Lara section with approved values and explicitly unresolved details.
- configured_phase_2g marks a Lara section whose required Phase 2G decisions are represented.

Travis is version 1.0.0 with definition_status: reviewable. His detailed Phase 2A contract and interfaces are present, but he is not active or approved for runtime execution.

Saly is version 1.0.0 with definition_status: reviewable. Her detailed Phase 2B contract and interfaces are present, but she is not active or approved for runtime execution.

Adam is version 1.0.0 with definition_status: reviewable. His detailed Phase 2C contract and interfaces are present, but he is not active or approved for runtime execution.

Brain is version 1.0.0 with definition_status: reviewable. His detailed Phase 2D contract and interfaces are present, but he is not active or approved for runtime execution.

Jax is version 1.0.0 with definition_status: reviewable. His detailed Phase 2E contract and interfaces are present, but he is not active or approved for runtime execution.

Maro is version 1.0.0 with definition_status: reviewable. His detailed Phase 2F contract and interfaces are present, but he is not active or approved for runtime execution.

Lara is version 1.0.0 with definition_status: reviewable. Her detailed Phase 2G contract and interfaces are present, but she is not active or approved for runtime execution.
