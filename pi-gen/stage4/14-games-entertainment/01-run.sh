#!/bin/bash -e
set -euo pipefail
# Install games, entertainment, and GUI launcher scripts

install -m 755 -d "${ROOTFS_DIR}/usr/local/bin"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/doom/wads"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/retro/roms"/{nes,snes,gb,gbc,gba,sms,genesis,atari2600,pcengine,lynx}
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/retro/saves"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/retro/cores"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/retro/system"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/loot/yt"
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/music"

# ── Copy game/entertainment scripts ──
for script in yt doom-play retro-play jellyfin-tv; do
    if [ -f "${PROJECT_ROOT}/scripts/hardware/${script}" ]; then
        install -m 755 "${PROJECT_ROOT}/scripts/hardware/${script}" "${ROOTFS_DIR}/usr/local/bin/"
        echo "[zeroday] Installed: ${script}"
    else
        echo "[zeroday] WARNING: Missing script: ${script}"
    fi
done

# ── Copy FreeDOOM WADs to doom directory ──
if [ -f "${ROOTFS_DIR}/usr/share/games/doom/freedoom1.wad" ]; then
    cp "${ROOTFS_DIR}/usr/share/games/doom/freedoom1.wad" "${ROOTFS_DIR}/opt/cardputer/doom/wads/"
fi
if [ -f "${ROOTFS_DIR}/usr/share/games/doom/freedoom2.wad" ]; then
    cp "${ROOTFS_DIR}/usr/share/games/doom/freedoom2.wad" "${ROOTFS_DIR}/opt/cardputer/doom/wads/"
fi

# ── Configure RetroArch for small screen ──
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/config/retroarch"

cat > "${ROOTFS_DIR}/opt/cardputer/config/retroarch/retroarch.cfg" << 'RETROEOF'
# ZERO-DAY OS RetroArch Configuration
# Optimized for Cardputer Zero (320x170 LCD, ARM64, 512MB RAM)

video_driver = "gl"
video_fullscreen = true
video_scale = 1.0
video_threaded = true
video_vsync = true
video_hard_sync = false
video_refresh_rate = 60.0
video_gpu_screenshot = false

audio_driver = "alsa"
audio_enable = true
audio_out_rate = 44100
audio_latency = 64
audio_sync = true

input_driver = "udev"
input_autodetect_enable = true
input_menu_toggle = "f1"
input_hold_fast_forward = "l2"
input_load_state = "f4"
input_save_state = "f2"
input_state_slot_increase = "f6"
input_state_slot_decrease = "f5"
input_quit = "escape"

savestate_auto_save = false
savestate_auto_load = false
slowmotion_ratio = 3.0
fastforward_ratio = 4.0

menu_driver = "rgui"
menu_show_start_screen = false
ui_companion_start = false
pause_nonactive = false

savefile_directory = "/opt/cardputer/retro/saves"
savestate_directory = "/opt/cardputer/retro/saves"
system_directory = "/opt/cardputer/retro/system"

log_verbosity = 1
RETROEOF

# ── Configure cage (Wayland kiosk compositor) ──
install -m 755 -d "${ROOTFS_DIR}/etc/xdg/cage"

cat > "${ROOTFS_DIR}/etc/xdg/cage/config" << 'CAGEEOF'
# cage Wayland kiosk compositor environment
WAYLAND_DISPLAY=wayland-0
SDL_VIDEODRIVER=wayland
PYGAME_HIDE_SUPPORT_PROMPT=1
SDL_RENDER_DRIVER=opengles2
CAGEEOF

# ── Wayland session for autologin ──
# The GUI launcher (cyber_launcher) runs fullscreen inside cage
# cage is lighter than sway (~2MB vs ~15MB) and perfect for kiosk/single-app

install -m 755 -d "${ROOTFS_DIR}/etc/systemd/system"

