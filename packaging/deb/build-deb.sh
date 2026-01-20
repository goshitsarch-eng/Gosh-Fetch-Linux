#!/bin/bash
set -e

VERSION="${1:-2.1.0}"
ARCH="${2:-amd64}"
FRONTEND="${3:-qt}"

PACKAGE_NAME="gosh-fetch"
BINARY_NAME="gosh-fetch-qt"
DEPENDS="qt6-base (>= 6.2), qt6-declarative (>= 6.2), libdbus-1-3"
DESCRIPTION="Built with Qt 6 / Qt Quick for a modern cross-desktop experience."

# Create directory structure
BUILD_DIR="build/${PACKAGE_NAME}_${VERSION}_${ARCH}"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/share/applications"
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$BUILD_DIR/usr/share/metainfo"
mkdir -p "$BUILD_DIR/usr/share/gosh-fetch/qml"
mkdir -p "$BUILD_DIR/usr/share/doc/${PACKAGE_NAME}"

# Copy binary
cp "../../target/release/${BINARY_NAME}" "$BUILD_DIR/usr/bin/"
chmod 755 "$BUILD_DIR/usr/bin/${BINARY_NAME}"

# Copy desktop file and icons
cp "../../gosh-fetch.desktop" "$BUILD_DIR/usr/share/applications/io.github.gosh.Fetch.desktop"
cp "../../resources/io.github.gosh.Fetch.png" "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps/"
cp "../../io.github.gosh.Fetch.metainfo.xml" "$BUILD_DIR/usr/share/metainfo/"
cp "../../crates/gosh-fetch-qt/qml/Main.qml" "$BUILD_DIR/usr/share/gosh-fetch/qml/"

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
