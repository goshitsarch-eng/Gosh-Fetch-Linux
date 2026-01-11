# Gosh-Fetch

A Linux download manager with native GTK4, COSMIC, and Qt frontends. Built entirely in Rust with the gosh-dl download engine.

## Philosophy

Gosh apps are built with a Linux-first mindset: simplicity, transparency, and user control.

## Features

- HTTP/HTTPS segmented downloads with automatic resume
- BitTorrent protocol with DHT, PEX, and Local Peer Discovery
- Magnet link support with metadata retrieval
- Native Rust download engine (gosh-dl) with no external dependencies
- Multiple native frontends: GTK4, COSMIC, Qt6
- System tray integration with minimize-to-tray
- No telemetry, accounts, or cloud features

### Download Management
- Real-time progress tracking with speed metrics
- Pause, resume, and cancel downloads
- Batch operations (Pause All, Resume All)
- Download queue management
- Download history and persistence
- Custom output filename per download
- Per-download speed limiting

### BitTorrent Support
- Torrent file and magnet link support
- DHT, PEX, and Local Peer Discovery
- Seeder/peer count monitoring
- Configurable seed ratio
- Auto-update tracker lists from community sources
- Selective file download from torrents

### Connection Settings
- Concurrent downloads limit (1-20)
- Connections per server (1-16)
- Segments per download (1-64)
- Global download/upload speed limits
- Custom user agent support

## Download Engine

Gosh-Fetch uses [gosh-dl](https://github.com/goshitsarch-eng/gosh-dl), a native Rust download engine built specifically for this project.

### Why a Native Engine?

| Feature | gosh-dl | External Tools |
|---------|---------|----------------|
| No external binaries | Yes | No |
| Memory safe | Yes (Rust) | Varies |
| Single binary distribution | Yes | No |
| Integrated error handling | Yes | Limited |
| Custom protocol support | Easy to add | Depends |

### gosh-dl Features

- **HTTP/HTTPS**: Segmented downloads with automatic resume
- **BitTorrent**: Full protocol support with DHT, PEX, LPD
- **Async I/O**: Built on Tokio for efficient concurrent downloads
- **Progress Events**: Real-time download status via event channels

## Requirements

### All Frontends
- [Rust](https://rustup.rs/) 1.77+

### GTK4 Frontend (Default)
- GTK4 4.14+
- libadwaita 1.5+

On Debian/Ubuntu:
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

On Fedora:
```bash
sudo dnf install gtk4-devel libadwaita-devel
```

On Arch Linux:
```bash
sudo pacman -S gtk4 libadwaita
```

### COSMIC Frontend
- libcosmic (from System76)

### Qt6 Frontend
- Qt6 with QtQuick and QtQuickControls2
- cmake

See [Qt Frontend Documentation](docs/QT_FRONTEND.md) for detailed requirements.

## Building

```bash
# Development build (GTK frontend, default)
cargo build

# Production build
cargo build --release

# Run the application
cargo run

# Run tests
cargo test

# Linting
cargo clippy

# Formatting
cargo fmt
```

### Building Specific Frontends

```bash
# GTK4/libadwaita frontend (default)
cargo build -p gosh-fetch-gtk --release
cargo run -p gosh-fetch-gtk

# COSMIC desktop frontend
cargo build -p gosh-fetch-cosmic --release
cargo run -p gosh-fetch-cosmic

# Qt6/QML frontend
cargo build -p gosh-fetch-qt --release
cargo run -p gosh-fetch-qt
```

## Usage

1. **Add Download** - Click the + button and enter a URL, magnet link, or select a torrent file
2. **Monitor Progress** - View real-time speed, progress, and ETA for each download
3. **Manage Downloads** - Pause, resume, or remove downloads individually or in batch
4. **View Completed** - Access download history and open completed files

The download list auto-refreshes in real-time. Downloads use configurable multi-segment transfers for optimal performance.

## Error Handling

- **Download stalled:** The download has no active connections. Check your network or try resuming.
- **Connection failed:** Unable to reach the server. Verify the URL and your network connection.
- **Torrent has no seeds:** No peers available to download from. The torrent may be inactive.

## Privacy

- No telemetry or analytics
- No data collection
- No network activity unless explicitly initiated by you
- All data stored locally on your device

## Architecture

The project is structured as a Rust workspace:

```
Gosh-Fetch-Linux/
├── crates/
│   ├── gosh-fetch-core/     # Shared core library (UI-agnostic)
│   ├── gosh-fetch-gtk/      # GTK4/libadwaita frontend
│   ├── gosh-fetch-cosmic/   # COSMIC desktop frontend
│   └── gosh-fetch-qt/       # Qt6/QML frontend
├── migrations/              # SQLite database schema
└── Cargo.toml              # Workspace configuration
```

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed technical documentation.

## Disclaimer

This software is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). It is provided "as is", without warranty of any kind, express or implied, including but not limited to the warranties of merchantability or fitness for a particular purpose. Use at your own risk.

## License

AGPL-3.0 - See [LICENSE](LICENSE)

The gosh-dl download engine is licensed under MIT.

## Roadmap

Planned features for future releases:

- **Browser Extension** - One-click downloads from your browser
- **Download Scheduler** - Schedule downloads for off-peak hours
- **Bandwidth Scheduler** - Time-based speed limit profiles
- **RSS Feed Support** - Automatic downloads from RSS/podcast feeds
- **Download Categories** - Organize downloads by type with custom save locations
- **Import/Export** - Backup and restore download history and settings

## Contributing

Contributions welcome. Please open an issue first for major changes.
