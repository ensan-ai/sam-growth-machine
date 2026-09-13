# SAM Neural Core Desktop

Phase 4A runnable macOS desktop application for the SAM GROWTH MACHINE.

## Development

Requirements:

- macOS Command Line Tools
- Rust stable
- Node.js 20+
- Ollama listening on `http://localhost:11434`
- local model `qwen3:14b`

From this directory:

```sh
pnpm install
pnpm tauri dev
```

The SQLite database is stored in the application data directory. Set
`SAM_GROWTH_MACHINE_ROOT` only when running the binary away from this repository.
OpenAI is optional and read only from `OPENAI_API_KEY`; the key is never stored or
shown. Paid escalation is disabled by default and must be enabled in Settings.

## Tests

```sh
pnpm test
pnpm test:rust
pnpm build
```

Rust integration tests use `MOCK_PROVIDER`. The normal desktop runtime does not
offer Mock as a selectable provider. Phase 4A publication and performance
collection remain explicit deterministic adapters until real platform APIs are
approved.

## Visual fallback

The Neural Core uses `react-force-graph-3d`/Three.js with fixed hierarchy
coordinates. Aurora and liquid-glass effects use lightweight CSS rather than
ShaderGradient or `liquid-glass-react`; this avoids turning decorative packages
into runtime blockers while preserving the locked visual direction.
