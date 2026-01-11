# Gosh-Fetch Architecture

This document describes the technical architecture of Gosh-Fetch.

## Overview

Gosh-Fetch is a Linux download manager built as a Rust workspace with multiple native frontends:

- **Core Library**: gosh-fetch-core (UI-agnostic business logic)
- **GTK Frontend**: gosh-fetch-gtk (GTK4/libadwaita)
- **COSMIC Frontend**: gosh-fetch-cosmic (System76 COSMIC desktop)
- **Qt Frontend**: gosh-fetch-qt (Qt6/QML via cxx-qt)
- **Download Engine**: gosh-dl (native Rust library)
- **Database**: SQLite with rusqlite

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Core Library | Rust | Shared business logic |
| GTK Frontend | GTK4 + libadwaita | GNOME desktop integration |
| COSMIC Frontend | libcosmic | Pop!_OS/COSMIC desktop |
| Qt Frontend | Qt6 + QML + cxx-qt | KDE/Qt desktop integration |
| Download Engine | gosh-dl | HTTP/BitTorrent handling |
| Database | SQLite (rusqlite) | Local data persistence |
| Async Runtime | Tokio | Concurrent download operations |

## Directory Structure

```
Gosh-Fetch-Linux/
├── crates/
│   ├── gosh-fetch-core/          # Shared core library
│   │   ├── src/
│   │   │   ├── lib.rs            # Module exports
│   │   │   ├── types.rs          # Data types (Download, Settings, etc.)
│   │   │   ├── error.rs          # Error handling
│   │   │   ├── service.rs        # DownloadService (engine bridge)
│   │   │   ├── engine_adapter.rs # gosh-dl type conversions
│   │   │   ├── utils.rs          # Utilities (TrackerUpdater, formatters)
│   │   │   └── db/
│   │   │       ├── mod.rs        # Database module exports
│   │   │       ├── connection.rs # Database initialization
│   │   │       ├── downloads.rs  # Downloads table operations
│   │   │       └── settings.rs   # Settings/Trackers operations
│   │   └── Cargo.toml
│   │
│   ├── gosh-fetch-gtk/           # GTK4/libadwaita frontend
│   │   ├── src/
│   │   │   ├── main.rs           # Application entry point
│   │   │   ├── application.rs    # GtkApplication subclass
│   │   │   ├── window/           # Main window implementation
│   │   │   ├── views/            # Page views (Downloads, Completed, Settings)
│   │   │   ├── widgets/          # Reusable widgets (DownloadRow)
│   │   │   ├── dialogs/          # Modal dialogs (AddDownloadDialog)
│   │   │   ├── models/           # GObject wrappers (DownloadObject)
│   │   │   └── tray/             # System tray (ksni)
│   │   ├── resources/            # GResource files (UI, CSS, icons)
│   │   └── Cargo.toml
│   │
│   ├── gosh-fetch-cosmic/        # COSMIC desktop frontend
│   │   ├── src/
│   │   │   ├── main.rs           # Application entry point
│   │   │   └── app.rs            # cosmic::Application implementation
│   │   └── Cargo.toml
│   │
│   └── gosh-fetch-qt/            # Qt6/QML frontend
│       ├── src/
│       │   ├── main.rs           # Application entry point
│       │   └── bridge.rs         # Rust/Qt bridge (cxx-qt)
│       ├── qml/                  # QML UI files
│       │   ├── main.qml
│       │   ├── DownloadsPage.qml
│       │   ├── CompletedPage.qml
│       │   └── SettingsPage.qml
│       └── Cargo.toml
│
├── migrations/
│   └── 001_initial.sql           # Database schema
│
├── Cargo.toml                    # Workspace configuration
└── docs/                         # Documentation
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Frontend (GTK / COSMIC / Qt)                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │   Views     │ ←→ │   State     │ ←→ │  Command Channel    │ │
│  │ (Downloads, │    │ (Downloads, │    │  (async_channel)    │ │
│  │  Settings)  │    │  Stats)     │    │                     │ │
│  └─────────────┘    └─────────────┘    └──────────┬──────────┘ │
└──────────────────────────────────────────────────┬─────────────┘
                                                   │
                    EngineCommand / UiMessage      │
                                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                      gosh-fetch-core                             │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    DownloadService                          ││
│  │        (Spawns tokio runtime in background thread)         ││
│  └─────────────────────────┬───────────────────────────────────┘│
│                            │                                    │
│           ┌────────────────┼────────────────┐                  │
│           ▼                ▼                ▼                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐        │
│  │   Database  │  │   Engine    │  │  TrackerUpdater │        │
│  │  (SQLite)   │  │   Adapter   │  │                 │        │
│  └──────┬──────┘  └────────┬────┘  └────────┬────────┘        │
│         │                  │                 │                  │
└─────────┼──────────────────┼─────────────────┼──────────────────┘
          │                  │                 │
          ▼                  ▼                 ▼
   ┌────────────┐     ┌─────────────┐   ┌──────────────┐
   │   SQLite   │     │   gosh-dl   │   │   HTTP API   │
   │  Database  │     │   Engine    │   │  (trackers)  │
   └────────────┘     └─────────────┘   └──────────────┘
```

