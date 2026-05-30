#!/bin/bash -e
set -euo pipefail
# stage3/06-terminal-term/01-run.sh
# Install zeroday-term custom terminal emulator for ZERO-DAY OS
# Optimized for 320x170 LCD, 46-key keyboard, no mouse

TERM_BIN="${PROJECT_ROOT}/terminal/target/aarch64-unknown-linux-gnu/release/zeroday-term"

echo "[zeroday-term] Installing custom terminal emulator..."

# Install terminal emulator binary if pre-built
install -m 755 -d "${ROOTFS_DIR}/usr/local/bin"

if [ -f "${TERM_BIN}" ]; then
    echo "[zeroday-term] Found pre-built binary — installing"
    install -m 755 "${TERM_BIN}" "${ROOTFS_DIR}/usr/local/bin/zeroday-term"

    BINARY_SIZE=$(du -h "${TERM_BIN}" | cut -f1)
    echo "[zeroday-term] Installed /usr/local/bin/zeroday-term (${BINARY_SIZE})"

    # ── Set zeroday-term as default terminal ──
    # Create symlink: st -> zeroday-term (for compatibility with cyber_launcher)
    chroot "${ROOTFS_DIR}" ln -sf zeroday-term /usr/local/bin/st 2>/dev/null || true

    # ── Configuration for zeroday-term ──
    install -m 755 -d "${ROOTFS_DIR}/etc/zeroday"

    cat > "${ROOTFS_DIR}/etc/zeroday/term.env" << 'TERMEOF'
# /etc/zeroday/term.env — ZERO-DAY OS terminal configuration
# zeroday-term: Custom terminal emulator for M5Stack Cardputer Zero
# Optimized for 320x170 LCD, 46-key keyboard, no mouse
#
# Features:
#   - Fn+Enter: Open new terminal window
#   - Fn+Esc (Alt+Esc): Close terminal
#   - Ctrl+Shift+Up/Down: Scroll
#   - Ctrl+Shift+C/V: Copy/Paste
#   - Fn+PgUp/PgDn: Font size adjust
#   - Status bar: Battery%, WiFi IP, CPU temp, Load, Time

ZERODAY_TERM_FONT_SIZE=8
ZERODAY_TERM_COLS=40
ZERODAY_TERM_ROWS=19
ZERODAY_TERM_WIDTH=320
ZERODAY_TERM_HEIGHT=170
ZERODAY_TERM_SHELL=/bin/bash
ZERODAY_TERM_STATUS_BAR=1
ZERODAY_TERM_COLORS=256
TERMEOF

    echo "[zeroday-term] Terminal installed and configured"
else
    echo "[zeroday-term] No pre-built binary found at ${TERM_BIN}"
    echo "[zeroday-term] To build: cd terminal && make cross-build"
    echo "[zeroday-term] stterm will be used as fallback terminal"
fi