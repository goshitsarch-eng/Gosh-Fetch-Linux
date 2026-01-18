# Gosh Fetch

A native Linux download manager built entirely in Rust. Gosh Fetch handles both HTTP/HTTPS downloads and BitTorrent with a clean GTK4/libadwaita interface.

## Philosophy

Gosh apps take a Linux-first approach: keep it simple, respect the user, and stay out of the way. No accounts, no telemetry, no cloud sync. Your downloads, your machine, your business.

## Features

Gosh Fetch supports segmented HTTP/HTTPS downloads with automatic resume, so interrupted transfers pick up where they left off. The BitTorrent implementation includes DHT, PEX, and Local Peer Discovery for finding peers without relying solely on trackers. Magnet links work out of the box.

The download manager tracks progress in real time with speed metrics and ETA. You can pause, resume, or cancel individual downloads, or use batch operations to control everything at once. Each download supports custom filenames and per-download speed limits. The app integrates with your system tray and remembers your download history between sessions.

For torrents, you get seeder and peer counts, configurable seed ratios, and selective file downloading. Tracker lists can auto-update from community sources.

Connection settings give you control over concurrent downloads (1 to 20), connections per server (1 to 16), and segments per download (1 to 64). Global speed limits keep your bandwidth in check.

## Download Engine

Gosh Fetch uses [gosh-dl](https://github.com/goshitsarch-eng/gosh-dl), a native Rust download engine built specifically for this project. Unlike wrappers around aria2 or wget, gosh-dl compiles into a single binary with no external dependencies. It's memory safe, has integrated error handling, and makes adding new protocols straightforward.

The engine handles HTTP/HTTPS with segmented downloads and automatic resume, plus full BitTorrent support including DHT, PEX, and LPD. Built on Tokio for async I/O, it efficiently manages concurrent downloads and streams real-time progress events back to the UI.

## Requirements

You'll need [Rust](https://rustup.rs/) 1.77 or newer.

The GTK4 frontend requires GTK4 4.14+ and libadwaita 1.5+. Install the development packages for your distro:

```bash
# Debian/Ubuntu
sudo apt install libgtk-4-dev libadwaita-1-dev

# Fedora
sudo dnf install gtk4-devel libadwaita-devel

# Arch
sudo pacman -S gtk4 libadwaita
```

## Building

Run `cargo build` for development or `cargo build --release` for production. Use `cargo run` to launch, `cargo test` for tests, `cargo clippy` for linting, and `cargo fmt` for formatting.

```bash
cargo build --release
cargo run
```

## Usage

Click the + button to add a download. Enter a URL, paste a magnet link, or select a torrent file. The download list updates in real time showing speed, progress, and ETA. Pause, resume, or remove downloads individually or use batch operations. Completed downloads appear in your history where you can open files directly.

Keyboard shortcuts: Ctrl+N for new download, Ctrl+Shift+P to pause all, Ctrl+Shift+R to resume all, Ctrl+Q to quit.

## Troubleshooting

If a download stalls, it has no active connections. Check your network and try resuming. Connection failures usually mean the server is unreachable or the URL is wrong. Torrents with no seeds have no peers to download from and may be inactive.

For debug logging:
```bash
RUST_LOG=debug cargo run
```

## Privacy

Gosh Fetch collects nothing. No telemetry, no analytics, no phoning home. The app only makes network requests when you explicitly start a download (and for BitTorrent DHT/tracker operations if enabled). Everything stays on your machine.

## Architecture

The project is a Rust workspace with two crates. The `gosh-fetch-core` crate contains shared logic that's UI-agnostic. The `gosh-fetch-gtk` crate provides the GTK4/libadwaita frontend. Database migrations for SQLite are in the `migrations/` directory. See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full technical breakdown.

## License

AGPL-3.0. See [LICENSE](LICENSE) for the full text. The gosh-dl download engine is MIT licensed.

This software is provided as-is with no warranty. Use at your own risk.

## Roadmap

Future plans include a browser extension for one-click downloads, scheduled downloading for off-peak hours, time-based bandwidth profiles, RSS/podcast feed support, download categories with custom save locations, and import/export for settings and history.

## Contributing

Contributions are welcome. Open an issue first for major changes. See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.
