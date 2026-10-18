# AGENTS.md — processor template project

Agent-facing loop for a processor plugin copied from this template. Repo-wide conventions
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
  --out ../my-processor.relune-plugin \
  --force
```

Manual alternative: `relune-plugin build`, then
`RELUNE_PLUGIN_COMPONENT=target/relune-plugin/plugin.wasm cargo test`.

Publishing is an author action outside this loop: `relune-plugin publish ../my-processor.relune-plugin`
signs the author in through the browser once, then uploads under their account (the first-publish
key locks plugin ownership). Neither the template nor its CI publishes anything.

## Facts you must not restate or copy

- Contract (manifest schema, signature format, WIT) comes from the pinned
  [`relune-plugin-sdk`](https://crates.io/crates/relune-plugin-sdk) release, bundled inside
  the crate; the CLI validates packages against it. Never vendor schema/WIT files here.
- Processor world: `relune:client-plugin/processor@1.0.0`; manifests use
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
4. Manifest/locale copy updated when user-visible naming or policy changed
   (`manifest.json`, `locales/en-US.json`).
5. `FINGERPRINT_LABEL` updated when the fingerprint should change (user-visible behavior
   change); stable otherwise — it feeds `implementation_fingerprint()`.

## Project-specific conventions

- Only the baseline surface is implemented here: input content read + logs + `blocks`
  output. Everything else is an SDK capability, not template code.
- `utf8_content_text(content, MAX_TEXT_BYTES)` enforces the read bound; keep a bound for
  any content-sized read you add.
- Errors cross the WIT boundary structurally: return `ProcessorError` variants (e.g.
  `InvalidInput`), never panic silently; host-capability failures arrive as
  `ProcessorError::Host(HostError)` with a stable code.
- Keep `rawContent.suffixes` in `manifest.json` aligned with the suffixes the plugin
  claims to handle.
