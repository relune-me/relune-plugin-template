//! Mock Host acceptance tests for the connector template.
//!
//! These tests run on the host (`cargo test`) against the real, freshly built
//! Wasm Component via the SDK Test API. `relune-plugin test` wires
//! `RELUNE_PLUGIN_COMPONENT` automatically; run this file directly only after a
//! build with that variable exported.

use relune_plugin_sdk::testing::connector::{
    ContentReadRequest, ConnectorError, HostErrorCode, OriginalTarget, ScanMode, ScanProgress,
    ScanRequest,
};
use relune_plugin_sdk::testing::{
    ComponentUnderTest, DirectoryCall, DirectoryEntry, DirectoryOrigin, MockHost, PluginLogEntry,
    PluginLogLevel, PluginTestRuntime,
};

fn runtime_and_connector() -> (PluginTestRuntime, relune_plugin_sdk::testing::ConnectorComponent) {
    let runtime = PluginTestRuntime::new().expect("test runtime");
    let component = ComponentUnderTest::from_env().expect("RELUNE_PLUGIN_COMPONENT");
    let connector = runtime.load_connector(&component).expect("load connector");
    (runtime, connector)
}

fn scan_request(config: &str, resume_cursor: Option<String>) -> ScanRequest {
    ScanRequest {
        config_json: config.to_owned(),
        last_successful_at_epoch_ms: None,
        successful_sync_token: None,
        resume_cursor,
    }
}

fn read_request(carrier_uri: &str, offset: u64, max_bytes: u32) -> ContentReadRequest {
    ContentReadRequest {
        config_json: "{}".to_owned(),
        carrier_uri: carrier_uri.to_owned(),
        offset,
        max_bytes,
    }
}

fn three_files() -> DirectoryOrigin {
    DirectoryOrigin::new()
        .with_entry(DirectoryEntry::file("a.txt", "a.txt", b"alpha".to_vec()))
        .with_entry(DirectoryEntry::file("b.txt", "b.txt", b"beta".to_vec()))
        .with_entry(DirectoryEntry::directory("sub", "sub"))
}

#[test]
fn fingerprint_is_deterministic_over_the_real_component() {
    let (_runtime, connector) = runtime_and_connector();
    let host = MockHost::new();
    let first = connector
        .implementation_fingerprint(&host)
        .expect("call")
        .expect("fingerprint");
    let second = connector
        .implementation_fingerprint(&host)
        .expect("call")
        .expect("fingerprint");
    assert_eq!(first, second);
}

#[test]
fn scan_pages_a_large_directory_through_the_host_cursor() {
    let (_runtime, connector) = runtime_and_connector();
    let host = MockHost::new();
    // 65 entries with the template's SCAN_PAGE_SIZE (64) force real pagination.
    let mut origin = DirectoryOrigin::new();
    for index in 0..65 {
        let id = format!("f{index:03}");
        origin = origin.with_entry(DirectoryEntry::file(&id, &id, b"x".to_vec()));
    }
    host.register_directory_origin("source", origin);

    let page = connector
        .scan(&host, &scan_request("{}", None))
        .expect("call")
        .expect("first page");
    assert_eq!(page.entries.len(), 64);
    assert_eq!(page.entries[0].uri, "source://f000");
    assert_eq!(page.entries[0].name, "f000");
    assert!(!page.entries[0].is_directory);
    assert!(matches!(page.mode, ScanMode::Snapshot));
    let ScanProgress::More(cursor) = page.progress else {
        panic!("first page must carry a cursor for the next scan")
    };
    assert!(matches!(
        host.logs().first(),
        Some(PluginLogEntry {
            level: PluginLogLevel::Debug,
            ..
        })
    ));

    let final_page = connector
        .scan(&host, &scan_request("{}", Some(cursor)))
        .expect("call")
        .expect("final page");
    assert_eq!(final_page.entries.len(), 1);
    assert_eq!(final_page.entries[0].uri, "source://f064");
    assert!(
        matches!(final_page.progress, ScanProgress::Complete),
        "second page must complete the scan"
    );

    let calls = host.directory_calls();
    assert_eq!(
        calls.first(),
        Some(&DirectoryCall::Open {
            source_id: "source".to_owned(),
        })
    );
    let list_calls = calls
        .iter()
        .filter(|call| matches!(call, DirectoryCall::ListEntries { .. }))
        .count();
    assert_eq!(list_calls, 2, "one host page per scan call");
}

#[test]
fn read_content_passes_through_ranges_and_eof() {
    let (_runtime, connector) = runtime_and_connector();
    let host = MockHost::new();
    host.register_directory_origin("source", three_files());

    let tail = connector
        .read_content(&host, &read_request("source://a.txt", 1, 4096))
        .expect("call")
        .expect("tail chunk");
    assert_eq!(tail.bytes, b"lpha");
    assert!(tail.is_last);

    let eof = connector
        .read_content(&host, &read_request("source://a.txt", 5, 10))
        .expect("call")
        .expect("eof chunk");
    assert!(eof.bytes.is_empty());
    assert!(eof.is_last);
}

#[test]
fn unauthorized_and_malformed_reads_fail_structurally() {
    let (_runtime, connector) = runtime_and_connector();

    // No origin registered: the Mock Host denies the capability up front.
    let denied = MockHost::new();
    match connector
        .scan(&denied, &scan_request("{}", None))
        .expect("call")
        .expect_err("unconfigured origin must be denied")
    {
        ConnectorError::Host(error) => {
            assert_eq!(error.code, HostErrorCode::Denied);
            assert!(!error.retryable);
        }
        other => panic!("expected structured denied, got {other:?}"),
    }

    // Registered origin, but a carrier URI without the plugin's prefix.
    let host = MockHost::new();
    host.register_directory_origin("source", three_files());
    match connector
        .read_content(&host, &read_request("wrong://a.txt", 0, 10))
        .expect("call")
        .expect_err("foreign carrier URI must be invalid config")
    {
        ConnectorError::InvalidConfig => {}
        other => panic!("expected InvalidConfig, got {other:?}"),
    }
}

#[test]
fn resolve_original_target_maps_plugin_carrier_uris() {
    let (_runtime, connector) = runtime_and_connector();

    let resolved = connector
        .resolve_original_target("{}", "source://a.txt", None)
        .expect("call")
        .expect("target");
    match resolved {
        Some(OriginalTarget::File(file)) => {
            assert_eq!(file.uri, "source://a.txt");
            assert_eq!(file.mime_type, None);
        }
        other => panic!("expected a file target, got {other:?}"),
    }

    let outside = connector
        .resolve_original_target("{}", "https://example.com/doc", None)
        .expect("call")
        .expect("no target");
    assert!(outside.is_none());
}
