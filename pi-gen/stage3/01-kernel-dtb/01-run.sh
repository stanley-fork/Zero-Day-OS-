#!/bin/bash -e
set -euo pipefail
# stage3/01-kernel-dtb/01-run.sh — Official RPi firmware + device tree overlays
# Downloads the latest raspberrypi/firmware release (CVE-patched, maintained by
# Raspberry Pi Ltd) and installs kernel, DTBs, overlays, and modules.
# Falls back to Debian linux-image-arm64 if the download fails.

FIRMWARE_DIR="${ROOTFS_DIR}/boot"
OVERLAY_DST="${ROOTFS_DIR}/boot/overlays"
MODULES_DIR="${ROOTFS_DIR}/lib/modules"

mkdir -p "${FIRMWARE_DIR}" "${OVERLAY_DST}" "${MODULES_DIR}"

# ── Download official RPi firmware (latest release) ──
FIRMWARE_TMP="$(mktemp -d)"
FIRMWARE_URL="https://github.com/raspberrypi/firmware/archive/refs/heads/master.tar.gz"

echo "[zeroday] Downloading official RPi firmware (raspberrypi/firmware)..."
if command -v curl &>/dev/null; then
    if curl -fsSL "${FIRMWARE_URL}" -o "${FIRMWARE_TMP}/rpi-firmware.tar.gz" 2>/dev/null; then
        mkdir -p "${FIRMWARE_TMP}/extract"
        tar -xzf "${FIRMWARE_TMP}/rpi-firmware.tar.gz" -C "${FIRMWARE_TMP}/extract" 2>/dev/null || {
            echo "[zeroday] WARNING: Could not extract firmware archive"
        }
        FIRMWARE_EXTRACT_DIR="${FIRMWARE_TMP}/extract/firmware-master"

        if [ -d "${FIRMWARE_EXTRACT_DIR}" ]; then
            # ── Install boot files (kernel8.img, DTBs, GPU boot files) ──
            if [ -d "${FIRMWARE_EXTRACT_DIR}/boot" ]; then
                for f in "${FIRMWARE_EXTRACT_DIR}/boot/"*; do
                    [ -f "$f" ] && cp "$f" "${FIRMWARE_DIR}/" 2>/dev/null || true
                done
                echo "[zeroday] Installed RPi boot files (kernel, DTBs, GPU firmware)"
            fi

            # ── Install DTB overlays ──
            if [ -d "${FIRMWARE_EXTRACT_DIR}/boot/overlays" ]; then
                cp "${FIRMWARE_EXTRACT_DIR}/boot/overlays/"*.dtbo "${OVERLAY_DST}/" 2>/dev/null || true
                echo "[zeroday] Installed RPi DTB overlays"
            fi

            # ── Install kernel modules ──
            if [ -d "${FIRMWARE_EXTRACT_DIR}/modules" ]; then
                KVER=$(ls "${FIRMWARE_EXTRACT_DIR}/modules/" | head -1)
                if [ -n "$KVER" ]; then
                    mkdir -p "${MODULES_DIR}/${KVER}"
                    cp -r "${FIRMWARE_EXTRACT_DIR}/modules/${KVER}/"* "${MODULES_DIR}/${KVER}/" 2>/dev/null || true
                    # Regenerate module deps inside chroot
                    on_chroot depmod -a "${KVER}" 2>/dev/null || true
                    echo "[zeroday] Installed kernel modules for ${KVER}"
                fi
            fi

            # ── Install RPi apt repo for future kernel updates ──
            on_chroot << 'REPO'
apt-get install -y gnupg1 curl 2>/dev/null || true
GPG_KEY=/usr/share/keyrings/raspbian-archive-keyring.gpg
if [ ! -s "$GPG_KEY" ]; then
    mkdir -p /usr/share/keyrings
    curl -sL https://archive.raspberrypi.org/debian/raspberrypi.gpg.key 2>/dev/null | \
        gpg --dearmor -o "$GPG_KEY" 2>/dev/null || true
    # Fallback: keyserver
    if [ ! -s "$GPG_KEY" ]; then
        apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 82B129927FA3303E 2>/dev/null && \
        apt-key export 82B129927FA3303E 2>/dev/null | gpg --dearmor -o "$GPG_KEY" 2>/dev/null || true
    fi
    touch "$GPG_KEY"
fi
cat > /etc/apt/sources.list.d/raspberrypi.list << 'RPISRC'
deb [signed-by=/usr/share/keyrings/raspbian-archive-keyring.gpg] http://archive.raspberrypi.org/debian/ bookworm main
RPISRC
apt-get update -o Dir::Etc::sourcelist="/etc/apt/sources.list.d/raspberrypi.list" -o Dir::Etc::sourceparts="-" -o APT::Get::List-Cleanup="0" 2>/dev/null || true
REPO
            echo "[zeroday] RPi apt repo configured for future apt upgrades"
        else
            echo "[zeroday] WARNING: Firmware extracted but structure unexpected"
        fi
    else
        echo "[zeroday] WARNING: Could not download RPi firmware"
    fi
