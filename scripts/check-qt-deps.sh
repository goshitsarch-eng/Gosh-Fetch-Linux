#!/bin/bash
# Check Qt6 dependencies for gosh-fetch-qt
# Run this script to verify your system has the required Qt6 packages

set -e

echo "Checking Qt6 dependencies for Gosh-Fetch Qt frontend..."
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_command() {
    if command -v "$1" &> /dev/null; then
        return 0
    fi
    return 1
}

# Detect distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/debian_version ]; then
        echo "debian"
    elif [ -f /etc/fedora-release ]; then
        echo "fedora"
    elif [ -f /etc/arch-release ]; then
        echo "arch"
    else
        echo "unknown"
    fi
}

DISTRO=$(detect_distro)
echo "Detected distribution: $DISTRO"
echo

# Check for Qt6
echo "Checking Qt6 installation..."

if check_command qmake6; then
    QMAKE="qmake6"
elif check_command qmake; then
    QMAKE="qmake"
    QT_VERSION=$($QMAKE --version 2>/dev/null | grep -o "Qt version [0-9]*" | grep -o "[0-9]*")
    if [ "$QT_VERSION" != "6" ]; then
        echo -e "${YELLOW}Warning: qmake found but may be Qt5, not Qt6${NC}"
    fi
else
    echo -e "${RED}ERROR: Qt6 not found. Please install Qt6 development packages.${NC}"
    echo
    case "$DISTRO" in
        debian|ubuntu)
            echo "Install with: sudo apt install qt6-declarative-dev qml6-module-qtquick qml6-module-qtquick-controls qml6-module-qtquick-layouts"
            ;;
        fedora)
            echo "Install with: sudo dnf install qt6-qtdeclarative-devel qt6-qtquickcontrols2-devel"
            ;;
        arch|manjaro)
            echo "Install with: sudo pacman -S qt6-declarative qt6-quickcontrols2"
            ;;
        opensuse*)
            echo "Install with: sudo zypper install qt6-declarative-devel qt6-quickcontrols2-devel"
            ;;
        *)
            echo "Please install Qt6 development packages for your distribution."
            ;;
    esac
    exit 1
fi

QT_VERSION=$($QMAKE --version 2>/dev/null | grep -o "Qt version [0-9.]*" || echo "unknown")
echo -e "${GREEN}Found: $QT_VERSION${NC}"

# Get QML install path
QML_PATH=$($QMAKE -query QT_INSTALL_QML 2>/dev/null || echo "")

if [ -z "$QML_PATH" ]; then
    echo -e "${YELLOW}Warning: Could not determine QML install path${NC}"
else
    echo "QML modules path: $QML_PATH"
fi

echo

# Check QML modules
echo "Checking required QML modules..."

MISSING=()
MODULES=("QtQuick" "QtQuick/Controls" "QtQuick/Layouts" "QtQuick/Window")

for module in "${MODULES[@]}"; do
    if [ -n "$QML_PATH" ] && [ -d "$QML_PATH/$module" ]; then
        echo -e "  ${GREEN}[OK]${NC} $module"
    else
        echo -e "  ${RED}[MISSING]${NC} $module"
        MISSING+=("$module")
    fi
done

echo

# Check cmake
echo "Checking build dependencies..."
if check_command cmake; then
    CMAKE_VERSION=$(cmake --version | head -1)
    echo -e "  ${GREEN}[OK]${NC} cmake ($CMAKE_VERSION)"
else
    echo -e "  ${RED}[MISSING]${NC} cmake"
    MISSING+=("cmake")
fi

echo

# Summary
if [ ${#MISSING[@]} -gt 0 ]; then
    echo -e "${RED}Missing dependencies: ${MISSING[*]}${NC}"
    echo
    echo "Install missing packages:"
    case "$DISTRO" in
        debian|ubuntu)
            echo "  sudo apt install qt6-declarative-dev qml6-module-qtquick qml6-module-qtquick-controls qml6-module-qtquick-layouts qml6-module-qtquick-window cmake"
            ;;
        fedora)
            echo "  sudo dnf install qt6-qtdeclarative-devel qt6-qtquickcontrols2-devel cmake"
            ;;
        arch|manjaro)
            echo "  sudo pacman -S qt6-declarative qt6-quickcontrols2 cmake"
            ;;
        opensuse*)
            echo "  sudo zypper install qt6-declarative-devel qt6-quickcontrols2-devel cmake"
            ;;
        *)
            echo "  Please install Qt6 QML modules and cmake for your distribution."
            ;;
    esac
    exit 1
else
    echo -e "${GREEN}All Qt6 dependencies are satisfied!${NC}"
    echo
    echo "You can now build the Qt frontend:"
    echo "  cargo build -p gosh-fetch-qt --release"
fi
