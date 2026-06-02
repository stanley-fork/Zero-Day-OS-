#!/bin/bash -e
set -euo pipefail
# stage3/10-trail-nav/01-run.sh
# Install zeroday-trail breadcrumb navigation daemon for ZERO-DAY OS
# Also installs gps-ctl and oled-ctl for GPS + SH1107 OLED support

TRAIL_BIN="${PROJECT_ROOT}/trail/target/aarch64-unknown-linux-gnu/release/zeroday-trail"

echo "[zeroday-trail] Installing breadcrumb navigation daemon..."

# Install trail binary
install -m 755 -d "${ROOTFS_DIR}/usr/local/bin"

if [ -f "${TRAIL_BIN}" ]; then
    echo "[zeroday-trail] Found pre-built binary — installing"
    install -m 755 "${TRAIL_BIN}" "${ROOTFS_DIR}/usr/local/bin/zeroday-trail"

    TRAIL_SIZE=$(du -h "${TRAIL_BIN}" | cut -f1)
    echo "[zeroday-trail] Installed /usr/local/bin/zeroday-trail (${TRAIL_SIZE})"
else
    echo "[zeroday-trail] No pre-built binary found at ${TRAIL_BIN}"
    echo "[zeroday-trail] To build: cd trail && make cross-build"
fi

# Install trail-ctl control script
if [ -f "${PROJECT_ROOT}/trail/trail-ctl" ]; then
    install -m 755 "${PROJECT_ROOT}/trail/trail-ctl" "${ROOTFS_DIR}/usr/local/bin/trail-ctl"
    echo "[zeroday-trail] Installed /usr/local/bin/trail-ctl"
fi

# Install GPS controller
if [ -f "${PROJECT_ROOT}/scripts/hardware/gps-ctl" ]; then
    install -m 755 "${PROJECT_ROOT}/scripts/hardware/gps-ctl" "${ROOTFS_DIR}/usr/local/bin/gps-ctl"
    echo "[zeroday-trail] Installed /usr/local/bin/gps-ctl"
fi

# Install external display manager
if [ -f "${PROJECT_ROOT}/scripts/hardware/ext-display" ]; then
    install -m 755 "${PROJECT_ROOT}/scripts/hardware/ext-display" "${ROOTFS_DIR}/usr/local/bin/ext-display"
    echo "[zeroday-trail] Installed /usr/local/bin/ext-display"
fi

# Install OLED controller (SH1107)
if [ -f "${PROJECT_ROOT}/scripts/hardware/oled-ctl" ]; then
    install -m 755 "${PROJECT_ROOT}/scripts/hardware/oled-ctl" "${ROOTFS_DIR}/usr/local/bin/oled-ctl"
    echo "[zeroday-trail] Installed /usr/local/bin/oled-ctl (SH1107)"
fi

# Install RFID2 controller (WS1850S)
if [ -f "${PROJECT_ROOT}/scripts/hardware/rfid2-ctl" ]; then
    install -m 755 "${PROJECT_ROOT}/scripts/hardware/rfid2-ctl" "${ROOTFS_DIR}/usr/local/bin/rfid2-ctl"
    echo "[zeroday-trail] Installed /usr/local/bin/rfid2-ctl (WS1850S)"
fi

# Install Unit C6L controller (ESP32-C6 + 0.96" LCD)
if [ -f "${PROJECT_ROOT}/scripts/hardware/c6l-ctl" ]; then
    install -m 755 "${PROJECT_ROOT}/scripts/hardware/c6l-ctl" "${ROOTFS_DIR}/usr/local/bin/c6l-ctl"
    echo "[zeroday-trail] Installed /usr/local/bin/c6l-ctl (ESP32-C6)"
fi

# Install C6L middleman (Zero-Day → ESP32 → C6L bridge)
if [ -f "${PROJECT_ROOT}/scripts/hardware/c6l-middleman" ]; then
    install -m 755 "${PROJECT_ROOT}/scripts/hardware/c6l-middleman" "${ROOTFS_DIR}/usr/local/bin/c6l-middleman"
    echo "[zeroday-trail] Installed /usr/local/bin/c6l-middleman (ESP32 middleman bridge)"
fi

# Create trail data directories
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/trail/breadcrumbs"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/trail/gps-tracks"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/trail/waypoints"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/trail/exports"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/trail/config"

# Trail daemon configuration
install -m 755 -d "${ROOTFS_DIR}/etc/zeroday/trail"

cat > "${ROOTFS_DIR}/etc/zeroday/trail/config.env" << 'TRAILEOF'
# /etc/zeroday/trail/config.env — ZERO-DAY OS breadcrumb navigation config
# zeroday-trail: WiFi fingerprint navigation for M5Stack Cardputer Zero
# Optimized for 320x170 LCD, 46-key keyboard, no mouse
#
# Modes:
#   trail-ctl start    — begin dropping breadcrumbs (WiFi scans)
#   trail-ctl mark X   — tag current waypoint (exit, stairs, server_room)
#   trail-ctl exit     — activate exit guidance (follow breadcrumbs back)
#   trail-ctl pause    — stop dropping (save battery)
#   trail-ctl status   — show daemon status
#
# GPS Integration (M5Stack GPS Module v1.1 on Grove UART):
#   gps-ctl start      — start GPS daemon
#   gps-ctl location    — show current lat/lon/alt
#   gps-ctl save "X"   — save GPS waypoint
#
# OLED Integration (M5Stack SH1107 on Grove I2C):
#   oled-ctl trail     — show trail direction on OLED
#   oled-ctl overwatch  — show threat level on OLED
#   oled-ctl sysinfo    — show CPU/mem/disk on OLED
#
# Grove port sharing: GPS (UART) and OLED/NFC (I2C) cannot be used simultaneously.
# GPS + HDMI or GPS + SPI TFT can coexist.

TRAIL_IFACE=wlan0
TRAIL_INTERVAL=15
TRAIL_THRESHOLD=30
TRAIL_MAX_BREADCRUMBS=2048
TRAIL_DECAY_HOURS=8
TRAIL_DATA_DIR=/opt/cardputer/trail/breadcrumbs
TRAIL_OVERWATCH=true
TRAIL_EVIL_TWIN=true
TRAIL_NEW_AP_WATCH=true
TRAIL_QUIET=false
TRAILEOF

echo "[zeroday-trail] Trail navigation installed and configured"