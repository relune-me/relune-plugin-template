//! Connector template: paginated scan + ranged reads over a host-authorized directory.
//!
//! The plugin is compiled to a Wasm Component for the contract world
//! `relune:client-plugin/connector@1.0.0`. Directory access is authorized by the
//! host per the manifest's `directoryReadAccess` capability and always flows
//! through the SDK's Guest API.

use relune_plugin_sdk::guest::connector::{
    self, ConnectorError, ContentChunk, ContentReadRequest, Guest, HostError, LogLevel,
    OriginalFileTarget, OriginalTarget, ScanEntry, ScanMode, ScanPage, ScanProgress, ScanRequest,
    Session, fingerprint_hash,
};

/// Change this together with the package name in Cargo.toml and the plugin id.
const FINGERPRINT_LABEL: &[u8] = b"my-connector/0.1.0";

/// Capability id declared for `directoryReadAccess` in manifest.json.
const ORIGIN_CAPABILITY_ID: &str = "source";

/// Scheme prefix for carrier URIs produced (and later resolved) by this plugin.
const CARRIER_URI_PREFIX: &str = "source://";

/// Entries listed per host `list_entries` page; larger directories paginate.
const SCAN_PAGE_SIZE: u32 = 64;

struct LocalFolderConnector;

connector::export!(LocalFolderConnector);

impl Guest for LocalFolderConnector {
    /// Configuration-independent semantic fingerprint. Must be stable across
    /// restarts; change it only when behavior changes in a user-visible way.
    fn implementation_fingerprint() -> Result<u64, ConnectorError> {
        Ok(fingerprint_hash(FINGERPRINT_LABEL))
    }

    fn scan(host: &Session, request: ScanRequest) -> Result<ScanPage, ConnectorError> {
        host.log(LogLevel::Debug, "local-folder-connector scan start");
        let origin = host
            .open_read_origin(ORIGIN_CAPABILITY_ID)
            .map_err(map_host_error)?;
        // One host page per scan call: the returned cursor comes back from the
        // host as `ScanProgress::More`, and the next scan resumes from it.
        let listed = origin
            .list_entries(None, request.resume_cursor.as_deref(), SCAN_PAGE_SIZE)
            .map_err(map_host_error)?;
        host.log(
            LogLevel::Info,
            &format!(
                "local-folder-connector scan listed {} entry/entries",
                listed.entries.len(),
            ),
        );
        let entries = listed
            .entries
            .into_iter()
            .map(|entry| ScanEntry {
                uri: format!("{CARRIER_URI_PREFIX}{}", entry.id),
                name: entry.name,
                mime_type: None,
                size_bytes: entry.size_bytes,
                modified_at_epoch_ms: entry.modified_at_epoch_ms,
                fingerprint: entry_fingerprint(
                    &entry.id,
                    entry.size_bytes,
                    entry.modified_at_epoch_ms,
                    entry.is_directory,
                ),
                is_directory: entry.is_directory,
                is_deleted: false,
            })
            .collect();
        // A full listing without an incremental anchor is an authoritative
        // snapshot; an anchored re-scan only reports changes.
        let mode = if request.last_successful_at_epoch_ms.is_none() {
            ScanMode::Snapshot
        } else {
            ScanMode::Delta
        };
        let progress = match listed.next_cursor {
            Some(cursor) => ScanProgress::More(cursor),
            None => ScanProgress::Complete,
        };
        Ok(ScanPage {
            entries,
            mode,
            progress,
            candidate_sync_token: None,
        })
    }

    fn read_content(
        host: &Session,
        request: ContentReadRequest,
    ) -> Result<ContentChunk, ConnectorError> {
        let entry_id = request
            .carrier_uri
            .strip_prefix(CARRIER_URI_PREFIX)
            .filter(|entry_id| !entry_id.is_empty())
            .ok_or(ConnectorError::InvalidConfig)?;
        host.log(
            LogLevel::Debug,
            &format!(
                "local-folder-connector read '{entry_id}' offset={} max-bytes={}",
                request.offset, request.max_bytes,
            ),
        );
        let origin = host
            .open_read_origin(ORIGIN_CAPABILITY_ID)
            .map_err(map_host_error)?;
        let content = origin
            .read(entry_id, request.offset, request.max_bytes)
            .map_err(map_host_error)?;
        Ok(ContentChunk {
            bytes: content.bytes,
            is_last: content.is_last,
        })
    }

    fn resolve_original_target(
        _config_json: String,
        carrier_uri: String,
        _metadata_json: Option<String>,
    ) -> Result<Option<OriginalTarget>, ConnectorError> {
        if !carrier_uri.starts_with(CARRIER_URI_PREFIX) {
            return Ok(None);
        }
        Ok(Some(OriginalTarget::File(OriginalFileTarget {
            uri: carrier_uri,
            mime_type: None,
        })))
    }
}

/// Host failures keep their stable code and retryable flag; never fold them into
/// plugin-specific enums.
fn map_host_error(error: HostError) -> ConnectorError {
    ConnectorError::Host(error)
}

/// Stable, content-independent entry fingerprint over id + size + modified time.
fn entry_fingerprint(
    id: &str,
    size_bytes: Option<u64>,
    modified_at_epoch_ms: Option<i64>,
    is_directory: bool,
) -> String {
    let mut input = Vec::with_capacity(id.len() + 27);
    input.extend_from_slice(&(id.len() as u64).to_le_bytes());
    input.extend_from_slice(id.as_bytes());
    match size_bytes {
        Some(size) => {
            input.push(1);
            input.extend_from_slice(&size.to_le_bytes());
        }
        None => input.push(0),
    }
    match modified_at_epoch_ms {
        Some(modified_at) => {
            input.push(1);
            input.extend_from_slice(&modified_at.to_le_bytes());
        }
        None => input.push(0),
    }
    input.push(u8::from(is_directory));
    format!("{:016x}", fingerprint_hash(&input))
}