cat > "${ROOTFS_DIR}/etc/systemd/system/zeroday-gui.service" << 'SVCEOF'
[Unit]
Description=ZERO-DAY OS GUI Launcher (Wayland kiosk)
After=zeroday-boot.service
Wants=zeroday-boot.service
Conflicts=zeroday-tui.service

[Service]
Type=simple
EnvironmentFile=/etc/xdg/cage/config
ExecStart=/usr/bin/cage -- /usr/local/bin/cyber_launcher
Restart=on-failure
RestartSec=3
OnFailure=zeroday-tui.service

[Install]
WantedBy=multi-user.target
SVCEOF

cat > "${ROOTFS_DIR}/etc/systemd/system/zeroday-tui.service" << 'SVCEOF'
[Unit]
Description=ZERO-DAY OS TUI Launcher (Xorg+i3 fallback)
After=zeroday-boot.service
Wants=zeroday-boot.service
Conflicts=zeroday-gui.service

[Service]
Type=simple
Environment=SDL_VIDEODRIVER=x11
ExecStart=/bin/sh -c 'startx /usr/bin/i3 -- :0 vt1'
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
SVCEOF

# ── Enable GUI launcher by default, TUI as fallback ──
chroot "${ROOTFS_DIR}" systemctl enable zeroday-gui.service 2>/dev/null || true

# ── DOOM shareware setup hint ──
cat > "${ROOTFS_DIR}/opt/cardputer/doom/README.md" << 'DOOMEOF'
# DOOM on ZERO-DAY OS

## Quick Start
```bash
doom-play play           # Launch with auto-detected WAD
doom-play shareware      # Download shareware WAD
doom-play list           # List available WADs
```

## WAD Files
Place WAD files in: `/opt/cardputer/doom/wads/`

Supported:
- doom1.wad (Shareware - free)
- doom.wad (Registered)
- doom2.wad (DOOM II)
- freedoom1.wad (FreeDOOM Phase 1 - free)
- freedoom2.wad (FreeDOOM Phase 2 - free)

## Display
- LCD (default): Scaled for 320x170 screen
- HDMI: Set `export ZERODAY_DISPLAY=hdmi` for fullscreen 1080p

## Controls
On the 46-key keyboard:
- W/A/S/D or Arrows = Move
- Space = Use/Open doors
- Ctrl or Left Shift = Fire
- Tab = Map
- 1-7 = Select weapon
- Esc = Menu/Quit
DOOMEOF

# ── Retro gaming README ──
cat > "${ROOTFS_DIR}/opt/cardputer/retro/README.md" << 'RETROEOF'
# Retro Gaming on ZERO-DAY OS

## Quick Start
```bash
retro-play              # Interactive game picker
retro-play nes          # Launch NES game
retro-play snes <rom>   # Launch specific ROM
retro-play cores        # Check installed cores
retro-play setup        # Configure RetroArch
```

## ROM Directories
Place ROM files in: `/opt/cardputer/retro/roms/<system>/`

| System | Directory | Extensions |
|--------|-----------|------------|
| NES | roms/nes/ | .nes .fds .unf |
| SNES | roms/snes/ | .smc .sfc .swc .fig |
| Game Boy | roms/gb/ | .gb .dmg |
| GBC | roms/gbc/ | .gbc |
| GBA | roms/gba/ | .gba |
| SMS | roms/sms/ | .sms |
| Genesis | roms/genesis/ | .gen .md .smd .bin |
| Atari 2600 | roms/atari2600/ | .a26 .bin |
| PC Engine | roms/pcengine/ | .pce .tg16 .cue |
| Lynx | roms/lynx/ | .lnx |

## Emulator Cores
- NES: FCEUmm (libretro-fceumm)
- SNES: Snes9x (libretro-snes9x)
- GB/GBC: Gambatte (libretro-gambatte)
- GBA: mGBA (libretro-mgba)
- Genesis/SMS: Genesis Plus GX (libretro-genesisplusgx)

