# ZERO-DAY OS v4.3.0

**The first penetration testing OS built for a credit-card-sized computer you can hold in one hand.**

Built for M5Stack Cardputer Zero — quad-core ARM64, 512MB RAM, 1.9" LCD, 46-key keyboard, WiFi, BT, IR, camera, battery.

---

## What's New

### Smithay 0.7 Wayland Compositor — Full Protocol Support

`zeroday-comp` now implements all core Smithay 0.7 Wayland protocol traits, upgrading from a stub launcher to a real Wayland compositor with full client protocol support:

- **CompositorHandler** — Surface commit handling, client compositor state (via `CompositorClientState` and `ClientData` trait)
- **SeatHandler** — Keyboard focus (`WlSurface`), pointer cursor tracking, seat state management
- **XdgShellHandler** — Toplevel windows (auto-activated with `xdg_toplevel::State::Activated`), popup surfaces with grab and repositioning (`reposition_request`), toplevel/popup destroy lifecycle
- **XdgDecorationHandler** — Server-side decorations enforced (`Mode::ServerSide`) — no client-side title bars wasting space on the 1.9" LCD
- **ShmHandler** — Shared memory buffer protocol for client pixel data
- **BufferHandler** — Buffer lifecycle management (`buffer_destroyed`)

Delegate macros: `delegate_compositor!`, `delegate_seat!`, `delegate_shm!`, `delegate_xdg_shell!`, `delegate_xdg_decoration!`

Binary: **1.5MB stripped** (panic=abort, LTO, opt-level=z) — smaller than cage (3MB), sway (15MB), or Xorg (30MB).

### Cross-Compilation — Debian Trixie Docker Image

The `cross-rs` Docker image for compositor compilation has been rebuilt on **Debian Trixie** (glibc 2.41) to fix build script compatibility with CachyOS/Arch hosts (glibc 2.43). The older Ubuntu Xenial image (glibc 2.31) caused `memoffset` build script failures.

The Docker image includes arm64 cross-compilation libraries:
- `libwayland-dev:arm64`, `libdrm-dev:arm64`, `libgbm-dev:arm64`
- `libegl-dev:arm64`, `libgles-dev:arm64`
- `libinput-dev:arm64`, `libudev-dev:arm64`, `libevdev-dev:arm64`
- `libxkbcommon-dev:arm64`, `libsystemd-dev:arm64`

### Jellyfin Media Player — Built From Source

Jellyfin Desktop (Qt5) is now available directly on the Cardputer Zero. Watch your Jellyfin server content at 1080P on an external HDMI monitor — no external player needed.

- `jellyfinmediaplayer` — Full Qt5 GUI client (best on HDMI)
- `jellyfin-tv` — TUI menu with auto-detect HDMI/fullscreen
- libmpv built from source with Wayland + Vulkan support
- wayland-protocols 1.36 built from source (HDR color-manager support)
- meson upgraded to 1.11.1 (libplacebo dependency)
- Custom mpv.conf optimized for ARM playback (limited to 50MiB buffer)

Press **Fn+M** from anywhere to launch the Jellyfin TV menu.

### HDMI Dual-Screen — Automatic Content + Controls

When an HDMI monitor is connected, the Cardputer Zero becomes a dual-screen media device:

