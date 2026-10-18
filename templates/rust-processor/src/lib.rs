//! Processor template: UTF-8 plain text in, paragraph blocks out.
//!
//! The plugin is compiled to a Wasm Component for the contract world
//! `relune:client-plugin/processor@1.0.0` and only ever talks to the host through
//! the SDK's Guest API — never directly to files, network, or system time.

use relune_plugin_sdk::guest::{
    self, Content, Guest, LogLevel, ProcessRequest, ProcessResponse, ProcessorError, Session,
    blocks_response, fingerprint_hash, utf8_content_text,
};

/// Change this together with the package name in Cargo.toml and the plugin id.
const FINGERPRINT_LABEL: &[u8] = b"my-processor/0.1.0";

/// Bound on how much text the plugin reads from the host per call.
const MAX_TEXT_BYTES: u64 = 1024 * 1024;

struct ParagraphProcessor;

guest::export!(ParagraphProcessor);

impl Guest for ParagraphProcessor {
    /// Configuration-independent semantic fingerprint. Must be stable across
    /// restarts; change it only when behavior changes in a user-visible way.
    fn implementation_fingerprint() -> Result<u64, ProcessorError> {
        Ok(fingerprint_hash(FINGERPRINT_LABEL))
    }

    fn process(
        host: &Session,
        content: Option<&Content>,
        _request: ProcessRequest,
    ) -> Result<ProcessResponse, ProcessorError> {
        host.log(LogLevel::Debug, "paragraph-processor invoked");
        let Some(content) = content else {
            return Err(ProcessorError::InvalidInput);
        };
        let text = utf8_content_text(content, MAX_TEXT_BYTES)?;
        if text.trim().is_empty() {
            return Err(ProcessorError::InvalidInput);
        }
        let paragraphs = paragraph_texts(&text);
        host.log(
            LogLevel::Info,
            &format!("paragraph-processor produced {} paragraph(s)", paragraphs.len()),
        );
        Ok(blocks_response(
            paragraph_blocks_json(&paragraphs),
            Some("UTF-8".to_owned()),
        ))
    }
}

/// Split on blank-line runs; each non-empty chunk becomes one paragraph.
fn paragraph_texts(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            if !lines.is_empty() {
                paragraphs.push(lines.join("\n").trim().to_owned());
                lines.clear();
            }
        } else {
            lines.push(line);
        }
    }
    if !lines.is_empty() {
        paragraphs.push(lines.join("\n").trim().to_owned());
    }
    paragraphs.into_iter().filter(|p| !p.is_empty()).collect()
}

/// Emit the SDK block schema (`"schemaVersion":2`) with one PARAGRAPH block per
/// paragraph — the same shape SDK's `guest::blocks::paragraph_json` emits for a
/// single block. Text is conservatively JSON-escaped.
fn paragraph_blocks_json(paragraphs: &[String]) -> String {
    let mut json = String::from("{\"schemaVersion\":2,\"blocks\":[");
    for (index, paragraph) in paragraphs.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        let key = format!("p{index}");
        json.push_str("{\"key\":");
        append_json_string(&mut json, &key);
        json.push_str(",\"node\":\"text\",\"kind\":\"PARAGRAPH\",\"text\":");
        append_json_string(&mut json, paragraph);
        json.push('}');
    }
    json.push_str("]}");
    json
}

fn append_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}
