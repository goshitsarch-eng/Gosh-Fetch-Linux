# Changelog

All notable changes to Gosh-Fetch will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2026-01-17

### Added

### Changed

### Fixed

## [2.0.0] - 2026-01-10

### Changed

#### Architecture
- Complete rewrite from Tauri/Svelte to pure Rust with native toolkit frontends
- New workspace structure with shared core library (`gosh-fetch-core`)
- GTK4/libadwaita frontend (`gosh-fetch-gtk`)

#### Core Library (gosh-fetch-core)
- UI-agnostic business logic shared across frontends
- `DownloadService` bridges tokio async runtime with UI main loops
- `EngineAdapter` provides unified gosh-dl integration
- SQLite database layer with rusqlite
- Message-based architecture using `async_channel`

#### GTK4 Frontend (gosh-fetch-gtk)
- Native GNOME experience with libadwaita
- NavigationSplitView for sidebar + content layout
- Real-time download monitoring with 1-second polling
- System tray support via ksni
- Keyboard shortcuts (Ctrl+N for new download, etc.)

### Removed
- Tauri v2 framework
- Svelte 5 frontend
- TypeScript/Node.js dependencies
- Cross-platform Windows/macOS support (now Linux-only)

## [1.1.1] - 2025-01-09

### Changed
- Updated gosh-dl engine to latest version
- Updated mainline DHT library to v6.0.1

## [1.0.0] - 2025

### Added

#### Download Features
- HTTP/HTTPS download support with multi-segment transfers
- BitTorrent protocol support (DHT, PEX, Local Peer Discovery)
- Magnet link support with metadata retrieval
- Torrent file parsing and selective file download
- Pause, resume, and cancel downloads
- Batch operations (Pause All, Resume All)
- Download queue management with configurable concurrent downloads
- Per-download speed limiting
- Custom output filename support
- Download history and persistence across sessions

#### BitTorrent
- Configurable seed ratio
- Peer monitoring and statistics
- Auto-update tracker lists from community sources
- DHT, PEX, and LPD toggle settings

#### User Interface
- Light, Dark, and System theme support
- Real-time progress tracking with speed metrics
- System tray integration with minimize-to-tray
- Native notifications on download completion

#### Settings
- Configurable download directory
- Concurrent downloads limit (1-20)
- Connections per server (1-16)
- Global download/upload speed limits
- Custom user agent selection

#### Technical
- Native Rust download engine (gosh-dl) - no external dependencies
- Cross-platform support: Windows, Linux, macOS
- SQLite database for local storage
- No telemetry or data collection