- **LCD (Screen #1)**: GUI launcher, controls, navigation — always visible
- **HDMI (Screen #2)**: Content display window — Jellyfin video, YouTube, DOOM at 1080P@30fps
- No HDMI monitor? Everything runs on LCD in scaled-down or audio-only mode

The compositor automatically creates/removes the HDMI output on plug/unplug. PulseAudio switches audio to HDMI when connected.

### PulseAudio — Fixed & Optimized

- PulseAudio installed via `01-run.sh` with `--force-confnew` (no more chroot conffile prompts)
- `pulseaudio-module-bluetooth` added for BLE audio support
- `pulseaudio-module-alsa-card` removed (doesn't exist in bookworm)
- Optimized `/etc/pulse/daemon.conf` for 512MB RAM constraints

### BLE Remote API — Flipper Zero-Style Companion

A full BLE GATT server runs on the Cardputer Zero for remote control from an Android/iOS companion app. The device advertises as "Cardputer-Zero" with service UUID `0000fe5e`.

6 characteristics: Command RX/TX (shell), File RX/TX (transfer), Status (dashboard), Screen (capture).

```
zeroday-ble-remote start   # Start BLE Remote API
zeroday-ble-remote status  # Show status
zeroday-ble-remote stop    # Stop BLE Remote API
```

See `scripts/hardware/ble-remote/ANDROID_API.md` for the full companion app protocol.

### C6L Direct BLE — No MonsterC5 Needed

The Cardputer Zero can now connect directly to the Unit C6L via BLE 4.2→5.0, bypassing the MonsterC5 hub entirely. This enables wireless Zigbee/Thread scanning and Meshtastic meshchat without any cables.

```
c6l-ctl ble pair           # Scan and pair C6L via direct BLE
C6L_MODE=ble c6l-ctl <cmd>  # Route commands over BLE
```

### Kernel — Official Raspberry Pi Firmware

Replaced custom kernel with official `raspberrypi/firmware` GitHub release (1.20260521). Includes all Raspberry Pi Foundation security patches and hardware support.

### Build Fixes

- **meson 1.11.1**: Upgraded from pip (bookworm has 1.0.1, libplacebo needs ≥1.3.0)
- **wayland-protocols 1.36**: Built from source (bookworm has 1.31, mpv needs color-manager-v1)
- **`set -euo pipefail` → `set -eu`** in pi-gen on_chroot heredocs (`/bin/sh` = dash, no pipefail)
- **libcec7 → libcec6**: Debian bookworm package name fix
- **jellyfin-mpv-shim**: Installed via pip (not in bookworm apt repos)
- **pulseaudio conffile**: Installed via `01-run.sh` with `--force-confnew` to avoid interactive prompt in chroot

---

## M5MonsterC5 Firmware v4.3.0

The ESP32C5 middle-manager firmware is included as a separate download. Flash it to your MonsterC5 board via USB from the Cardputer Zero or any development machine.

### Download

| File | Size | SHA256 |
|---|---|---|
| `zeroday-monsterc5-firmware-v4.3.0.zip` | ~166 KB | See `sha256sum.txt` inside zip |

Contains:
- `bootloader.bin` (21 KB) — ESP-IDF bootloader
- `partition-table.bin` (3 KB) — 4MB flash partition layout
- `zeroday-monsterc5.bin` (304 KB) — Main firmware with WiFi attacks, GPS passthrough, C6L routing, Meshtastic mesh

### Flash from Cardputer Zero

```bash
monsterctl flash cardputer    # Flash from Cardputer Zero SD card
```

### Flash from development machine

```bash
esptool.py --chip esp32c5 -b 460800 \
    --before default_reset --after hard_reset \
    write_flash \
    --flash_mode dio --flash_size 2MB --flash_freq 80m \
    0x2000 bootloader.bin \
    0x8000 partition-table.bin \
    0x10000 zeroday-monsterc5.bin
```

### Firmware features

- WiFi attack engine (deauth, evil twin, SAE overflow, handshake, karma, sniffer, blackout, wardrive)
- GPS passthrough (AT6558, Grove IN, UART 9600)
- C6L routing (ESP32-C6, Grove OUT, I2C+UART 115200)
- Meshtastic LoRa mesh node
- UART multiplexing with `GPS:`, `C6L:`, `MESH:` prefixes
- Board auto-detection (GPS, C6L LCD)
- Serial protocol at 115200 baud

---

## Flash Instructions (OS Image)

```bash
# Extract the image
unzip 2026-05-31-zeroday-os--full.zip

# Flash to microSD (replace sdX with your device)
sudo dd if=2026-05-31-zeroday-os.img of=/dev/sdX bs=4M status=progress conv=fsync

# Or use BalenaEtcher
```

**Recommended card:** 32GB+ microSD (Class 10 / A1)

## First Boot

1. Insert microSD into Cardputer Zero
2. Connect power via micro-USB
3. Login: **root** / **zeroday** — change immediately: `passwd`
4. Configure WiFi: `cardputer-wifi-setup`
5. Launch GUI: `Fn + Tab` or run `cyber_launcher`

## Boot Chain

```
zeroday-comp (Smithay 0.7 Rust Wayland compositor, ~1.5MB)
    └── OnFailure → cage (Wayland kiosk, ~3MB)
            └── OnFailure → Xorg + i3 + stterm (~30MB)
```

---

## Download

| File | Size | Description |
|---|---|---|
| `2026-05-31-zeroday-os--full.zip` | ~1.2 GB | Full image with all tools, games, Jellyfin |
| `2026-05-31-zeroday-os--lite.zip` | ~1.4 GB | Lite image (minimal tools) |
| `zeroday-monsterc5-firmware-v4.3.0.zip` | ~166 KB | M5MonsterC5 ESP32C5 firmware (bootloader + partition table + app) |

---

## Hardware Support

| Component | Driver | Status |
|---|---|---|
| 1.9" ST7789V LCD (320x170) | Device tree overlay | Working |
| 46-key TCA8418 keyboard | Device tree overlay | Working |
| HDMI-A-1 (1080P@30fps, Screen #2) | DRM/KMS hotplug | Working |
| USB-A keyboard/mouse | usbhid udev rules | Working |
| ES8389 audio codec | I2S device tree | Working |
| IMX219 camera | libcamera | Working |
| BMI270 IMU | I2C sysfs | Working |
| RX8130 RTC | I2C rtc module | Working |
| BQ27220 battery gauge | I2C power supply | Working |
| IR transceiver | lirc + GPIO | Working |
| WiFi (802.11 b/g/n) | brcmfmac SDIO | Working |
| Bluetooth 4.2 + BLE | hciattach UART | Working |
| RTL8821CU dongle (wlan1) | DKMS driver | Working |

---

## Known Issues

- **zeroday-comp**: calloop event loop integration for DRM backend rendering is still pending (currently uses a sleep-loop). The Wayland protocol traits are complete and compile, but the DRM rendering pipeline needs the calloop event loop for real frame delivery. Cage + Xorg+i3 fallbacks work perfectly.
- **HDMI**: Limited to 1080P@30fps on RP3A0 (bandwidth constraint)
- **Jellyfin Desktop**: Best experience on HDMI — functional but tight on the 320x170 LCD
- **jellyfin-mpv-shim**: Cast receiver installed via pip (not apt) — bookworm doesn't package it

---

*Built for the field. Designed for the edge. Fits in your wallet.*