# Gosh-Fetch Core API Reference

This document describes the public API of the `gosh-fetch-core` crate, which provides the shared functionality for all Gosh-Fetch frontends.

## Table of Contents

- [DownloadService](#downloadservice)
- [EngineAdapter](#engineadapter)
- [Database Operations](#database-operations)
- [Types](#types)
- [Utilities](#utilities)

---

## DownloadService

The `DownloadService` bridges the async gosh-dl download engine with UI main loops.

### Creating a Service

```rust
use gosh_fetch_core::{DownloadService, Settings};

// Create service with settings
let settings = Settings::default();
let service = DownloadService::new_async(&settings).await?;
```

### Spawning the Service

The service runs in a background thread with its own tokio runtime:

```rust
use async_channel::{bounded, Sender, Receiver};
use gosh_fetch_core::{EngineCommand, UiMessage};

// Create communication channels
let (ui_sender, ui_receiver) = bounded::<UiMessage>(100);
let (cmd_sender, cmd_receiver) = bounded::<EngineCommand>(100);

// Spawn the service (consumes self)
service.spawn(ui_sender, cmd_receiver);

// Now use cmd_sender to send commands and ui_receiver to receive updates
```

### EngineCommand

Commands sent from the UI to the engine:

```rust
pub enum EngineCommand {
    /// Add an HTTP/HTTPS download
    AddDownload {
        url: String,
        options: Option<DownloadOptions>,
    },

    /// Add a magnet link
    AddMagnet {
        uri: String,
        options: Option<DownloadOptions>,
    },

    /// Add a torrent file
    AddTorrent {
        data: Vec<u8>,
        options: Option<DownloadOptions>,
    },

    /// Pause a download by GID
    Pause(String),

    /// Resume a download by GID
    Resume(String),

    /// Remove a download
    Remove {
        gid: String,
        delete_files: bool,
    },

    /// Pause all downloads
    PauseAll,

    /// Resume all downloads
    ResumeAll,

    /// Update engine configuration
    UpdateConfig(EngineConfig),

    /// Request current downloads list
    RefreshDownloads,

    /// Request global statistics
    RefreshStats,

    /// Shutdown the service
    Shutdown,
}
```

### UiMessage

Messages sent from the engine to the UI:

```rust
pub enum UiMessage {
    /// A download was added
    DownloadAdded(Download),

    /// A download was updated (gid, updated download)
    DownloadUpdated(String, Download),

    /// A download was removed (gid)
    DownloadRemoved(String),

    /// A download completed
    DownloadCompleted(Download),

    /// A download failed (gid, error message)
    DownloadFailed(String, String),

    /// Global stats updated
    StatsUpdated(GlobalStats),

    /// Full downloads list
    DownloadsList(Vec<Download>),

    /// Error message
    Error(String),

    /// Engine initialized and ready
    EngineReady,
}
```

---

## EngineAdapter

The `EngineAdapter` wraps gosh-dl's `DownloadEngine` and converts between engine types and application types.

### Methods

```rust
impl EngineAdapter {
    /// Create a new adapter with the given engine
    pub fn new(engine: Arc<DownloadEngine>) -> Self;

    /// Get a reference to the underlying engine
    pub fn engine(&self) -> &Arc<DownloadEngine>;

    /// Add an HTTP download
    pub async fn add_download(
        &self,
        url: String,
        options: Option<DownloadOptions>,
    ) -> Result<String, EngineError>;

    /// Add multiple downloads
    pub async fn add_urls(
        &self,
        urls: Vec<String>,
        options: Option<DownloadOptions>,
    ) -> Result<Vec<String>, EngineError>;

    /// Add a torrent file
    pub async fn add_torrent(
        &self,
        torrent_data: &[u8],
        options: Option<DownloadOptions>,
    ) -> Result<String, EngineError>;

    /// Add a magnet link
    pub async fn add_magnet(
        &self,
        magnet_uri: &str,
        options: Option<DownloadOptions>,
    ) -> Result<String, EngineError>;

    /// Pause a download
    pub async fn pause(&self, gid: &str) -> Result<(), EngineError>;

    /// Pause all downloads
    pub async fn pause_all(&self) -> Result<(), EngineError>;

    /// Resume a download
    pub async fn resume(&self, gid: &str) -> Result<(), EngineError>;

    /// Resume all downloads
    pub async fn resume_all(&self) -> Result<(), EngineError>;

    /// Remove a download
    pub async fn remove(
        &self,
        gid: &str,
        delete_files: bool,
    ) -> Result<(), EngineError>;

    /// Get status of a single download
    pub fn get_status(&self, gid: &str) -> Option<Download>;

    /// Get all downloads
    pub fn get_all(&self) -> Vec<Download>;

    /// Get active downloads
    pub fn get_active(&self) -> Vec<Download>;

    /// Get global statistics
    pub fn get_global_stats(&self) -> GlobalStats;

    /// Set speed limits
    pub fn set_speed_limit(
        &self,
        download_limit: Option<u64>,
        upload_limit: Option<u64>,
    ) -> Result<(), EngineError>;

    /// Get torrent files for a download
    pub fn get_torrent_files(&self, gid: &str) -> Option<Vec<TorrentFileInfo>>;

    /// Get peer information for a torrent
    pub fn get_peers(&self, gid: &str) -> Option<Vec<PeerInfo>>;

    /// Update engine configuration
    pub fn update_config(&self, config: EngineConfig) -> Result<(), EngineError>;

    /// Get current engine configuration
    pub fn get_config(&self) -> EngineConfig;
}
```

---

## Database Operations

### Initialization

```rust
use gosh_fetch_core::{init_database, get_db_path, Database};

// Get the database path
let path = get_db_path();
// ~/.local/share/io.github.gosh.Fetch/gosh-fetch.db

// Initialize database with schema
let db = init_database()?;
```

### DownloadsDb

Operations for the downloads table:

```rust
use gosh_fetch_core::DownloadsDb;

// Save a download
let id = DownloadsDb::save(&db, &download)?;

// Get by GID
let download = DownloadsDb::get_by_gid(&db, "uuid-string")?;

// Get completed downloads
let completed = DownloadsDb::get_completed(&db, 100)?;

// Get incomplete downloads (for restoration)
let incomplete = DownloadsDb::get_incomplete(&db)?;

// Update status
DownloadsDb::update_status(&db, "gid", DownloadState::Paused)?;

// Mark as completed
DownloadsDb::mark_completed(&db, "gid", "2024-01-15T10:30:00Z")?;

// Delete a record
DownloadsDb::delete(&db, "gid")?;

// Clear all completed downloads
DownloadsDb::clear_history(&db)?;

// Count completed downloads
let count = DownloadsDb::count_completed(&db)?;
```

### SettingsDb

Operations for the settings table:

```rust
use gosh_fetch_core::{SettingsDb, Settings};

// Load all settings
let settings = SettingsDb::load(&db)?;

// Save all settings
SettingsDb::save(&db, &settings)?;

// Get a single setting
let value = SettingsDb::get(&db, "download_path")?;

// Set a single setting
SettingsDb::set(&db, "download_path", "/home/user/Downloads")?;
```

### TrackersDb

Operations for the trackers table:

```rust
use gosh_fetch_core::TrackersDb;

// Get enabled trackers
let trackers = TrackersDb::get_enabled(&db)?;

// Replace all trackers
TrackersDb::replace_all(&db, &["udp://tracker1", "udp://tracker2"])?;

// Get last update time
let last_updated = TrackersDb::get_last_updated(&db)?;
```

---

## Types

### Download

```rust
pub struct Download {
    pub id: i64,                           // Database ID
    pub gid: String,                       // Engine GID (UUID)
    pub name: String,                      // Display name
    pub url: Option<String>,               // Source URL
    pub magnet_uri: Option<String>,        // Magnet link
    pub info_hash: Option<String>,         // BitTorrent info hash
    pub download_type: DownloadType,       // Type of download
    pub status: DownloadState,             // Current state
    pub total_size: u64,                   // Total bytes
    pub completed_size: u64,               // Downloaded bytes
    pub download_speed: u64,               // Bytes per second
    pub upload_speed: u64,                 // Bytes per second
    pub save_path: String,                 // Destination directory
    pub created_at: String,                // ISO 8601 timestamp
    pub completed_at: Option<String>,      // ISO 8601 timestamp
    pub error_message: Option<String>,     // Error description
    pub connections: u32,                  // Active connections
    pub seeders: u32,                      // Connected seeders
    pub selected_files: Option<Vec<usize>>, // Selected file indices
}
```

### DownloadType

```rust
pub enum DownloadType {
    Http,
    Ftp,
    Torrent,
    Magnet,
}
```

### DownloadState

```rust
pub enum DownloadState {
    Active,    // Currently downloading
    Waiting,   // Queued
    Paused,    // Paused by user
    Complete,  // Finished
    Error,     // Failed
    Removed,   // Removed from engine
}
```

### DownloadOptions

```rust
pub struct DownloadOptions {
    pub dir: Option<String>,                    // Save directory
    pub out: Option<String>,                    // Output filename
    pub max_connection_per_server: Option<String>, // Connections per server
    pub user_agent: Option<String>,             // HTTP user agent
    pub referer: Option<String>,                // HTTP referer
    pub header: Option<Vec<String>>,            // Custom headers
    pub select_file: Option<String>,            // Torrent file indices
    pub seed_ratio: Option<String>,             // Seed ratio
    pub max_download_limit: Option<String>,     // Download speed limit
    pub max_upload_limit: Option<String>,       // Upload speed limit
}
```

### GlobalStats

```rust
pub struct GlobalStats {
    pub download_speed: u64,  // Total download speed
    pub upload_speed: u64,    // Total upload speed
    pub num_active: u32,      // Active downloads
    pub num_waiting: u32,     // Queued downloads
    pub num_stopped: u32,     // Stopped downloads
}
```

### Settings

```rust
pub struct Settings {
    pub download_path: String,
    pub max_concurrent_downloads: u32,
    pub max_connections_per_server: u32,
    pub split_count: u32,
    pub download_speed_limit: u64,
    pub upload_speed_limit: u64,
    pub user_agent: String,
    pub enable_notifications: bool,
    pub close_to_tray: bool,
    pub bt_enable_dht: bool,
    pub bt_enable_pex: bool,
    pub bt_enable_lpd: bool,
    pub bt_max_peers: u32,
    pub bt_seed_ratio: f64,
    pub auto_update_trackers: bool,
    pub delete_files_on_remove: bool,
}
```

### TorrentInfo

```rust
pub struct TorrentInfo {
    pub name: String,
    pub info_hash: String,
    pub total_size: u64,
    pub files: Vec<TorrentFileEntry>,
    pub comment: Option<String>,
    pub creation_date: Option<i64>,
    pub announce_list: Vec<String>,
}

pub struct TorrentFileEntry {
    pub index: usize,
    pub path: String,
    pub length: u64,
}
```

### MagnetInfo

```rust
pub struct MagnetInfo {
    pub name: Option<String>,
    pub info_hash: String,
    pub trackers: Vec<String>,
}
```

---

## Utilities

### Formatting Functions

```rust
use gosh_fetch_core::{format_bytes, format_speed, format_eta, calculate_progress};

// Format bytes to human-readable
format_bytes(1_500_000);  // "1.43 MB"

// Format speed
format_speed(500_000);    // "488.28 KB/s"

// Format ETA
format_eta(3600, 100);    // "36s"

// Calculate progress percentage
calculate_progress(750, 1000);  // 0.75
```

### TrackerUpdater

```rust
use gosh_fetch_core::TrackerUpdater;

let mut updater = TrackerUpdater::new();

// Check if update needed (24-hour interval)
if updater.needs_update() {
    let trackers = updater.fetch_trackers().await?;
}

// Get cached trackers
let trackers = updater.get_trackers();

// Manually set trackers
updater.set_trackers(vec!["udp://tracker.example.com:1234".to_string()]);
```

### User Agent Presets

```rust
use gosh_fetch_core::get_user_agent_presets;

let presets = get_user_agent_presets();
// Returns Vec<(&str, &str)> of (name, user_agent) tuples:
// - ("gosh-dl/0.1.0", "gosh-dl/0.1.0")
// - ("Chrome (Windows)", "Mozilla/5.0 ...")
// - ("Firefox (Linux)", "Mozilla/5.0 ...")
// - etc.
```

---

## Error Handling

```rust
use gosh_fetch_core::{Error, Result};

pub enum Error {
    Engine(String),
    EngineNotInitialized,
    Database(String),
    Io(std::io::Error),
    Serialization(serde_json::Error),
    InvalidInput(String),
    NotFound(String),
    Network(String),
    Sqlite(rusqlite::Error),
    Channel(String),
}

// All functions return Result<T> = std::result::Result<T, Error>
```