## Key Components

### Core Library (gosh-fetch-core)

The core library provides UI-agnostic functionality that all frontends share.

#### DownloadService (service.rs)

Bridges the async download engine with UI main loops:
- Spawns a background thread with its own tokio runtime
- Receives `EngineCommand` messages from UI via async_channel
- Sends `UiMessage` updates back to UI
- Handles download lifecycle events from gosh-dl

```rust
pub enum EngineCommand {
    AddDownload { url, options },
    AddMagnet { uri, options },
    AddTorrent { data, options },
    Pause(gid),
    Resume(gid),
    Remove { gid, delete_files },
    PauseAll,
    ResumeAll,
    UpdateConfig(config),
    RefreshDownloads,
    RefreshStats,
    Shutdown,
}

pub enum UiMessage {
    DownloadAdded(Download),
    DownloadUpdated(gid, Download),
    DownloadRemoved(gid),
    DownloadCompleted(Download),
    DownloadFailed(gid, error),
    StatsUpdated(GlobalStats),
    DownloadsList(Vec<Download>),
    Error(String),
    EngineReady,
}
```

#### EngineAdapter (engine_adapter.rs)

Converts between gosh-dl types and application types:
- Wraps `Arc<DownloadEngine>`
- Provides simplified API for download operations
- Handles GID parsing (UUID format)
- Converts engine status to `Download` type

#### Database (db/)

SQLite persistence layer:
- `Database`: Thread-safe connection wrapper with `Arc<Mutex<Connection>>`
- `DownloadsDb`: CRUD operations for downloads table
- `SettingsDb`: Key-value settings storage
- `TrackersDb`: BitTorrent tracker URL management

### GTK Frontend (gosh-fetch-gtk)

Native GNOME experience using GTK4 and libadwaita.

#### Application (application.rs)

AdwApplication subclass that:
- Initializes database on activation
- Loads settings from database
- Spawns DownloadService in background thread
- Creates async_channel for command/message passing
- Sets up keyboard shortcuts (Ctrl+N, Ctrl+Shift+P, etc.)

#### Window (window/)

Main application window with:
- NavigationSplitView for sidebar + content layout
- Three main views: Downloads, Completed, Settings
- Real-time statistics display in sidebar
- Toast notifications for errors
- 1-second polling for download updates

#### Views

- **DownloadsView**: Active downloads with filtering (All/Active/Paused/Errors)
- **CompletedView**: Download history from database
- **SettingsView**: All configuration options organized in preference groups

### COSMIC Frontend (gosh-fetch-cosmic)

Native frontend for System76's COSMIC desktop using libcosmic.

Implements `cosmic::Application` trait with:
- Navigation bar for page switching
- Context drawer for about dialog
- Subscription-based UI updates
- Full settings management

### Qt Frontend (gosh-fetch-qt)

