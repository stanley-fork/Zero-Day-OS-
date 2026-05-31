#!/bin/bash -e
set -euo pipefail
# stage3/04-hardware-enable/01-run.sh — Enable Cardputer Zero hardware
# Enables I2C, SPI, camera, IR, and other peripherals
# Note: raspi-config is NOT available in pure Debian — use direct config instead

# ── Enable I2C, SPI, Camera via direct config (not raspi-config) ──
on_chroot << EOF
# Create I2C and SPI device nodes / permissions
mkdir -p /etc/modules-load.d
mkdir -p /etc/udev/rules.d

# Create input and video groups if they don't exist (for USB keyboard/mouse + DRM)
getent group input >/dev/null 2>&1 || groupadd -r input
getent group video >/dev/null 2>&1 || groupadd -r video

# Add root user to input and video groups for compositor access
usermod -aG input root 2>/dev/null || true
usermod -aG video root 2>/dev/null || true

# Enable i2c and spi via systemd-modules-load
echo "i2c_dev" > /etc/modules-load.d/i2c.conf
echo "spi_bcm2835" >> /etc/modules-load.d/i2c.conf 2>/dev/null || true
echo "spidev" >> /etc/modules-load.d/i2c.conf 2>/dev/null || true

# Add the spidev overlay to config.txt if it exists (RPi boot)
if [ -f /boot/config.txt ]; then
    echo "dtparam=spi=on" >> /boot/config.txt 2>/dev/null || true
    echo "dtparam=i2c_arm=on" >> /boot/config.txt 2>/dev/null || true
fi
EOF

# ── Load required kernel modules ──
# Note: BCM2835-specific modules may not exist on all hardware.
# The build will be deployed on BCM2837-based Cardputer Zero,
# but we list alternatives and let the kernel skip unknown modules.
cat > "${ROOTFS_DIR}/etc/modules-load.d/zeroday.conf" << 'MODULES'
# ZERO-DAY OS kernel modules
i2c_dev
i2c_bcm2835
spi_bcm2835
spidev
brcmfmac
brcmutil
hci_uart
btbcm
btintel
btrtl
rfkill
lirc_dev
uinput
# USB host — enables USB-A keyboard/mouse/hub on Cardputer Zero
dwc2
usbhid
usb_storage
evdev
# Note: USB gadget modules (g_ether, g_serial, libcomposite) are NOT loaded
# statically because they conflict with each other and require configfs setup.
# They are loaded dynamically by usb-gadget-mode when the user activates them.
# Note: lirc_rpi does not exist in mainline kernels. Use lirc_dev + ir_gpio_tx/rx
# device tree overlays instead.
MODULES

# ── Configure RTC (RX8130CE) ──
cat > "${ROOTFS_DIR}/etc/modules-load.d/rtc.conf" << 'EOF'
rtc_rx8130
i2c_dev
EOF

# ── Set CPU governor via systemd service (not in chroot) ──
# Writing to /sys inside on_chroot affects the BUILD HOST, not the target.
# Instead, configure this as a first-boot service.
cat > "${ROOTFS_DIR}/etc/systemd/system/cpufreq-ondemand.service" << 'EOF'
[Unit]
Description=Set CPU governor to ondemand
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/bin/sh -c 'echo ondemand > /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || true'

[Install]
WantedBy=multi-user.target
EOF

on_chroot << EOF
systemctl enable cpufreq-ondemand.service 2>/dev/null || true
systemctl enable i2c-dev.service 2>/dev/null || true
EOF

echo "[zeroday] Hardware modules configured."

# ── HDMI hotplug udev rule ──
# When an HDMI monitor is connected/disconnected, DRM sends a change event.
# This rule triggers the zeroday-comp compositor to reconfigure outputs.
cat > "${ROOTFS_DIR}/etc/udev/rules.d/99-hdmi-hotplug.rules" << 'UDEV'
# HDMI hotplug — notify compositor of DRM output changes
ACTION=="change", SUBSYSTEM=="drm", RUN+="/usr/local/bin/hdmi-hotplug-notify"
# DRM device permissions — allow video group access for compositor
SUBSYSTEM=="drm", GROUP="video", MODE="0660"
# Backlight permissions — allow video group to control LCD backlight
SUBSYSTEM=="backlight", GROUP="video", MODE="0660"
UDEV

