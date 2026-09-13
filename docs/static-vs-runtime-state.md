# Static Configuration vs Live Runtime State

This boundary is mandatory.

## Static configuration: repository-owned

Static configuration describes the approved design of an employee or workflow. Examples include:

- identity, employee id, name, title, and definition version
- organizational definition and manager relationship
- mission and responsibility boundaries
- declared inputs, outputs, skills, and tools
- permission and approval policies
- model, budget, retry, timeout, and execution-environment policies
- memory policies
- triggers, schedules, guardrails, KPIs, eval definitions, and event requirements
- schemas and contracts

Static configuration is reviewed, versioned, and changed deliberately.

## Live runtime state: future database-owned

Live state describes what is happening or what happened during operation. It must not be written into employee configuration files. Examples include:

- current task and current status
- token use and current or accumulated cost
- tasks completed today
- performance and activity history
- traffic generated
- live errors and retry counts
- current approval queue
- execution traces and emitted receipts
- current schedules or leases as runtime instances
- working-memory contents

The future runtime database and retention design are unresolved and are not created in Phase 1.

## Enforcement principle

Repository configuration may define what must be measured or recorded, but it must not store the resulting live measurements or events. A future implementation should reject or isolate runtime fields if they appear in static definitions.
