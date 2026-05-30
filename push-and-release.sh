#!/bin/bash
set -e
echo "=== Pushing to GitHub ==="
git push -u origin main
echo "=== Creating GitHub Release v4.2.2 ==="
gh release create v4.2.2 \
  --title "v4.2.2 — ZERO-DAY OS Full Release" \
  --notes "$(cat <<'EOF'
## ZERO-DAY OS v4.2.2 — M5Stack Cardputer Zero

### What's New

**Retro Gaming**
- RetroArch + 5 emulator cores (NES, SNES, GB/GBC, GBA, Genesis)
- Interactive game picker via `retro-play`
- ROM directories pre-configured at `/opt/cardputer/retro/roms/<system>/`
- RetroArch config optimized for 320x170 LCD

**YouTube**
- Search, play, download YouTube videos via `yt`
- Audio-only mode for battery saving (`yt audio`)
- On-screen playback or HDMI output (`ZERODAY_DISPLAY=hdmi`)
- yt-dlp + mpv (Wayland-native streaming, no full download needed)

**DOOM**
- `chocolate-doom` + FreeDOOM WADs pre-installed
- `doom-play` launcher with auto-WAD detection
- Plays natively at 320x170 (nearly DOOM's original 320x200)

**M5MonsterC5 ESP32C5 WiFi Attack Board**
- 30+ attack modes: deauth, evil twin, SAE overflow, karma, handshake capture, sniffer, blackout, beacon spam, rogue AP, ARP poison
- Two interfaces: `monsterctl` (CLI) and `install-janos` (interactive TUI)
- Dedicated WiFi attack hardware — wlan0 stays online for C2

**Ragnar Reconnaissance**
- Lightweight `ragnar-scan` — autonomous 3-phase recon in <50MB RAM
- `ragnar-ctl` service controller for full Ragnar web dashboard
- `threat-intel` — CVE/CISA KEV lookup
- `device-classify` — network device fingerprinting from nmap XML

**Captive Portal Templates**
- 9 portal types: wifi, corporate, social, email, bank, cloud, hotel, airport, coffee, library, gym, custom
- `captive-portal` command with start/stop/logs/list subcommands

**C2 & Tunnels**
- `quick-c2` — encrypted C2 listener (TLS via socat/OpenSSL)
- `tunnel-mgr` — SSH SOCKS5, port forward, reverse tunnel, ICMP tunnel
- `payload-craft` — generate reverse shell payloads
- `doh-proxy` — DNS-over-HTTPS proxy (evade DNS monitoring)

**Exfiltration**
- `exfil-discord` — 7 subcommands (setup, send, file, loot, cmd, screenshot, alert)
- `exfil-dns` — DNS data exfiltration

**Wardriving**
- `wardrive` — GPS-tagged WiFi scanning with KML export and WiGLE upload

**BLE Spam (Wall of Flippers)**
- `ble-spam samsung_tv` / `swift_pair` / `air_pod` / `all`

**MAC Randomization**
- `mac-rotate` — randomize/restore/check device MAC address

**WebUI Dashboard**
- `webui` — HTTPS dashboard on :8443

**Device Lock**
- `device-lock` — PIN lock, auto-lock timeout

**Boot Animation**
- `zeroday-bootanim` — glitch ASCII boot animation

**Loot Organizer**
- `loot-organize` — sort captured data by type

---

### Full Tool List (100+ commands)

WiFi: wifi-scan, wifi-deauth, wifi-handshake, wifi-pmkid, wifi-evil-twin, wifi-crack, wifi-monitor-toggle, wifi-survey-log, captive-portal, wardrive, mac-rotate
Network: net-discover, net-quickscan, net-vulnscan, iot-scan, ragnar-scan, quick-c2, tunnel-mgr, doh-proxy, threat-intel, device-classify, arpspoof, ntlmrelayx, gobuster, exfil-dns
Bluetooth: bt-scan, bt-deep, bt-attack, ble-gatt, bettercap, ble-spam
Camera: cam-snap, cam-stream, cam-ocr
IR: ir-scan, ir-replay, ir-brute
NFC: nfc-read, nfc-clone, nfc-emulate
Sub-GHz: subghz-scan, subghz-record, subghz-replay
Mesh: mesh-chat, mesh-setup
SDR: sdr-scan, rf-capture
Hardware: gpio-probe, cardputer-battery, dongle-setup, monsterctl, install-janos
Exploit: john, hydra, payload-craft, revshell-stabilize
Gaming: doom-play, retro-play, yt
Media: webradio-danish, music-player
System: panic, power-mode, zeroday-boot, zeroday-bootanim, cardputer-wifi-setup, cardputer-wifi-toggle, usb-gadget-mode, stealth-backlight-toggle, first-boot, opencode-session, opencode-ask, tamper-watch, mac-rotate, loot-organize, device-lock, webui, ragnar-ctl

---

### Bootable Images

| File | Description | Size |
|------|-------------|------|
| `2026-05-30-zeroday-os--full.zip` | Complete system with all tools | ~957MB |
| `2026-05-30-zeroday-os--lite.zip` | Lite system (without cleanup) | ~1.3GB |

### Flash
```bash
unzip 2026-05-30-zeroday-os--full.zip
sudo dd if=2026-05-30-zeroday-os--full.img of=/dev/sdX bs=4M status=progress conv=fsync
```

### First Boot
1. Login: `root` / `zeroday` — change immediately: `passwd`
2. WiFi: `cardputer-wifi-setup`
3. TUI: `Fn + Tab` or `cyber_launcher`

### Known Limitations
- 512MB RAM — no metasploit (OOM risk)
- OpenCode arm64 binary not yet available; stub installed
- pip packages (nfcpy, cc1101, rfcat, meshtastic) deferred to first boot
EOF
)" \
  ./pi-gen/deploy/2026-05-30-zeroday-os--full.zip \
  ./pi-gen/deploy/2026-05-30-zeroday-os--lite.zip
echo "=== Done! ==="