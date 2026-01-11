# Qt6 Frontend

Gosh-Fetch provides an alternative Qt6/QML frontend for users who prefer the Qt ecosystem or need integration with Qt-based desktop environments.

## Overview

The Qt frontend (`gosh-fetch-qt`) uses:
- **Qt6** for the UI framework
- **QML** for declarative UI
- **cxx-qt** for Rust/Qt interoperability

## System Requirements

### Debian / Ubuntu

```bash
sudo apt install qt6-declarative-dev \
    qml6-module-qtquick \
    qml6-module-qtquick-controls \
    qml6-module-qtquick-layouts \
    qml6-module-qtquick-window \
    qml6-module-qtqml-workerscript \
    cmake
```

### Fedora

```bash
sudo dnf install qt6-qtdeclarative-devel \
    qt6-qtquickcontrols2-devel \
    cmake
```

### Arch Linux

```bash
sudo pacman -S qt6-declarative qt6-quickcontrols2 cmake
```

### openSUSE

```bash
sudo zypper install qt6-declarative-devel \
    qt6-quickcontrols2-devel \
    cmake
```

### NixOS / Nix

The `flake.nix` includes Qt6 dependencies in the dev shell:

```bash
nix develop
```

## Building

```bash
# Build Qt frontend
cargo build -p gosh-fetch-qt --release

# Run Qt frontend
cargo run -p gosh-fetch-qt
```

## Troubleshooting

### QQmlApplicationEngine Failed to Load Component

**Error:**
```
QQmlApplicationEngine failed to load component
qrc:/qt/qml/io/github/gosh/Fetch/qml/main.qml:3:1: module "QtQuick" is not installed
```

**Cause:** Qt6 QML runtime modules are not installed on your system.

**Solution:** Install the required Qt6 packages for your distribution (see System Requirements above).

### Verifying Qt6 Installation

Check if Qt6 is properly installed:

```bash
# Check Qt version
qmake6 --version

# Check QML module path
qmake6 -query QT_INSTALL_QML

# Verify QtQuick module exists
ls $(qmake6 -query QT_INSTALL_QML)/QtQuick
```

### Environment Variables

If QML modules are installed in a non-standard location:

```bash
# Set QML import path
export QML_IMPORT_PATH=/path/to/qt6/qml

# Debug QML loading issues
export QT_DEBUG_PLUGINS=1
export QML_IMPORT_TRACE=1
```

### Build Errors

If the build fails with linker errors:

1. Ensure cmake is installed (required by cxx-qt-build)
2. Verify Qt6 development headers are installed
3. Check that pkg-config can find Qt6:
   ```bash
   pkg-config --libs Qt6Quick Qt6QuickControls2
   ```

## Features

The Qt frontend provides:
- Native Qt6 look and feel
- QML-based responsive UI
- Full feature parity with GTK frontend
- System theme integration

## Limitations

- Requires Qt6 (Qt5 is not supported)
- System tray functionality depends on desktop environment Qt support
