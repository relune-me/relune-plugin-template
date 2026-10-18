# rust-connector template

A minimal but real **connector** plugin: it scans a host-authorized directory with paginated
host reads and serves ranged content reads, then maps its carrier URIs back to original
targets. It exercises the baseline connector surface only — directory read origin (one
`FIELD` binding), snapshot scanning with pagination, ranged reads, and file targets.

## Layout

```text
manifest.json        Contract manifest (specVersion "1.0"), declares the folder binding
locales/en-US.json   Display name/description/policy
src/lib.rs           Guest implementation
tests/host_test.rs   Mock Host acceptance tests over the real component
rust-toolchain.toml  Pins Rust 1.98.1 + wasm32-unknown-unknown
```

## Commands

```bash
relune-plugin test        # build the component, then run the Mock Host test suite
relune-plugin build --release
relune-plugin key generate --out ../my-author.key        # once per author identity
relune-plugin package \
  --manifest manifest.json \
  --component target/relune-plugin/plugin.wasm \
  --key ../my-author.key \
  --locale en-US=locales/en-US.json \
  --out ../my-connector.relune-plugin \
  --force
```

## Customization

- Rename the plugin identity in `Cargo.toml` (`name`), `src/lib.rs`
  (`FINGERPRINT_LABEL`), and `manifest.json` (`id`, `version`, `publisher`).
- The `folder` config field + `directoryReadAccess` entry in `manifest.json` is the
  authorization seam: the plugin can only open the origin id (`source`) the manifest
  declares as coming from that binding.
- Remote-backed or multi-origin connectors, delta scans, and deep-link targets follow the
  same shapes; see the SDK docs for the pinned version.
- Agents: start from `AGENTS.md`.
