#!/bin/sh
# Install Sabbatia Tempora for the current user (no root needed).
#   ./packaging/install.sh            build and install into ~/.local
#   ./packaging/install.sh uninstall  remove those files
set -eu
PREFIX="${PREFIX:-$HOME/.local}"
cd "$(dirname "$0")/.."

BIN="$PREFIX/bin/sabbatia"
APP="$PREFIX/share/applications/sabbatia.desktop"
ICON="$PREFIX/share/icons/hicolor/96x96/apps/sabbatia.png"

if [ "${1:-}" = "uninstall" ]; then
    rm -f "$BIN" "$APP" "$ICON" "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/sabbatia.desktop"
    echo "Removed. Configuration kept in ${XDG_CONFIG_HOME:-$HOME/.config}/sabbatia/"
    exit 0
fi

cargo build --release --locked
install -Dm755 target/release/sabbatia "$BIN"
install -Dm644 packaging/sabbatia.desktop "$APP"
install -Dm644 assets/sabbatia.png "$ICON"
command -v update-desktop-database >/dev/null && update-desktop-database -q "$PREFIX/share/applications" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$PREFIX/share/icons/hicolor" || true
echo "Installed $BIN"
case ":$PATH:" in *":$PREFIX/bin:"*) ;; *) echo "Note: $PREFIX/bin is not on PATH." ;; esac
