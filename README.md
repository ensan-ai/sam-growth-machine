# SAM Growth Machine

Local-first agentic OS for SAM SHERIF | PRACTICAL AI.

This repository defines the company, the SAM AI Employee Standard v1, and the
desktop Neural Core. Kernel-loop runtime work lives on `slice/kernel-loop`.

## V1 scope

V1 serves SAM PERSONAL BRAND only. Sam is the Founder, CEO, Human Authority, and final approval layer. Sam is not an AI employee.

The seven AI employees are Travis, Saly, Adam, Brain, Jax, Maro, and Lara. No other employee is defined in V1.

## Kernel slice

`docs/slice-v1-kernel-loop.md` is the frozen implementation spec for the first
closed loop:

`RESEARCH_SIGNAL → OPPORTUNITY → DECISION+ASSIGNMENT → BRIEF → DRAFT + NO_VISUAL_REQUIRED → PACKAGE → Sam APPROVAL → Execute (LOCAL_LEDGER) → Observe → Learn`

Intended public surface remains LinkedIn. The slice execution adapter is `LOCAL_LEDGER`. Only Brain may invoke a model. Maro is policy + adapter, never an LLM.

## Source of truth

- docs/product-vision.md records the approved product and business scope.
- docs/ai-employee-standard-v1.md defines the employee specification standard.
- docs/slice-v1-kernel-loop.md is the frozen kernel-loop implementation spec.
- agents/*/manifest.yaml is the primary static, machine-readable employee shell.
- agents/*/contract.md is its human-readable companion.
- workflows/personal-brand-growth-loop.yaml records only the approved high-level sequence.
- docs/unresolved-decisions.md is the decision backlog. An unresolved value must not be guessed or promoted into configuration.

## Current status

Employee definition packages are reviewable. The kernel loop is implemented in
`desktop/src-tauri` on branch `slice/kernel-loop`. Neural Core UI, OAuth, and
real social platform APIs are out of this slice.