# ── USB input device udev rules ──
# USB keyboards and mice plugged into the USB-A port get proper permissions
# and keyboard layout. Also handles hotplug notification.
cat > "${ROOTFS_DIR}/etc/udev/rules.d/70-usb-input.rules" << 'UDEV'
# USB keyboard — set keyboard group, readable layout via libinput
KERNEL=="event[0-9]*", SUBSYSTEM=="input", ATTRS{idVendor}=="*", ATTRS{idProduct}=="*", ENV{ID_INPUT_KEYBOARD}=="1", GROUP="input", MODE="0660"
# USB mouse — set input group
KERNEL=="event[0-9]*", SUBSYSTEM=="input", ENV{ID_INPUT_MOUSE}=="1", GROUP="input", MODE="0660"
# USB keyboard hotplug — notify compositor
ACTION=="add", SUBSYSTEM=="input", ENV{ID_INPUT_KEYBOARD}=="1", RUN+="/usr/local/bin/usb-input-notify add"
ACTION=="remove", SUBSYSTEM=="input", ENV{ID_INPUT_KEYBOARD}=="1", RUN+="/usr/local/bin/usb-input-notify remove"
# USB mouse hotplug
ACTION=="add", SUBSYSTEM=="input", ENV{ID_INPUT_MOUSE}=="1", RUN+="/usr/local/bin/usb-input-notify add"
ACTION=="remove", SUBSYSTEM=="input", ENV{ID_INPUT_MOUSE}=="1", RUN+="/usr/local/bin/usb-input-notify remove"
UDEV

# ── HDMI hotplug notification script ──
cat > "${ROOTFS_DIR}/usr/local/bin/hdmi-hotplug-notify" << 'HDMISCRIPT'
#!/bin/sh
# hdmi-hotplug-notify — Called by udev on DRM change events
# When HDMI is connected: enables HDMI-A-1 output for ALL apps
# When HDMI is disconnected: disables HDMI-A-1 output
# Works with zeroday-comp, cage, and sway compositors.
HDMI_STATUS="/sys/class/drm/card0-HDMI-A-1/status"
CONNECTED="disconnected"
if [ -f "$HDMI_STATUS" ]; then
    CONNECTED=$(cat "$HDMI_STATUS" 2>/dev/null || echo "disconnected")
fi

logger -t zeroday "HDMI hotplug: $CONNECTED"

case "$CONNECTED" in
    connected)
        # HDMI monitor connected — enable external output for all apps
        logger -t zeroday "HDMI connected: enabling 1080P external display"

        # Set ZERODAY_DISPLAY for all current and future processes
        export ZERODAY_DISPLAY=hdmi
        export ZERODAY_HDMI=1
        echo "ZERODAY_DISPLAY=hdmi" > /tmp/zeroday-display.env
        echo "ZERODAY_HDMI=1" >> /tmp/zeroday-display.env

        # Switch PulseAudio audio to HDMI sink
        if command -v pactl >/dev/null 2>&1; then
            pactl set-default-sink hdmi 2>/dev/null || true
            pactl set-sink-volume hdmi 80% 2>/dev/null || true
        fi

        # Enable HDMI output in sway (if running)
        if pgrep -x sway >/dev/null 2>&1; then
            swaymsg output HDMI-A-1 enable 2>/dev/null || true
            swaymsg output HDMI-A-1 mode 1920x1080@30Hz 2>/dev/null || true
            swaymsg output HDMI-A-1 pos 0 0 2>/dev/null || true
        fi

        # Signal zeroday-comp (SIGUSR1 = output reconfigure)
        for pid in $(pgrep -f zeroday-comp 2>/dev/null); do
            kill -USR1 "$pid" 2>/dev/null || true
        done

        # Signal cage and use wlr-randr to enable HDMI output
        if command -v wlr-randr >/dev/null 2>&1; then
            WAYLAND_DISPLAY=wayland-0 wlr-randr --output HDMI-A-1 --mode 1920x1080@30Hz --on 2>/dev/null || true
        fi
        for pid in $(pgrep -f cage 2>/dev/null); do
            kill -USR1 "$pid" 2>/dev/null || true
        done
        ;;
    disconnected)
        # HDMI monitor disconnected — disable external output
        logger -t zeroday "HDMI disconnected: reverting to LCD-only"

        # Clear ZERODAY_DISPLAY
        echo "ZERODAY_DISPLAY=lcd" > /tmp/zeroday-display.env
        echo "ZERODAY_HDMI=0" >> /tmp/zeroday-display.env

        # Switch PulseAudio audio back to ES8390/speakers
        if command -v pactl >/dev/null 2>&1; then
            pactl set-default-sink es8390 2>/dev/null || true
        fi

        # Disable HDMI output in sway
        if pgrep -x sway >/dev/null 2>&1; then
            swaymsg output HDMI-A-1 disable 2>/dev/null || true
        fi

        # Signal compositors
        for pid in $(pgrep -f zeroday-comp 2>/dev/null); do
            kill -USR1 "$pid" 2>/dev/null || true
        done
        for pid in $(pgrep -f cage 2>/dev/null); do
            kill -USR1 "$pid" 2>/dev/null || true
        done
        ;;