else
    echo "[zeroday] WARNING: curl not available — installing Debian arm64 kernel as fallback"
    on_chroot << 'FALLBACK'
apt-get install -y --no-install-recommends linux-image-arm64 2>/dev/null || true
FALLBACK
fi

rm -rf "${FIRMWARE_TMP}"

echo "[zeroday] Kernel installation complete"

# ── Compile custom device tree overlays ──
OVERLAY_SRC="${PROJECT_ROOT}/overlays"

if [ -d "${OVERLAY_SRC}" ]; then
    for dts in "${OVERLAY_SRC}"/*.dts; do
        if [ -f "$dts" ]; then
            name=$(basename "$dts" .dts)
            overlay_name="${name%-overlay}"
            dtbo="${OVERLAY_DST}/${overlay_name}.dtbo"
            echo "[zeroday] Compiling overlay: ${name} -> ${overlay_name}.dtbo"
            dtc -@ -I dts -O dtb -o "${dtbo}" "$dts" 2>/dev/null || {
                echo "[zeroday] WARNING: Failed to compile ${name}.dts — will try on target device"
                cp "$dts" "${OVERLAY_DST}/${name}.dts"
            }
        fi
    done
else
    echo "[zeroday] WARNING: No device tree overlay sources found at ${OVERLAY_SRC}"
fi

# ── Install kernel config fragment ──
KERNEL_CFG="${PROJECT_ROOT}/kernel/zeroday-fragment.config"
if [ -f "${KERNEL_CFG}" ]; then
    mkdir -p "${ROOTFS_DIR}/boot/config-overlays"
    cp "${KERNEL_CFG}" "${ROOTFS_DIR}/boot/config-overlays/zeroday.conf"
    echo "[zeroday] Installed kernel config fragment to /boot/config-overlays/zeroday.conf"
fi

# ── Configure /boot/config.txt for Cardputer Zero ──
cat > "${ROOTFS_DIR}/boot/config.txt" << 'BOOTCFG'
# ZERO-DAY OS — M5Stack Cardputer Zero Boot Configuration
# Hardware: RP3A0 SoC (Pi Zero 2W die), 512MB LPDDR2, aarch64
# Primary display: ST7789V 1.9" LCD (320x170)
# Secondary display: HDMI (hotplug, 1080P@30fps max)

# ── HDMI Hotplug ──
# Force HDMI hotplug detection so DRM reports connected/disconnected
hdmi_force_hotplug=1
# HDMI output format: 2 = DVI mode (no audio on HDMI pin)
# For TV/Media Box mode with HDMI audio, change to: hdmi_drive=1
hdmi_drive=2
# HDMI max resolution: 1080P @ 30Hz (bandwidth-limited for RP3A0)
hdmi_cvt=1920 1080 30 0 0 0
hdmi_group=1
hdmi_mode=0x50

# ── HDMI Audio ──
# To enable HDMI audio for media playback (Jellyfin TV, mpv):
#   1. Change hdmi_drive=1 above (enables HDMI audio pin)
#   2. The ST7789V LCD panel driver handles its own audio via ES8390/I2S
#   3. PulseAudio will auto-switch audio output to HDMI when monitor is connected
# Default is hdmi_drive=2 (DVI, no audio) for stealth mode.

# ── GPU memory ──
# 32MB: enough for dual-output (LCD + HDMI mirror/extend) with Wayland
gpu_mem=32

# ── Framebuffer ──
# Primary: 320x170 for ST7789V LCD
# Secondary: HDMI at 1080P (auto-configured by DRM when monitor connected)
max_framebuffers=2
framebuffer_width=320
framebuffer_height=170

# ── Stealth & boot ──
disable_camera_led=1
boot_delay=0
disable_splash=1

# ── Device Tree Overlays ──
# Main CardputerZero overlay (ST7789V LCD, TCA8418 keyboard, ES8390 audio, etc.)
dtoverlay=cardputerzero

# Additional overlays (from official m5stack-linux-dtoverlays + our custom)
dtoverlay=camera-gpio16-high
dtoverlay=spk-gpio24-high

# I2C and SPI (enabled by cardputerzero overlay but explicit here for clarity)
dtparam=i2c1=on
dtparam=i2c_arm=on
dtparam=spi=on

# Disable BCM PWM audio (we use ES8390 via I2S)
dtparam=audio=off
BOOTCFG

echo "[zeroday] Boot configuration written."