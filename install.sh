#!/usr/bin/env bash
set -e

# Edged - Installation Script
echo "==> Installing Edged (Wayland Screen Ruler)..."

PREFIX="${PREFIX:-/usr/local}"
if [ "$EUID" -ne 0 ]; then
    PREFIX="${HOME}/.local"
    echo "==> Installing for current user to ${PREFIX} (run with sudo to install system-wide to /usr/local)"
fi

BIN_DIR="${PREFIX}/bin"
APP_DIR="${PREFIX}/share/applications"
ICON_DIR="${PREFIX}/share/icons/hicolor"
DOC_DIR="${PREFIX}/share/doc/edged"

mkdir -p "${BIN_DIR}" "${APP_DIR}" "${ICON_DIR}/256x256/apps" "${ICON_DIR}/scalable/apps" "${DOC_DIR}"

if [ -f "edged" ]; then
    install -m 755 edged "${BIN_DIR}/edged"
elif [ -f "target/release/edged" ]; then
    install -m 755 target/release/edged "${BIN_DIR}/edged"
else
    echo "Error: edged binary not found! Please build it first with 'cargo build --release'."
    exit 1
fi

if [ -f "edged.desktop" ]; then
    install -m 644 edged.desktop "${APP_DIR}/edged.desktop"
fi

if [ -f "assets/edged.png" ]; then
    install -m 644 assets/edged.png "${ICON_DIR}/256x256/apps/edged.png"
fi

if [ -f "assets/edged.svg" ]; then
    install -m 644 assets/edged.svg "${ICON_DIR}/scalable/apps/edged.svg"
fi

if [ -f "config.example.toml" ]; then
    install -m 644 config.example.toml "${DOC_DIR}/config.example.toml"
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${APP_DIR}" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q "${ICON_DIR}" 2>/dev/null || true
fi

echo "==> Installation complete! Binary installed to ${BIN_DIR}/edged"
echo "==> Run 'edged --help' or configure shortcuts in your Wayland compositor (Hyprland, Sway, GNOME)."
