# AGENTS.md — connector template project

Agent-facing loop for a connector plugin copied from this template. Repo-wide conventions
live in the root `AGENTS.md` of the template repository.

## Authoritative loop (all steps fail loudly on their own)

```bash
relune-plugin build                                # → target/relune-plugin/plugin.wasm
relune-plugin test                                 # builds, then runs tests/host_test.rs through the Mock Host
relune-plugin key generate --out ../my-author.key  # one key per author identity, outside the repository
relune-plugin package \
  --manifest manifest.json \
  --component target/relune-plugin/plugin.wasm \
  --key ../my-author.key \
  --locale en-US=locales/en-US.json \
  --out ../my-connector.relune-plugin \
  --force
```

Manual alternative: `relune-plugin build`, then
`RELUNE_PLUGIN_COMPONENT=target/relune-plugin/plugin.wasm cargo test`.

## Facts you must not restate or copy

- Contract (manifest schema, signature format, WIT) comes from the pinned
  [`relune-plugin-sdk`](https://crates.io/crates/relune-plugin-sdk) release, bundled inside
  the crate; the CLI validates packages against it. Never vendor schema/WIT files here.
- Connector world: `relune:client-plugin/connector@1.0.0`; manifests use
  `specVersion` `"1.0"`.
- API signatures and deeper capabilities (remote requests, media, OCR, embeddings) belong to
  the pinned SDK version — read them at <https://docs.rs/relune-plugin-sdk>, not from
  sources on disk.

## Definition of done for behavior changes

1. `src/lib.rs` implements the change; `tests/host_test.rs` gains/adjusts Mock Host
   assertions in the same commit.
2. `relune-plugin test` is green (the CLI refuses success when zero tests ran).
3. `relune-plugin package` with a fresh key produces a signed package; the CLI rejects
   contract violations (schema, worlds, signature payload) automatically.
4. Manifest/config/locale copy updated when the config schema, capability declarations, or
   user-visible naming changed.
5. `FINGERPRINT_LABEL` updated when the fingerprint should change (user-visible behavior
   change); stable otherwise — it feeds `implementation_fingerprint()`.

## Project-specific conventions

- Authorization is manifest-driven: `read`/`list_entries` may only target the origin id
  declared in `hostCapabilities.directoryReadAccess` (`source`, bound to the `folder`
  config field). Any other origin ids must be declared before use — undeclared access is a
  contract violation, not a defensive-coding bug.
- One host page per scan call: forward the host cursor via `ScanProgress::More` instead of
  looping inside a single call; the template keeps `SCAN_PAGE_SIZE` as its page bound.
- Entry fingerprints must stay content-independent and stable across rescans; keep the
  fingerprint recipe pure whenever you extend source metadata into scans.
- Host failures keep their stable codes (`ConnectorError::Host(HostError)`); plugin-domain
  failures use the dedicated `ConnectorError` variants (`InvalidConfig`, …). Never fold
  host errors into plugin enums.
- Carrier URIs are the plugin's contract with itself: same prefix constant in `scan`,
  `read_content`, and `resolve_original_target`; foreign URIs must not resolve to files.
