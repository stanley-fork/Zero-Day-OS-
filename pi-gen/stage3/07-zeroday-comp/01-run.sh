#!/bin/bash -e
set -euo pipefail
# stage3/07-zeroday-comp/01-run.sh
# Install zeroday-comp Wayland compositor
#
# The compositor binary must be pre-built on the host:
#   cd compositor && make cross-build
# Or: cargo build --release --target aarch64-unknown-linux-gnu
#
# If the binary is not found, cage (Wayland kiosk) is used as fallback.

COMP_BIN="${PROJECT_ROOT}/compositor/target/aarch64-unknown-linux-gnu/release/zeroday-comp"

# ── Install wayland client libraries (needed by Pygame/SDL2) ──
on_chroot << EOF
apt-get -o Acquire::Retries=3 install --no-install-recommends -y \
    libwayland-client0 \
    libwayland-cursor0 \
    libwayland-egl1 \
    wlr-randr \
    2>/dev/null || true
EOF

# ── Install compositor binary if pre-built ──
install -m 755 -d "${ROOTFS_DIR}/usr/local/bin"

if [ -f "${COMP_BIN}" ]; then
    echo "[zeroday-comp] Found pre-built binary — installing"
    install -m 755 "${COMP_BIN}" "${ROOTFS_DIR}/usr/local/bin/zeroday-comp"
    
    BINARY_SIZE=$(du -h "${COMP_BIN}" | cut -f1)
    echo "[zeroday-comp] Installed /usr/local/bin/zeroday-comp (${BINARY_SIZE})"
else
    echo "[zeroday-comp] No pre-built binary found at ${COMP_BIN}"
    echo "[zeroday-comp] To build: cd compositor && make cross-build"
    echo "[zeroday-comp] Falling back to cage (Wayland kiosk) compositor"
    echo "[zeroday-comp] Skipping zeroday-comp.service install"
    
    # Still install cage as primary if zeroday-comp is missing
    # The zeroday-gui.service (cage) is already installed by 14-games-entertainment
    exit 0
fi

# ── zeroday-comp configuration ──
install -m 755 -d "${ROOTFS_DIR}/etc/zeroday"

cat > "${ROOTFS_DIR}/etc/zeroday/comp.env" << 'COMPENV'
# /etc/zeroday/comp.env — ZERO-DAY OS custom compositor environment
# zeroday-comp: Wayland compositor for M5Stack Cardputer Zero
# Dual-output: ST7789V LCD (320x170) + HDMI hotplug (1080P@30fps max)
#
# Boot chain:
#   systemd → zeroday-boot.service → zeroday-comp (Wayland)
#   zeroday-comp runs cyber_launcher (Pygame on Wayland via SDL2)
#   if zeroday-comp fails → cage → Xorg + i3 (TUI)
#
# Features over cage:
#   - Fn-key compositor-level bindings (panic, stealth, quick-launch)
#   - Automatic backlight control and power management
#   - HDMI hotplug: auto-detects monitor, configures as Monitor 2 @ 1080P@30fps
#   - Sub-2MB RAM overhead (vs ~3MB cage, ~15MB sway)
#   - DRM/KMS direct rendering for ST7789V
#   - Single-client kiosk: no window management overhead
#   - Clean signal handling (SIGTERM kills children first)

WAYLAND_DISPLAY=wayland-0
SDL_VIDEODRIVER=wayland
PYGAME_HIDE_SUPPORT_PROMPT=1
SDL_RENDER_DRIVER=opengles2
ZERODAY_COMP_DRM=/dev/dri/card0
ZERODAY_COMP_RESOLUTION=320x170
ZERODAY_COMP_FPS=30

# HDMI hotplug (auto-detect via /sys/class/drm/card0-HDMI-A-1/status)
# Set to 1 to force-enable HDMI even without monitor detected
ZERODAY_HDMI=0

# HDMI monitor resolution (max 1080P @ 30fps on RP3A0)
ZERODAY_HDMI_WIDTH=1920
ZERODAY_HDMI_HEIGHT=1080
ZERODAY_HDMI_FPS=30

ZERODAY_LCD_WIDTH=320
ZERODAY_LCD_HEIGHT=170
ZERODAY_COMP_NO_CURSOR=1
COMPENV

# ── Systemd service: zeroday-comp (primary) ──
install -m 755 -d "${ROOTFS_DIR}/etc/systemd/system"

cat > "${ROOTFS_DIR}/etc/systemd/system/zeroday-comp.service" << 'SVCEOF'
[Unit]
Description=ZERO-DAY OS Wayland Compositor (zeroday-comp)
After=zeroday-boot.service
Wants=zeroday-boot.service
Conflicts=zeroday-gui.service zeroday-tui.service

[Service]
Type=simple
EnvironmentFile=/etc/zeroday/comp.env
ExecStartPre=/bin/sh -c 'command -v zeroday-comp >/dev/null 2>&1 || exit 1'
ExecStart=/usr/local/bin/zeroday-comp --client /usr/local/bin/cyber_launcher --no-cursor --hdmi-auto
Restart=on-failure
RestartSec=3
OnFailure=zeroday-gui.service

[Install]
WantedBy=multi-user.target
SVCEOF

# ── Update zeroday-gui.service to be fallback ──
cat > "${ROOTFS_DIR}/etc/systemd/system/zeroday-gui.service" << 'SVCEOF2'
[Unit]
Description=ZERO-DAY OS GUI Launcher (cage Wayland kiosk — fallback)
After=zeroday-boot.service
Wants=zeroday-boot.service
Conflicts=zeroday-comp.service zeroday-tui.service

[Service]
Type=simple
EnvironmentFile=/etc/xdg/cage/config
ExecStartPre=/bin/sh -c 'command -v cage >/dev/null 2>&1 || exit 1'
ExecStart=/usr/bin/cage -- /usr/local/bin/cyber_launcher
Restart=on-failure
RestartSec=3
OnFailure=zeroday-tui.service

[Install]
WantedBy=multi-user.target
SVCEOF2

# ── Enable zeroday-comp as primary, with fallback chain ──
# Priority: zeroday-comp → cage (zeroday-gui) → Xorg+i3 (zeroday-tui)
chroot "${ROOTFS_DIR}" systemctl enable zeroday-comp.service 2>/dev/null || true
chroot "${ROOTFS_DIR}" systemctl disable zeroday-gui.service 2>/dev/null || true

echo "[zeroday-comp] Compositor installed and enabled"
echo "[zeroday-comp] Boot priority: zeroday-comp → cage → Xorg+i3"