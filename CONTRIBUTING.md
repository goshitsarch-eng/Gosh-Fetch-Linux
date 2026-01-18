# Contributing to Gosh-Fetch

Thank you for your interest in contributing to Gosh-Fetch! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) 1.77+
- GTK4 4.14+ and libadwaita 1.5+ development libraries

### Getting Started

1. Fork and clone the repository:
```bash
git clone https://github.com/YOUR_USERNAME/Gosh-Fetch-linux.git
cd Gosh-Fetch-linux
```

2. Install dependencies (Debian/Ubuntu example):
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

3. Build the project:
```bash
cargo build
```

4. Run the application:
```bash
cargo run
```

### Build Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Build all crates (development) |
| `cargo build --release` | Build all crates (production) |
| `cargo run` | Run the GTK frontend |
| `cargo run -p gosh-fetch-gtk` | Run GTK frontend explicitly |
| `cargo test` | Run all tests |
| `cargo clippy` | Run Clippy linter |
| `cargo fmt` | Format code |

## Project Structure

```
Gosh-Fetch-linux/
├── crates/
│   ├── gosh-fetch-core/     # Shared core library (UI-agnostic)
│   │   ├── src/
│   │   │   ├── db/          # Database operations
│   │   │   ├── types.rs     # Data types
│   │   │   ├── service.rs   # DownloadService
│   │   │   ├── engine_adapter.rs  # gosh-dl integration
│   │   │   └── utils.rs     # Utilities
│   │   └── Cargo.toml
│   │
│   └── gosh-fetch-gtk/      # GTK4/libadwaita frontend
│       ├── src/
│       │   ├── window/      # Main window
│       │   ├── views/       # Page views
│       │   ├── widgets/     # Reusable widgets
│       │   ├── dialogs/     # Modal dialogs
│       │   └── tray/        # System tray
│       ├── resources/       # GResource files
│       └── Cargo.toml
│
├── migrations/              # SQLite database schema
└── Cargo.toml              # Workspace configuration
```

## Code Style

### Rust
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add documentation comments for public APIs
- Use GTK4/libadwaita idioms for GTK frontend code
- Use async/await with tokio for async operations

### Commit Message Guidelines

Use conventional commit prefixes:
- `feat:` New features
- `fix:` Bug fixes
- `refactor:` Code restructuring
- `docs:` Documentation changes
- `chore:` Maintenance tasks
- `test:` Adding or updating tests

Examples:
```
feat: Add download scheduling option
fix: Handle network timeout during magnet resolution
refactor: Extract download row widget to separate module
docs: Update installation instructions for Fedora
```

## Pull Request Process

1. Create a new branch for your feature or fix:
```bash
git checkout -b feature/your-feature-name
```

2. Make your changes and test thoroughly

3. Ensure code passes checks:
```bash
cargo fmt
cargo clippy
cargo test
```

4. Commit with a descriptive message:
```bash
git commit -m "feat: brief description of changes"
```

5. Push and create a pull request

## Testing

Run the full test suite:
```bash
cargo test
```

Run tests for a specific crate:
```bash
cargo test -p gosh-fetch-core
```

## Adding a New Feature

When adding a new feature:

1. If the feature involves core business logic, add it to `gosh-fetch-core`
2. If it's UI-specific, add it to `gosh-fetch-gtk`
3. Update documentation as needed

## Reporting Issues

When reporting issues, please include:
- Operating system and version
- Desktop environment (GNOME, Cinnamon, etc.)
- Steps to reproduce the issue
- Expected vs actual behavior
- Error messages or logs if applicable

To get logs, run with:
```bash
RUST_LOG=debug cargo run
```

## License

By contributing to Gosh-Fetch, you agree that your contributions will be licensed under the AGPL-3.0 license.