Native Qt6 experience using QML and cxx-qt for Rust interop.

- QML files define the UI
- `bridge.rs` provides Rust/Qt communication
- Same command/message pattern as other frontends

## Database Schema

### downloads

Stores download history and state for persistence.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Primary key (auto-increment) |
| gid | TEXT | Unique download identifier (UUID) |
| name | TEXT | Display name |
| url | TEXT | Source URL (HTTP downloads) |
| magnet_uri | TEXT | Magnet link (torrents) |
| info_hash | TEXT | BitTorrent info hash |
| download_type | TEXT | http/ftp/torrent/magnet |
| status | TEXT | waiting/active/paused/complete/error/removed |
| total_size | INTEGER | Total bytes |
| completed_size | INTEGER | Downloaded bytes |
| download_speed | INTEGER | Current download speed |
| upload_speed | INTEGER | Current upload speed |
| save_path | TEXT | Destination directory |
| created_at | DATETIME | Creation timestamp |
| completed_at | DATETIME | Completion timestamp |
| error_message | TEXT | Error description |
| selected_files | TEXT | Comma-separated file indices |

Indexes: `idx_downloads_status`, `idx_downloads_created`, `idx_downloads_gid`

### settings

Key-value store for configuration.

| Key | Default | Description |
|-----|---------|-------------|
| download_path | ~/Downloads | Default save directory |
| max_concurrent_downloads | 5 | Simultaneous downloads (1-20) |
| max_connections_per_server | 16 | Connections per host (1-16) |
| split_count | 16 | Segments per download |
| download_speed_limit | 0 | Global download limit (0=unlimited) |
| upload_speed_limit | 0 | Global upload limit (0=unlimited) |
| user_agent | gosh-dl/0.1.0 | HTTP user agent |
| enable_notifications | true | Show completion notifications |
| close_to_tray | true | Minimize to tray on close |
| bt_enable_dht | true | BitTorrent DHT |
| bt_enable_pex | true | BitTorrent Peer Exchange |
| bt_enable_lpd | true | Local Peer Discovery |
| bt_max_peers | 55 | Max peers per torrent |
| bt_seed_ratio | 1.0 | Seed ratio before stopping |
| auto_update_trackers | true | Auto-fetch tracker lists |
| delete_files_on_remove | false | Delete files when removing download |

### trackers

BitTorrent tracker URLs.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Primary key |
| url | TEXT | Tracker URL (unique) |
| enabled | INTEGER | Is tracker enabled (boolean) |
| last_checked | DATETIME | Last check timestamp |
| is_working | INTEGER | Last known status (boolean) |

### tracker_meta

Metadata for tracker list updates (singleton row with id=1).

| Column | Type | Description |
|--------|------|-------------|
| last_updated | DATETIME | Last fetch time |
| source_url | TEXT | Tracker list source URL |

## Download Engine (gosh-dl)

gosh-dl is a native Rust download engine providing:

- **HTTP/HTTPS**: Multi-segment parallel downloads with resume support
- **BitTorrent**: Full protocol with DHT, PEX, LPD
- **Magnet Links**: Metadata retrieval and download

Key characteristics:
- Async I/O with Tokio
- Event-based progress updates via broadcast channels
- Memory-safe Rust implementation
- No external binary dependencies
- Session persistence via SQLite

## Communication Pattern

All frontends use the same architecture:

1. **UI Thread**: Runs the native toolkit event loop (GTK main loop, Qt event loop, etc.)
2. **Background Thread**: Runs tokio runtime with DownloadService
3. **async_channel**: Bidirectional communication between threads
   - `Sender<EngineCommand>`: UI sends commands to engine
   - `Receiver<UiMessage>`: UI receives updates from engine

The UI polls for updates (1-second interval) by sending `RefreshDownloads` and `RefreshStats` commands.

## Security Considerations

- All data stored locally (no cloud services)
- No telemetry or analytics
- SQLite database in `~/.local/share/io.github.gosh.Fetch/`
- Settings stored per-user
- Engine database stored separately for session persistence
