#!/bin/bash -e
set -euo pipefail
# stage3/09-file-manager/01-run.sh
# Install zeroday-fm custom file explorer for ZERO-DAY OS
# TUI file manager optimized for 320x170 LCD, 46-key keyboard, no mouse

FM_BIN="${PROJECT_ROOT}/explorer/target/aarch64-unknown-linux-gnu/release/zeroday-fm"

echo "[zeroday-fm] Installing custom file explorer..."

# Install file manager binary if pre-built
install -m 755 -d "${ROOTFS_DIR}/usr/local/bin"

if [ -f "${FM_BIN}" ]; then
    echo "[zeroday-fm] Found pre-built binary — installing"
    install -m 755 "${FM_BIN}" "${ROOTFS_DIR}/usr/local/bin/zeroday-fm"

    BINARY_SIZE=$(du -h "${FM_BIN}" | cut -f1)
    echo "[zeroday-fm] Installed /usr/local/bin/zeroday-fm (${BINARY_SIZE})"

    # Create compatibility symlink: fm -> zeroday-fm
    chroot "${ROOTFS_DIR}" ln -sf zeroday-fm /usr/local/bin/fm 2>/dev/null || true

    # Configuration for zeroday-fm
    install -m 755 -d "${ROOTFS_DIR}/etc/zeroday"

    cat > "${ROOTFS_DIR}/etc/zeroday/fm.env" << 'FMEOF'
# /etc/zeroday/fm.env — ZERO-DAY OS file explorer configuration
# zeroday-fm: TUI file explorer for M5Stack Cardputer Zero
# Optimized for 320x170 LCD, 46-key keyboard, no mouse
#
# Features:
#   - Arrow key / j/k navigation
#   - Hex viewer (Alt+H)
#   - Metadata display (Alt+M)
#   - Bookmarks (Alt+B)
#   - Search (Ctrl+F or /)
#   - Copy/Cut/Paste (Ctrl+Y/X/V)
#   - Delete (Ctrl+D), Rename (Ctrl+R), Mkdir (Ctrl+N)
#   - Zip/Unzip (Ctrl+Z/Ctrl+E)
#   - Mark files (Space), Mark all (Ctrl+A)
#   - Hidden files toggle (.)
#   - Sort cycle (Ctrl+S: Type/Name/Size/Date)
#   - Back/Forward navigation (Backspace/Ctrl+O/Ctrl+I)

ZERODAY_FM_SHOW_HIDDEN=0
ZERODAY_FM_SORT=type
ZERODAY_FM_START_DIR=/root
FMEOF

    echo "[zeroday-fm] File explorer installed and configured"
else
    echo "[zeroday-fm] No pre-built binary found at ${FM_BIN}"
    echo "[zeroday-fm] To build: cd explorer && make cross-build"
    echo "[zeroday-fm] midnight commander will be used as fallback"
fi