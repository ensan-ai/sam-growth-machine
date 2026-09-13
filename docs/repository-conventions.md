# Repository Conventions

## Scope

This repository is specification-first. Phase 1 contains Markdown, YAML, and JSON Schema artifacts only. Reserved runtime and desktop directories contain scope notices, not application code.

## Naming

- Employee ids and directory names use lowercase slugs: travis, saly, adam, brain, jax, maro, lara.
- Human-readable names preserve approved spelling and capitalization.
- YAML uses snake_case keys.
- JSON Schema files use the .schema.json suffix.
- Definition versions use semantic version strings.
- Unknown values are represented as null, empty collections, and an accompanying configuration_status of unconfigured_phase_2; they are never guessed.

## Source hierarchy

1. Approved product and governance documents define business intent.
2. The SAM AI Employee Standard defines required structure.
3. Each manifest is the primary machine-readable employee definition.
4. Each contract explains its employee definition to humans.
5. Input and output schemas define interfaces.
6. A prompt may eventually express behavioral instructions but cannot override higher-level contracts.
7. Evaluation cases test the approved contract; they do not create policy.

## Change discipline

- A new employee requires an explicit approved V1 scope change.
- A business-specific value moves from unresolved to decided only after approval.
- Material employee contract changes require a version change.
- Static configuration must never absorb live runtime state.
- Workflow specifications remain non-executable until a future implementation phase is approved.

## Validation expectations

- All YAML must parse.
- All JSON must parse and all JSON Schema documents must pass schema checks.
- All seven agent packages must have identical required paths.
- Every manifest must conform to schemas/ai-employee-manifest.schema.json.
- The high-level workflow must conform to schemas/workflow-spec.schema.json.
- The repository must contain exactly the seven approved agent directories.
