# Product Requirements Document

## Product Overview

Gosh-Fetch is a download manager for Linux that handles HTTP/HTTPS and BitTorrent downloads without requiring external dependencies. It provides a native Qt 6 / Qt Quick interface for modern Linux desktops.

## Objectives

1. Provide a straightforward download manager that respects user privacy
2. Handle segmented HTTP downloads with resume capability
3. Support BitTorrent protocol including magnet links
4. Run as a native Linux application without Electron or web tech
5. Store all data locally with no cloud services or telemetry

## Target Users

Linux desktop users who need a download manager for:
- Large file downloads requiring resume capability
- BitTorrent downloads without a separate client
- Managing multiple concurrent downloads
- Bandwidth control through speed limits

## Functional Requirements

### Download Management

| Requirement | Status | Notes |
|-------------|--------|-------|
| Add HTTP/HTTPS downloads | Implemented | Via URL input or drag-drop |
| Add magnet links | Implemented | Paste or click magnet: links |
| Add torrent files | Implemented | File chooser dialog |
| Pause/resume downloads | Implemented | Individual and batch |
| Cancel downloads | Implemented | With optional file deletion |
| Download queue | Implemented | Configurable concurrent limit |
| Per-download speed limits | Implemented | Via download options |
| Custom filenames | Implemented | Via download options |
| Checksum verification | Implemented | MD5/SHA256 in options |
| Mirror URLs | Implemented | Fallback sources in options |
| Scheduled downloads | Not implemented | Field exists, no UI exposure |

### BitTorrent Features

| Requirement | Status | Notes |
|-------------|--------|-------|
| Magnet link support | Implemented | Metadata retrieval |
| Torrent file parsing | Implemented | With file preview |
| Selective file download | Implemented | Choose files before adding |
| DHT peer discovery | Implemented | Configurable |
| PEX peer exchange | Implemented | Configurable |
| Local Peer Discovery | Implemented | Configurable |
| Seed ratio limit | Implemented | Stop seeding after ratio |
| Auto-update trackers | Implemented | From ngosang/trackerslist |

### User Interface

| Requirement | Status | Notes |
|-------------|--------|-------|
| Downloads list view | Implemented | With filtering |
| Completed downloads history | Implemented | Persisted in database |
| Settings panel | Implemented | All configurable options |
| Real-time progress | Implemented | Speed, ETA, percentage |
| System tray | Implemented | Qt Quick system tray integration |
| Toast notifications | Implemented | Errors and completion |
| Keyboard shortcuts | Implemented | Ctrl+N, Ctrl+Shift+P/R |
| Dark/light theme | Implemented | Qt Quick controls + custom theme |

### Connection Settings

| Setting | Range | Default |
|---------|-------|---------|
| Concurrent downloads | 1-20 | 5 |
| Connections per server | 1-16 | 16 |
| Segments per download | 1-64 | 16 |
| Global download speed | 0=unlimited | Unlimited |
| Global upload speed | 0=unlimited | Unlimited |

## Non-Functional Requirements

### Performance

- UI responsive during active downloads
- Memory usage proportional to download count
- Database operations non-blocking to UI

### Reliability

- Resume interrupted downloads
- Persist download state across restarts
- Handle network failures gracefully

### Security

- No network activity without user action
- Local-only data storage
- No telemetry or analytics

### Compatibility

- Linux x86_64 and aarch64
- Qt 6 (Qt Base + Qt Quick)
- GNOME, KDE Plasma, Cinnamon, and other Linux desktops

## Success Metrics

Not applicable - no analytics collected.

## Constraints

- Single platform (Linux only)
- Single frontend (Qt Quick)
- Requires Qt 6 runtime libraries

## Future Considerations

Based on README roadmap:
- Browser extension for one-click downloads
- Scheduled downloading for off-peak hours
- Time-based bandwidth profiles
- RSS/podcast feed support
- Download categories with custom save locations
- Import/export for settings and history

## Out of Scope

- Windows/macOS support
- Mobile platforms
- Cloud synchronization
- Account/authentication systems
- Streaming protocol support (HLS, DASH)