esac
HDMISCRIPT
chmod +x "${ROOTFS_DIR}/usr/local/bin/hdmi-hotplug-notify"

# ── USB input hotplug notification script ──
cat > "${ROOTFS_DIR}/usr/local/bin/usb-input-notify" << 'USBSCRIPT'
#!/bin/sh
# usb-input-notify — Called by udev on USB keyboard/mouse add/remove
# Logs the event and signals the compositor to rescan input devices
ACTION="$1"
if [ -z "$ACTION" ]; then
    # Called from udev ACTION environment
    ACTION="$ACTION"
fi

PRODUCT=""
if [ -f "/sys$DEVPATH/device/product" ]; then
    PRODUCT=$(cat "/sys$DEVPATH/device/product" 2>/dev/null || echo "unknown")
fi

logger -t zeroday "USB input $ACTION: $PRODUCT ($DEVPATH)"

# Signal compositor to rescan input devices (SIGUSR2)
COMP_PIDS=$(pgrep -f zeroday-comp 2>/dev/null || true)
for pid in $COMP_PIDS; do
    kill -USR2 "$pid" 2>/dev/null || true
done
# Signal cage (fallback compositor)
CAGE_PIDS=$(pgrep -f cage 2>/dev/null || true)
for pid in $CAGE_PIDS; do
    kill -USR2 "$pid" 2>/dev/null || true
done
USBSCRIPT
chmod +x "${ROOTFS_DIR}/usr/local/bin/usb-input-notify"

echo "[zeroday] HDMI hotplug and USB input udev rules installed."

# ── PulseAudio configuration for HDMI audio switching ──
# When HDMI monitor is connected, PulseAudio auto-switches to HDMI audio sink.
# When disconnected, switches back to ES8390/speakers.
mkdir -p "${ROOTFS_DIR}/etc/pulse"

cat > "${ROOTFS_DIR}/etc/pulse/default.pa" << 'PAPULSE'
#!/usr/bin/pulseaudio -nF
# ZERO-DAY OS PulseAudio configuration
# Auto-loads ALSA modules, auto-switches to HDMI when connected

# Load ALSA drivers
.ifexists module-alsa-sink.so
load-module module-alsa-sink device=hw:0,0 sink_name=es8390 sink_properties="device.description='ES8390_Speaker'"
.endif

.ifexists module-alsa-sink.so
# HDMI audio sink — only active when HDMI monitor is connected
load-module module-alsa-sink device=hw:0,1 sink_name=hdmi sink_properties="device.description='HDMI_Audio'" profile=hdmi
.endif

# Load module-switch-on-port-available to auto-switch audio output
.ifexists module-switch-on-port-available.so
load-module module-switch-on-port-available
.endif

# Auto-switch to HDMI when connected, back to speakers when disconnected
load-module module-alsa-sink-control

# Load native protocol
load-module module-native-protocol-unix

# Load D-Bus protocol (for Bluetooth audio)
.ifexists module-bluez5-discover.so
load-module module-bluez5-discover
.endif

# Load console-kit for session tracking
.ifexists module-console-kit.so
load-module module-console-kit
.endif

# Default sink: auto (prefers HDMI when connected, ES8390 otherwise)
set-default-sink auto
PAPULSE

# PulseAudio daemon configuration
cat > "${ROOTFS_DIR}/etc/pulse/daemon.conf" << 'PADAEMON'
# ZERO-DAY OS PulseAudio daemon config
resample-method = speex-fixed-1
default-sample-format = s16le
default-sample-rate = 44100
alternate-sample-rate = 48000
default-fragments = 2
default-fragment-size-msec = 15
high-priority = yes
nice-level = -11
realtime-scheduling = yes
realtime-priority = 5
PADAEMON

echo "[zeroday] PulseAudio configuration installed."