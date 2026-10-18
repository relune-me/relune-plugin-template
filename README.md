# Relune Plugin Templates

Official starting points for building [Relune](https://github.com/relune-me/relune) plugins in
Rust. The templates are written for coding agents first: each project carries an `AGENTS.md`
with the authoritative build → test → package loop, and a Mock Host test suite that runs
against the real compiled WebAssembly component.

Two templates are provided:

| Template | Kind | What it demonstrates |
| --- | --- | --- |
| [`templates/rust-processor/`](templates/rust-processor/) | processor | Reads UTF-8 plain text supplied by the host and emits typed paragraph blocks |
| [`templates/rust-connector/`](templates/rust-connector/) | connector | Enumerates and reads a host-authorized directory with paginated scan and ranged reads |

Both templates:

- Depend on [`relune-plugin-sdk`](https://crates.io/crates/relune-plugin-sdk) **1.0.1** from
  crates.io — no path dependencies, no monorepo access needed.
- Compile to Wasm Components for the contract worlds
  `relune:client-plugin/processor@1.0.0` and `relune:client-plugin/connector@1.0.0`.
- Never copy contract sources (schema, fixtures, WIT): the pinned SDK release bundles them,
  and the developer CLI validates against the bundled contract automatically.
- Pin the Rust toolchain via `rust-toolchain.toml` (1.98.1 + `wasm32-unknown-unknown`), so any
  machine or agent sandbox gets the same build.

## Quick start (agents: follow the AGENTS.md files)

```bash
# One-time setup: pinned toolchain + developer CLI + component tool from crates.io
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --target wasm32-unknown-unknown
cargo install relune-plugin-sdk --locked --features tools
cargo install wasm-tools --version 1.254.0 --locked

# Start a project from a template, then follow its AGENTS.md
cp -R templates/rust-processor my-plugin && cd my-plugin
relune-plugin test
relune-plugin key generate --out ../my-author.key
relune-plugin package \
  --manifest manifest.json \
  --component target/relune-plugin/plugin.wasm \
  --key ../my-author.key \
  --locale en-US=locales/en-US.json \
  --out ../my-plugin.relune-plugin \
  --force
```

API reference lives at [docs.rs/relune-plugin-sdk](https://docs.rs/relune-plugin-sdk).
Deep host capabilities (remote requests, media, OCR, embeddings) are available in the same
SDK; the templates cover only the baseline, and both AGENTS.md files list the extension
points without duplicating signatures.

Signing keys, passphrases, and signed packages belong outside the project directory.

## Publishing (author action)

Publishing is not part of the template loop, and no template embeds publishing automation. It is
performed by the official CLI and Relune's services:

```bash
relune-plugin publish ../my-plugin.relune-plugin
```

- The first publish opens the browser once for an OAuth consent (loopback PKCE redirect). The CLI
  then keeps the authorization in the user config directory (Unix `0600`) and reuses it until
  `relune-plugin auth logout`; `relune-plugin auth status` shows the current account.
- Publishing is constrained by the account and by the plugin's first-publish public key: the key
  that first publishes a plugin id locks ownership, and later versions must be signed by it.
- Quota semantics: every request that leaves the CLI counts against the publish quota for the
  current UTC hour whether it succeeds or fails, is not refunded, and the CLI never retries
  automatically after the request was sent.
