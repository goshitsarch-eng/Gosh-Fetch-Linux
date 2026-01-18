# Technical Specification

Gosh-Fetch is a native Linux download manager written in Rust, providing HTTP/HTTPS segmented downloads and BitTorrent support through a clean GTK4/libadwaita interface.

## System Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    GTK4/libadwaita Frontend                         │
│  GoshFetchApplication → GoshFetchWindow → Views (Downloads,        │
│                                           Completed, Settings)      │
└─────────────────────────────────────┬───────────────────────────────┘
                                      │ async_channel
                                      │ EngineCommand ↓ / UiMessage ↑
┌─────────────────────────────────────┴───────────────────────────────┐
│                         gosh-fetch-core                             │
│  DownloadService (background thread with tokio runtime)             │
│  EngineAdapter (gosh-dl type conversion)                            │
│  Database (SQLite via rusqlite)                                     │
└─────────────────────────────────────┬───────────────────────────────┘
                                      │
                          ┌───────────┴───────────┐
                          ▼                       ▼
                   ┌─────────────┐         ┌─────────────┐
                   │   gosh-dl   │         │   SQLite    │
                   │   Engine    │         │  Database   │
                   └─────────────┘         └─────────────┘
```

### Thread Model

The application runs two main threads:

1. **UI Thread**: GTK4 main loop handling all user interface operations
2. **Engine Thread**: Tokio runtime hosting the download engine and event processing

Communication uses bounded async channels (capacity 100 messages):
- `Sender<EngineCommand>`: UI → Engine
- `Receiver<UiMessage>`: Engine → UI

The UI polls for updates every 1 second by sending `RefreshDownloads` and `RefreshStats` commands.

## Technology Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 1.77+ |
| UI Framework | GTK4 + libadwaita | 4.14+ / 1.5+ |
| Async Runtime | Tokio | 1.x |
| Download Engine | gosh-dl | Git HEAD |
| Database | SQLite (rusqlite) | 0.32 |
| IPC | async_channel | 2.x |
| System Tray | ksni | 0.2 |

## Data Models

### Download

Represents an active or historical download:

```rust
struct Download {
    id: i64,                           // Database primary key
    gid: String,                       // UUID identifying the download
    name: String,                      // Display name
    url: Option<String>,               // HTTP source URL
    magnet_uri: Option<String>,        // Magnet link
    info_hash: Option<String>,         // BitTorrent info hash
    download_type: DownloadType,       // Http | Ftp | Torrent | Magnet
    status: DownloadState,             // Active | Waiting | Paused | Complete | Error | Removed
    total_size: u64,                   // Bytes
    completed_size: u64,               // Bytes downloaded
    download_speed: u64,               // Bytes/second
    upload_speed: u64,                 // Bytes/second
    save_path: String,                 // Destination directory
    created_at: String,                // ISO 8601 timestamp
    completed_at: Option<String>,      // ISO 8601 timestamp
    error_message: Option<String>,     // Error details
    connections: u32,                  // Active HTTP connections
    seeders: u32,                      // Connected BitTorrent seeders
    selected_files: Option<Vec<usize>>,// Torrent file indices
}
```

### Settings

Application configuration with sensible defaults:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| download_path | String | ~/Downloads | Default save directory |
| max_concurrent_downloads | u32 | 5 | Simultaneous downloads (1-20) |
| max_connections_per_server | u32 | 16 | Connections per host |
| split_count | u32 | 16 | Segments per download |
| download_speed_limit | u64 | 0 | Bytes/sec (0=unlimited) |
| upload_speed_limit | u64 | 0 | Bytes/sec (0=unlimited) |
| user_agent | String | gosh-dl/0.1.0 | HTTP user agent |
| enable_notifications | bool | true | Desktop notifications |
| close_to_tray | bool | true | Minimize to tray on close |
| bt_enable_dht | bool | true | BitTorrent DHT |
| bt_enable_pex | bool | true | Peer Exchange |
| bt_enable_lpd | bool | true | Local Peer Discovery |
| bt_max_peers | u32 | 55 | Max peers per torrent |
| bt_seed_ratio | f64 | 1.0 | Seed ratio threshold |
| auto_update_trackers | bool | true | Auto-fetch tracker lists |
| delete_files_on_remove | bool | false | Delete files on removal |
| proxy_enabled | bool | false | Enable proxy |
| proxy_type | String | http | http/https/socks5 |
| proxy_url | String | "" | Proxy address |
| proxy_user | Option | None | Proxy username |
| proxy_pass | Option | None | Proxy password |
| min_segment_size | u32 | 1024 | Min segment size (KB) |
| bt_preallocation | String | sparse | none/sparse/full |

## Database Schema

SQLite database at `~/.local/share/io.github.gosh.Fetch/gosh-fetch.db`.

### Tables

**downloads**: Download records and history
- Indexed on: status, created_at, gid

**settings**: Key-value configuration store

**trackers**: BitTorrent tracker URLs with enabled/working status

**tracker_meta**: Singleton row tracking last tracker list update

Engine session data stored separately at `~/.local/share/io.github.gosh.Fetch/engine.db`.

## API Surface

### Core Library Exports

```rust
// Database
pub use db::{get_db_path, init_database, Database, DownloadsDb, SettingsDb, TrackersDb};

// Engine integration
pub use engine_adapter::{EngineAdapter, PeerInfo, TorrentFileInfo};

// Error handling
pub use error::{Error, Result};

// Service layer
pub use service::{DownloadService, EngineCommand, UiMessage};

// All types
pub use types::*;

// Utilities
pub use utils::{calculate_progress, format_bytes, format_eta, format_speed, TrackerUpdater};

// Re-exported from gosh-dl
pub use gosh_dl::EngineConfig;
```

### Command/Message Protocol

Frontend sends:
- AddDownload, AddMagnet, AddTorrent
- Pause, Resume, Remove
- PauseAll, ResumeAll
- UpdateConfig, RefreshDownloads, RefreshStats
- Shutdown

Engine responds:
- DownloadAdded, DownloadUpdated, DownloadRemoved
- DownloadCompleted, DownloadFailed
- StatsUpdated, DownloadsList
- Error, EngineReady

## Performance Characteristics

- Release profile: LTO enabled, single codegen unit, optimized for size
- Channel buffers: 100 messages
- UI polling interval: 1 second
- Tracker update interval: 24 hours
- Completed history limit: 100 entries (in-memory)

## Security Model

- All data stored locally under `~/.local/share/io.github.gosh.Fetch/`
- No telemetry, analytics, or network requests except for:
  - User-initiated downloads
  - BitTorrent DHT/PEX/LPD (if enabled)
  - Tracker list fetches from ngosang/trackerslist (if enabled)
- No authentication system
- Single-user operation

## Build Requirements

**Required**: Rust 1.77+, GTK4 4.14+, libadwaita 1.5+

**Build targets**: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu

**Packaging formats**: DEB, RPM, AppImage, Flatpak

## Known Limitations

- Linux only (no Windows/macOS)
- GTK frontend only (COSMIC and Qt frontends not yet implemented)
- No FTP protocol support (DownloadType::Ftp exists in types but engine doesn't support it)
- No scheduled downloads (field exists in DownloadOptions but not exposed in UI)
- No browser extension integration
- No RSS/podcast feed support
