#!/bin/bash
set -e

VERSION="${1:-2.0.0}"
ARCH="${2:-amd64}"
FRONTEND="${3:-gtk}"

case "$FRONTEND" in
  gtk)
    PACKAGE_NAME="gosh-fetch"
    BINARY_NAME="gosh-fetch-gtk"
    DEPENDS="libgtk-4-1 (>= 4.14), libadwaita-1-0 (>= 1.5), libdbus-1-3"
    DESCRIPTION="Built with GTK4 and libadwaita for a native GNOME experience."
    ;;
  cosmic)
    PACKAGE_NAME="gosh-fetch-cosmic"
    BINARY_NAME="gosh-fetch-cosmic"
    DEPENDS="libxkbcommon0, libwayland-client0, libinput10, libudev1, libseat1, libdbus-1-3, libvulkan1"
    DESCRIPTION="Built with libcosmic for a native COSMIC desktop experience."
    ;;
  qt)
    PACKAGE_NAME="gosh-fetch-qt"
    BINARY_NAME="gosh-fetch-qt"
    DEPENDS="libqt6core6, libqt6gui6, libqt6widgets6, libqt6qml6, libqt6quick6, libdbus-1-3"
    DESCRIPTION="Built with Qt6 for a native KDE Plasma experience."
    ;;
  *)
    echo "Unknown frontend: $FRONTEND (use gtk, cosmic, or qt)"
    exit 1
    ;;
esac

# Create directory structure
BUILD_DIR="build/${PACKAGE_NAME}_${VERSION}_${ARCH}"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/share/applications"
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$BUILD_DIR/usr/share/metainfo"
mkdir -p "$BUILD_DIR/usr/share/doc/${PACKAGE_NAME}"

# Copy binary
cp "../../target/release/${BINARY_NAME}" "$BUILD_DIR/usr/bin/"
chmod 755 "$BUILD_DIR/usr/bin/${BINARY_NAME}"

# Copy desktop file and icons
cp "../../gosh-fetch.desktop" "$BUILD_DIR/usr/share/applications/io.github.gosh.Fetch.desktop"
cp "../../resources/io.github.gosh.Fetch.png" "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps/"
cp "../../io.github.gosh.Fetch.metainfo.xml" "$BUILD_DIR/usr/share/metainfo/"

# Copy license
cp "../../LICENSE" "$BUILD_DIR/usr/share/doc/${PACKAGE_NAME}/copyright"

# Calculate installed size
INSTALLED_SIZE=$(du -sk "$BUILD_DIR" | cut -f1)

# Create control file
cat > "$BUILD_DIR/DEBIAN/control" << EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: net
Priority: optional
Architecture: ${ARCH}
Depends: ${DEPENDS}
Installed-Size: ${INSTALLED_SIZE}
Maintainer: Gosh <gosh@example.com>
Homepage: https://github.com/goshitsarch-eng/Gosh-Fetch-linux
Description: Modern download manager for Linux
 Gosh Fetch is a powerful and modern download manager for Linux with support
 for HTTP, HTTPS, and BitTorrent downloads. ${DESCRIPTION}
 .
 Features:
  - HTTP/HTTPS downloads with resume support
  - BitTorrent and magnet link support
  - Download scheduling and queue management
  - System tray integration
EOF

# Build the package
dpkg-deb --build --root-owner-group "$BUILD_DIR"
mv "build/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb" .

echo "Built: ${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
