# SAM Neural Core Desktop

Phase 4A runnable macOS desktop application for the SAM GROWTH MACHINE.

The kernel loop on `slice/kernel-loop` runs seven role-owned capabilities
inside this desktop runtime. Intended platform is LinkedIn; execution in this
slice is the `LOCAL_LEDGER` adapter. Only Brain may call a model. Observe
defaults to `UNAVAILABLE`.

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
# headless kernel tests (no GTK/WebKit):
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --lib

pnpm build
```

Rust kernel tests use a content-preserving Brain test double. `MOCK_PROVIDER`
and schema-fixture recovery are not a success path. Synthetic `COMPLETE`
observations are tests-only and must set `mocked: true`.

## Visual fallback

The Neural Core uses `react-force-graph-3d`/Three.js with fixed hierarchy
coordinates. Aurora and liquid-glass effects use lightweight CSS rather than
ShaderGradient or `liquid-glass-react`; this avoids turning decorative packages
into runtime blockers while preserving the locked visual direction.
