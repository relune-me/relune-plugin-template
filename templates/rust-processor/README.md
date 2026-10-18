# rust-processor template

A minimal but real **processor** plugin: the runtime hands it UTF-8 plain text, and it emits
typed paragraph blocks (SDK block schema `schemaVersion:2`, one `PARAGRAPH` block per
paragraph). It exercises the baseline processor surface only — input content, logs, and the
`blocks` output slot.

## Layout

```text
manifest.json        Contract manifest (specVersion "1.0")
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
  --out ../my-processor.relune-plugin \
  --force
```

## Customization

- Rename the plugin identity in `Cargo.toml` (`name`), `src/lib.rs`
  (`FINGERPRINT_LABEL`), and `manifest.json` (`id`, `version`, `publisher`).
- Keep `rawContent.suffixes` in sync with the input MIME/suffix classes your plugin
  actually handles (`txt` here).
- Agents: start from `AGENTS.md`.
