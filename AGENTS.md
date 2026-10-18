# AGENTS.md — Relune plugin template repository

You are a coding agent starting or maintaining a Relune plugin from one of the templates in
this repository. This file is the repo map; each template has its own `AGENTS.md` with the
project-level loop. Read both before making changes.

## Repository layout

| Path | Purpose |
| --- | --- |
| `templates/rust-processor/` | Processor plugin: UTF-8 text in → paragraph blocks out. |
| `templates/rust-connector/` | Connector plugin: paginated scan + ranged reads over a host-authorized directory. |
| `.github/workflows/ci.yml` | CI smoke: build → test → signed-package packaging for both templates. |
| `README.md` | Human entry point; quick start and layout. |

## Authoritative facts (do not restate or copy them)

- The Relune plugin contract (manifest schema, signature format, WIT worlds) is defined by the
  published [`relune-plugin-sdk`](https://crates.io/crates/relune-plugin-sdk) release pinned in
  each template's `Cargo.toml`. The SDK bundles the schema, signature contract, and WIT
  documents; the `relune-plugin` CLI validates against them.
- Templates pin: SDK `1.0.0`, manifest `specVersion` `"1.0"`, worlds
  `relune:client-plugin/processor@1.0.0` and `relune:client-plugin/connector@1.0.0`,
  Rust toolchain `1.98.1` with target `wasm32-unknown-unknown`, and (via the CLI) the
  `wasm-tools` component tool at its CLI-pinned version.
- API and behavior questions are answered by the docs for the pinned version:
  <https://docs.rs/relune-plugin-sdk>. If docs and code disagree, trust the compiled
  contract and report the drift.

## The loop a fresh plugin project must pass

Inside a template copy (each step fails loudly on its own; never skip or fake a step):

```bash
relune-plugin build          # Rust project → Wasm Component (wasm32-unknown-unknown)
relune-plugin test           # builds the component, then runs cargo test through the Mock Host Test API
relune-plugin key generate --out ../<project>-author.key   # once per author identity; keep outside the repo
relune-plugin package \
  --manifest manifest.json \
  --component target/relune-plugin/plugin.wasm \
  --key ../<project>-author.key \
  --locale en-US=locales/en-US.json \
  --out ../<project>.relune-plugin \
  --force
```

- `relune-plugin test` refuses to report success when no test actually ran. Do not weaken or
  bypass it; keep `tests/host_test.rs` real.
- Mock Host tests in the templates are contract-level acceptance for the template behavior.
  When changing plugin behavior, extend these tests in the same commit.
- `cargo test` (without the CLI) requires `RELUNE_PLUGIN_COMPONENT` pointing at a freshly
  built component; the CLI `test` subcommand orchestrates this for you.

## When turning a template into a real plugin

Minimum rename checklist (all four or the contract/graphs will disagree):

1. `Cargo.toml` → `[package] name` and, if you renamed the binary-facing crate, keep
   `publish = false`.
2. `src/lib.rs` → the `FINGERPRINT_LABEL` byte string (funnels into the implementation
   fingerprint; change it when implementation behavior changes in a user-visible way).
3. `manifest.json` → `id`, `version`, `publisher` (and `releasedAt` for new releases).
4. `locales/en-US.json` → user-visible `name`, `description`, and an honest `policy`.

## Rules for agents

- Never copy schema, WIT, signature fixtures, or canonicalization code into a plugin project.
  The pinned SDK is the only contract source; report missing APIs as SDK gaps instead.
- Never add upload, marketplace, telemetry, or credential handling to templates.
  Publishing happens through Relune's own services and is out of scope here.
- Keys (`*.key`, PEM files) and signed packages (`*.relune-plugin`) stay outside the project
  directory; `.gitignore` rejects them as a backstop.
- Keep the pinned toolchain and SDK versions unless the task explicitly asks for an upgrade,
  and upgrade both together in one change with `relune-plugin test` green.
- Keep every template buildable from a cold start with only public artifacts: crates.io,
  docs.rs, and this repository. Nothing may reach into the private Relune monorepo.
- Component outputs land under `target/relune-plugin/`; never commit build artifacts or
  `Cargo.lock` changes unrelated to dependency edits.

## Extending beyond the template baseline

The templates intentionally show only the baseline surface. Deeper host capabilities are all
lived in the SDK and documented per version at docs.rs — read them there instead of copying
signatures into docs here:

- **Remote requests** — HTTP fetch through the host with structured, retryable errors.
- **Media** — pixel-buffer inspection, rendering declared/remote/encoded media.
- **OCR** — whole-image text recognition through the host.
- **Embeddings** — contract-typed text and visual embeddings for RAG flows.

Connector-specific extension: multiple directory origins, child-entry listing, delta scans
(`ScanMode::Delta`), original-target deep links, and remote-backed connectors. All follow the
same Mock Host mock-and-observe patterns shown in `tests/host_test.rs`.