## Controls (46-key keyboard)
- Arrows = D-Pad
- Z/J = Button A
- X/K = Button B
- Space/Enter = Start
- Tab = Select
- F1 = RetroArch menu
RETROEOF

# ── Jellyfin TV Media Box README ──
mkdir -p "${ROOTFS_DIR}/opt/cardputer/loot/media"
cat > "${ROOTFS_DIR}/opt/cardputer/loot/media/README.md" << 'TVEMODEOF'
# TV Media Box Mode — ZERO-DAY OS

## Quick Start
```bash
jellyfin-tv                   # Interactive menu (auto-detects Jellyfin Desktop)
jellyfin-tv connect <url>     # Connect to Jellyfin server
jellyfin-tv cast              # Start cast receiver
jellyfin-tv play <url>        # Play URL directly (YouTube, etc.)
jellyfin-tv local             # Play local media files
jellyfin-tv status            # Check playback status
jellyfin-tv off               # Stop all media
```

## Jellyfin Desktop (GUI Client)
If `jellyfin-media-player` is installed (built in pi-gen stage 16):
- Press `D` from the jellyfin-tv menu to launch the full GUI client
- Or run: `jellyfinmediaplayer`
- Works on HDMI (1080P) and LCD (320x170)
- Uses Qt5 WebEngine for the Jellyfin web interface + mpv for playback

## HDMI Auto-Detect
When an HDMI monitor is connected:
- Video plays at 1080P fullscreen
- Audio outputs via HDMI
- Set `ZERODAY_DISPLAY=hdmi` to force HDMI mode

Without HDMI:
- Audio-only mode (saves battery)
- Music/radio/podcasts via speakers or headphone jack

## Fn+M Shortcut
Press Fn+M to launch the Jellyfin TV menu from the compositor.
If `jellyfinmediaplayer` (GUI desktop client) is installed, pressing
`D` in the menu launches the full Qt5 WebEngine client.

## Supported Formats
- Video: MP4, MKV, AVI, MOV, WebM (via mpv)
- Audio: MP3, FLAC, WAV, OGG, AAC, M4A
- Streaming: YouTube (via yt-dlp), HLS, direct URLs

## Jellyfin Server Setup
1. Install Jellyfin on your NAS/PC: https://jellyfin.org
2. Run: `jellyfin-tv connect http://YOUR-SERVER:8096`
3. Browse and play — or use cast mode from the Jellyfin app

## Direct Play
Any URL that mpv can play works:
```bash
jellyfin-tv play https://example.com/video.mp4
jellyfin-tv play https://youtube.com/watch?v=VIDEO_ID
```
TVEMODEOF

# ── Jellyfin mpv-shim config for HDMI (fallback client) ──
install -m 755 -d "${ROOTFS_DIR}/opt/cardputer/config/jellyfin-mpv-shim"
cat > "${ROOTFS_DIR}/opt/cardputer/config/jellyfin-mpv-shim/mpv-shim.conf" << 'SHIMCONF'
# Jellyfin mpv-shim configuration for ZERO-DAY OS
# Optimized for HDMI 1080P output on Cardputer Zero

[mpv]
# HDMI mode: fullscreen, hardware decode
mpv_flags=--fs --no-border --vo=gpu --gpu-context=wayland --hwdec=auto --volume=80

# Audio output: auto-select (HDMI or speakers)
audio_device=auto

[general]
# Server discovery
discover_mode=1
SHIMCONF

# Install jellyfin-mpv-shim via pip (NOT in Debian bookworm apt repos)
# Provides cast receiver — fallback if jellyfin-media-player (stage 16) build is skipped
on_chroot << EOF
pip3 install --break-system-packages jellyfin-mpv-shim 2>/dev/null || \
pip3 install jellyfin-mpv-shim 2>/dev/null || \
echo "[zeroday] jellyfin-mpv-shim pip install deferred (install manually: pip3 install jellyfin-mpv-shim)"
EOF

echo "[zeroday] Games and entertainment installed."