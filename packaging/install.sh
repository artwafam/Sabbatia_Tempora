#!/bin/sh
# Install or update Sabbatia Tempora for the current user (no root needed).
#   ./packaging/install.sh            build and install into ~/.local
#   ./packaging/install.sh uninstall  remove those files
#
# Updating while the companion is on the desktop is fine: the running copy is
# stopped, replaced, and started again so the new version shows at once.
set -eu
PREFIX="${PREFIX:-$HOME/.local}"
cd "$(dirname "$0")/.."

BIN="$PREFIX/bin/sabbatia"
APP="$PREFIX/share/applications/sabbatia.desktop"
ICON="$PREFIX/share/icons/hicolor/96x96/apps/sabbatia.png"

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }

# Process ids running the installed copy ($BIN) — never anything else.
installed_pids() {
    for pid in $(pgrep -x sabbatia 2>/dev/null); do
        exe=$(readlink "/proc/$pid/exe" 2>/dev/null || true)
        # A replaced binary shows as "<path> (deleted)".
        case "$exe" in "$BIN" | "$BIN (deleted)") echo "$pid" ;; esac
    done
}

stop_running() {
    pids=$(installed_pids)
    [ -n "$pids" ] || return 1
    say "Stopping the running Sabbatia…"
    kill $pids 2>/dev/null || true
    # Give it a moment to save its position and quit.
    i=0
    while [ -n "$(installed_pids)" ] && [ $i -lt 30 ]; do sleep 0.1; i=$((i + 1)); done
    pids=$(installed_pids)
    [ -z "$pids" ] || kill -9 $pids 2>/dev/null || true
    return 0
}

if [ "${1:-}" = "uninstall" ]; then
    stop_running || true
    rm -f "$BIN" "$APP" "$ICON" "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/sabbatia.desktop"
    echo "Removed. Configuration kept in ${XDG_CONFIG_HOME:-$HOME/.config}/sabbatia/"
    exit 0
fi

say "Building (optimised release)…"
echo "    The last step links the whole program in one go and prints nothing"
echo "    for a minute or so. If cargo says \"Blocking waiting for file lock\","
echo "    another cargo (often VS Code's Rust extension) is busy: wait, or close it."
cargo build --release --locked

say "Installing into $PREFIX…"
was_running=false
if stop_running; then was_running=true; fi
mkdir -p "$(dirname "$BIN")"
# Copy beside the target, then rename: the swap is atomic, so a half-written
# binary can never be left behind.
install -m755 target/release/sabbatia "$BIN.new"
mv -f "$BIN.new" "$BIN"
install -Dm644 packaging/sabbatia.desktop "$APP"
install -Dm644 assets/sabbatia.png "$ICON"
command -v update-desktop-database >/dev/null && update-desktop-database -q "$PREFIX/share/applications" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$PREFIX/share/icons/hicolor" || true

if $was_running; then
    say "Starting the new version…"
    # Detached from this terminal, so closing it doesn't close the companion.
    if command -v setsid >/dev/null; then
        setsid -f "$BIN" >/dev/null 2>&1 </dev/null
    else
        nohup "$BIN" >/dev/null 2>&1 </dev/null &
    fi
fi

say "Installed $("$BIN" --version) at $BIN"
case ":$PATH:" in *":$PREFIX/bin:"*) ;; *) echo "Note: $PREFIX/bin is not on PATH." ;; esac
