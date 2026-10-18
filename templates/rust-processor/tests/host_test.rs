//! Mock Host acceptance tests for the processor template.
//!
//! These tests run on the host (`cargo test`) against the real, freshly built
//! Wasm Component via the SDK Test API. `relune-plugin test` wires
//! `RELUNE_PLUGIN_COMPONENT` automatically; run this file directly only after a
//! build with that variable exported.

use relune_plugin_sdk::testing::{
    ComponentUnderTest, MockHost, PluginLogEntry, PluginLogLevel, PluginTestRuntime,
    processor::{CandidateFacts, ProcessRequest, ProcessorError, ProductValue},
};

fn runtime_and_processor() -> (PluginTestRuntime, relune_plugin_sdk::testing::ProcessorComponent) {
    let runtime = PluginTestRuntime::new().expect("test runtime");
    let component = ComponentUnderTest::from_env().expect("RELUNE_PLUGIN_COMPONENT");
    let processor = runtime.load_processor(&component).expect("load processor");
    (runtime, processor)
}

fn request() -> ProcessRequest {
    ProcessRequest {
        config_json: "{}".to_owned(),
        facts: CandidateFacts {
            origin_uri: "source://notes.txt".to_owned(),
            name: "notes.txt".to_owned(),
            mime_type: Some("text/plain".to_owned()),
            size_bytes: 42,
            source_modified_at_epoch_ms: None,
            source_anchor: None,
            normalized_suffix: "txt".to_owned(),
        },
        upstream_products: Vec::new(),
    }
}

#[test]
fn fingerprint_is_deterministic_over_the_real_component() {
    let (_runtime, processor) = runtime_and_processor();
    let host = MockHost::new();
    let first = processor
        .implementation_fingerprint(&host)
        .expect("call")
        .expect("fingerprint");
    let second = processor
        .implementation_fingerprint(&host)
        .expect("call")
        .expect("fingerprint");
    assert_eq!(first, second);
}

#[test]
fn utf8_text_is_split_into_paragraph_blocks() {
    let (_runtime, processor) = runtime_and_processor();
    let host = MockHost::new();
    host.set_input_content(b"First paragraph.\n\nSecond paragraph.\n".to_vec());

    let response = processor
        .process(&host, &request())
        .expect("call")
        .expect("process");
    assert_eq!(response.products.len(), 1);
    let blocks = match &response.products[0] {
        ProductValue::Blocks(json) => json.clone(),
        other => panic!("expected a blocks product, got {other:?}"),
    };
    assert!(blocks.contains("\"schemaVersion\":2"), "got {blocks}");
    assert_eq!(blocks.matches("\"kind\":\"PARAGRAPH\"").count(), 2);
    assert!(blocks.contains("\"key\":\"p0\""));
    assert!(blocks.contains("\"text\":\"First paragraph.\""));
    assert!(blocks.contains("\"text\":\"Second paragraph.\""));
    assert_eq!(response.raw_encoding.as_deref(), Some("UTF-8"));
    assert!(response.child_candidates.is_empty());

    assert_eq!(
        host.logs(),
        vec![
            PluginLogEntry {
                level: PluginLogLevel::Debug,
                message: "paragraph-processor invoked".to_owned(),
            },
            PluginLogEntry {
                level: PluginLogLevel::Info,
                message: "paragraph-processor produced 2 paragraph(s)".to_owned(),
            },
        ]
    );
}

#[test]
fn quotes_and_backslashes_survive_the_json_escape() {
    let (_runtime, processor) = runtime_and_processor();
    let host = MockHost::new();
    host.set_input_content(br#"He said "hi" \ now."#.to_vec());

    let response = processor
        .process(&host, &request())
        .expect("call")
        .expect("process");
    let blocks = match &response.products[0] {
        ProductValue::Blocks(json) => json.clone(),
        other => panic!("expected a blocks product, got {other:?}"),
    };
    assert!(
        blocks.contains(r#""text":"He said \"hi\" \\ now.""#),
        "got {blocks}"
    );
}

#[test]
fn missing_or_whitespace_content_is_invalid_input() {
    let (_runtime, processor) = runtime_and_processor();

    let missing = MockHost::new();
    match processor
        .process(&missing, &request())
        .expect("call")
        .expect_err("missing content must be rejected")
    {
        ProcessorError::InvalidInput => {}
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let whitespace = MockHost::new();
    whitespace.set_input_content(b"  \n\t \n".to_vec());
    match processor
        .process(&whitespace, &request())
        .expect("call")
        .expect_err("whitespace-only content must be rejected")
    {
        ProcessorError::InvalidInput => {}
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}
