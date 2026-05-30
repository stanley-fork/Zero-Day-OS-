# ZERO-DAY OS

<p align="center">
  <img src="assets/logo.png" alt="ZERO-DAY OS Logo" width="480">
</p>

**The first penetration testing OS built for a credit-card-sized computer you can hold in one hand.**

ZERO-DAY OS v4.2.2 turns the M5Stack Cardputer Zero — a quad-core ARM64 box with WiFi, BT, IR, a camera, a battery, and a built-in keyboard — into a pocketable offensive security weapon. Every byte of this distro is optimized for the constraints of 512MB RAM and a 1.9" screen. No desktop. No bloat. No compromises. Powered by a custom Rust Wayland compositor and terminal emulator, both built for the hardware.

[![Release v4.2.2](https://img.shields.io/github/v/release/jayis1/Zero-Day-OS-?label=latest%20release)](https://github.com/jayis1/Zero-Day-OS-/releases/latest)

---

## What Makes This Different

You can install Kali on a Raspberry Pi. That's not what this is.

| Stock Pi + Kali | ZERO-DAY OS |
|---|---|
| Boots into a desktop you can't use on 1.9" | Boots into `zeroday-comp` — a Rust Wayland compositor built for 320x170 |
| 2GB+ RAM just for the DE | ~60MB idle, 512MB total — 450MB for tools (`zeroday-comp` uses ~2MB) |
| Mouse required | 46-key Omni-Key system — zero mouse needed |
| Tools are menu items you click | Tools are **2 keystrokes away** from anywhere |
| CLI needed for file management | Native D-Pad File Explorer built into TUI |
| No hardware awareness | IR, camera, IMU, battery — all weaponized |
| Close lid, pray | Press `Fn + P` — everything dies and sanitizes instantly |
| You carry a laptop bag | You carry a credit card |

---

## Hardware — M5Stack Cardputer Zero

| Spec | Value |
|---|---|
| **SoC** | RP3A0 (Pi Zero 2W die), Quad-Core Cortex-A53 |
| **Architecture** | aarch64 / arm64 |
| **RAM** | 512MB LPDDR2 |
| **Display** | 1.9" ST7789V 320x170 RGB565 LCD |
| **Keyboard** | TCA8418 46-key matrix (I2C) |
| **Audio** | ES8390 codec + TPA6130A2 headphone amp (I2S) |
| **IMU** | BMI270 6-axis accelerometer + gyroscope (I2C) |
| **RTC** | RX8130 (I2C) |
| **IO Expander** | PY32IO16 — 16 GPIO + PWM (I2C) |
| **Battery** | BQ27220 fuel gauge (I2C) |
| **WiFi** | 802.11 b/g/n (SDIO) |
| **BT/BLE** | Bluetooth 4.2 + BLE (UART) |
| **IR** | Transceiver (GPIO) |
| **Camera** | IMX219 8MP (CSI) |
| **USB** | USB-C device + USB-A host |
| **Expansion** | Grove (I2C/UART) + 14-pin GPIO header |

Device tree: [`cardputerzero-overlay.dts`](overlays/cardputerzero-overlay.dts) — single comprehensive overlay.

---

## The Constraints We Solved

| Constraint | Our Solution |
|---|---|
| **512MB RAM** | `musl` where possible, `dropbear` over `sshd`, no `postgres`, no heavy daemons. Metasploit excluded. `zeroday-comp`: ~2MB vs cage ~3MB vs sway ~15MB vs Xorg ~30MB |
| **1.9" 320x170 display** | `zeroday-comp` — custom Rust Wayland compositor with Fn-key bindings, DRM/KMS direct rendering |
| **46-key matrix keyboard** | `Fn` Omni-Key system. Every tool is 2 keypresses from anywhere. Compositor-level key handling. |
| **1500mAh battery** | Three power profiles (performance / balanced / stealth). `autosleep`. Radio toggle hotkeys. |
| **No mouse, ever** | `i3` tiling WM backend. tmux splits. Arrow-key everything. |
| **Credit-card size (85x54mm)** | No external dongles needed. IR, BT, WiFi, camera — all on-board. |

---

## Architecture

```
 ┌──────────────────────────────────────────────────────────┐
 │                    ZERO-DAY OS STACK                      │
 ├──────────────────────────────────────────────────────────┤
 │                                                          │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │       GUI LAUNCHER  ·  cyber_launcher            │    │
 │   │     (Wayland via zeroday-comp, primary display)  │    │
 │   │                                                   │    │
 │   │   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐     │    │
 │   │   │WIFI │ │M5MON│ │ NET │ │  BT │ │  IR │     │    │
 │   │   └─────┘ └─────┘ └─────┘ └─────┘ └─────┘     │    │
 │   │   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐     │    │
 │   │   │ CAM │ │PAYLD│ │RADIO│ │NFC  │ │SHELL│     │    │
 │   │   └─────┘ └─────┘ └─────┘ └─────┘ └─────┘     │    │
 │   │   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐     │    │
 │   │   │MEDIA│ │ YT  │ │GAMES│ │RETRO│ │ SYS │     │    │
 │   │   └─────┘ └─────┘ └─────┘ └─────┘ └─────┘     │    │
 │   │   ┌─────┐ ┌────────┐ ┌─────┐                     │    │
 │   │   │OPEN │ │OPENCODE│ │FILE │                     │    │
 │   │   └─────┘ └────────┘ └─────┘                     │    │
 │   │                                                   │    │
 │   │   Fallback 1: cage (Wayland kiosk, ~3MB)         │    │
 │   │   Fallback 2: Xorg + i3 + st (TUI, ~30MB)        │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                          │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   zeroday-comp (Rust Wayland compositor, ~2MB)   │    │
 │   │   · DRM/KMS direct rendering → ST7789V LCD      │    │
 │   │   · Fn-key compositor bindings (panic/stealth)   │    │
 │   │   · Single-client kiosk for cyber_launcher       │    │
 │   │   · Automatic backlight & power management       │    │
 │   │   · Clean signal handling (SIGTERM → children)   │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                          │
 │   ┌───────────┐  ┌───────────┐  ┌───────────────────┐   │
 │   │  i3 wm    │  │  OpenCode │  │  Panic System      │   │
 │   │ (tiling)  │  │  (editor) │  │  (kill + wipe)     │   │
 │   └───────────┘  └───────────┘  └───────────────────┘   │
 │                                                          │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   One-Key Hacking Scripts  /usr/local/bin        │    │
 │   │   wifi-* · net-* · bt-* · ir-* · cam-*          │    │
 │   │   nfc-* · subghz-* · mesh-* · dongle-* · sdr-*  │    │
 │   │   arpspoof · ntlmrelayx · exfil-dns · gobuster   │    │
 │   │   john · hydra · bettercap · ragnar-ctl · panic   │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                          │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   Debian Bookworm arm64  +  Kali Rolling repos  │    │
 │   │   aircrack · nmap · bettercap · sqlmap · john    │    │
 │   │   hydra · gobuster · dsniff · responder · curl   │    │
 │   │   hashcat-utils · hcxdumptool · meshtastic       │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                          │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   RP3A0 Device Tree Overlays                      │    │
 │   │   SPI (LCD) · I2C (kbd,IMU,battery,RTC,IO)      │    │
 │   │   I2S (audio) · CSI (camera) · GPIO (IR,USB)    │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                          │
 └──────────────────────────────────────────────────────────┘
```

---

## Tool Arsenal

Every tool chosen for **sub-100MB RAM at idle**. No fat daemons. No database servers. Metasploit is excluded (requires 1GB+ RAM).

### WiFi Offense
| Command | Description |
|---|---|
| `sudo wifi-scan <iface>` | Quick survey — list all APs, channels, encryption |
| `sudo wifi-deauth <iface> <bssid> <chan>` | Monitor mode + deauth attack |
| `sudo wifi-handshake <iface> <bssid> <chan>` | Capture WPA handshakes → `/opt/cardputer/handshakes/` |
| `sudo wifi-pmkid <iface> <bssid> <chan>` | PMKID capture via hcxdumptool |
| `sudo wifi-evil-twin <ap> <inet> <essid>` | Rogue AP: hostapd + dnsmasq + captive portal |
| `sudo wifi-crack <cap>` | Crack captured handshakes (aircrack/hashcat) |
| `sudo wifi-monitor-toggle` | Toggle managed/monitor mode |
| `sudo wifi-survey-log <iface> [seconds]` | Continuous WiFi survey logger |

### Network Recon & Attack
| Command | Description |
|---|---|
| `sudo net-discover <iface> [subnet]` | ARP scan + ping sweep |
| `net-quickscan <target> [profile]` | Nmap: quick/web/full/stealth/vuln |
| `sudo net-vulnscan <target>` | Nmap vuln → nikto → whatweb |
| `net-pivot <mode> [args]` | SOCKS5 proxy / chisel tunnel / DNS tunnel |
| `device-classify <nmap_xml>` | Parse nmap XML, classify by OUI and service |
| `threat-intel <ip|cve>` | CVE/CISA KEV lookup via NVD API |
| `sudo iot-scan <target> [profile]` | IoT-focused Nmap scan presets |
| `ragnar-scan <iface> <profile>` | Autonomous 3-phase network recon |
| `sudo doh-proxy start <provider> [port]` | DNS-over-HTTPS proxy (evade DNS monitoring) |

### C2 & Tunnels
| Command | Description |
|---|---|
| `quick-c2 listen [port]` | Encrypted C2 listener (socat + OpenSSL) |
| `quick-c2 payload <type> <ip> <port>` | Generate shell one-liners |
| `tunnel-mgr socks <host> <port> <user>` | SSH SOCKS5 proxy (auto-reconnect) |
| `tunnel-mgr forward <lport> <rhost:rport> <ssh>` | SSH local port forward |
| `tunnel-mgr reverse <rport> <lport> <ssh>` | SSH reverse port forward |
| `tunnel-mgr icmp <host>` | ICMP tunnel (stealthy C2) |
| `tunnel-mgr list` | List active tunnels |
| `tunnel-mgr killall` | Kill all active tunnels |
| `payload-craft <type> <ip> <port>` | Generate reverse shell payloads |

### Bluetooth
| Command | Description |
|---|---|
| `sudo bt-scan` | BLE + Classic discovery |
| `sudo bt-deep <mac>` | Deep enumerate: name, class, SDP, LMP |
| `sudo bt-attack blueborne <mac>` | BlueBorne vulnerability test |
| `sudo bt-attack l2ping_flood <mac>` | L2CAP ping flood (DoS) |
| `sudo bt-attack rfcomm_scan <mac>` | RFCOMM channel scan |
| `sudo ble-gatt <mac>` | GATT service + handle enumeration |
| `sudo bettercap` | MITM + BLE attack framework |
| `sudo ble-spam samsung_tv` | Wall of Flippers: Samsung TV popup spam |
| `sudo ble-spam swift_pair` | Wall of Flippers: Swift Pair notifications |
| `sudo ble-spam air_pod` | Wall of Flippers: AirPods pairing spam |
| `sudo ble-spam all` | Cycle all BLE spam attacks |

### Captive Portal
| Command | Description |
|---|---|
| `sudo captive-portal start <ap> <inet> <ssid> [type]` | Rogue AP + credential harvesting |
| `sudo captive-portal stop` | Stop captive portal |
| `sudo captive-portal logs` | Show captured credentials |
| `sudo captive-portal list` | List portal templates |
| Portal types: `wifi` `corporate` `social` `email` `bank` `cloud` `hotel` `airport` `coffee` `library` `gym` `custom` |

### Wardriving
| Command | Description |
|---|---|
| `wardrive start [iface]` | GPS-tagged WiFi scanning + KML export |
| `wardrive stop` | Stop and save wardrive session |
| `wardrive export [session]` | Export to KML (Google Earth) |
| `wardrive list` | List wardrive sessions |
| `wardrive upload [session]` | Upload to WiGLE |

### Exfiltration & C2
| Command | Description |
|---|---|
| `exfil-discord setup <webhook>` | Configure Discord webhook |
| `exfil-discord send <msg>` | Send message to Discord |
| `exfil-discord file <path>` | Upload file to Discord |
| `exfil-discord loot` | Upload all loot files |
| `exfil-discord cmd <cmd>` | Run command + send output |
| `exfil-discord screenshot` | Capture + upload screenshot |
| `exfil-discord status` | Device status to Discord |
| `exfil-discord alert <msg>` | High-priority alert |
| `exfil-dns send <file>` | DNS data exfiltration |

### M5MonsterC5 — ESP32C5 WiFi Attack Board
| Command | Description |
|---|---|
| `monsterctl ping` | Verify board connection |
| `monsterctl status` | Board firmware, WiFi mode, running attacks |
| `monsterctl scan` | List all visible APs |
| `monsterctl select <n>` | Select target AP(s) |
| `monsterctl deauth` | Deauth all clients of selected AP |
| `monsterctl evil_twin` | Clone AP + start captive portal |
| `monsterctl sae_overflow` | Flood WPA3 SAE handshake (DoS) |
| `monsterctl karma` | Probe request responder |
| `monsterctl handshake` | Capture WPA/WPA2 handshake |
| `monsterctl sniffer` | Capture all WiFi traffic |
| `monsterctl blackout` | Mass deauth all visible APs |
| `monsterctl sniffer_dog` | Follow a specific client |
| `monsterctl beacon_spam` | Flood beacons with random SSIDs |
| `monsterctl rogue_ap` | Start standalone rogue AP |
| `monsterctl arp_poison <gw>` | ARP poisoning attack |
| `monsterctl deauth_detect` | Monitor for deauth frames |
| `monsterctl wardrive` | GPS-tagged wardrive scan |
| `monsterctl nmap <target>` | Port scan via board WiFi |
| `monsterctl stop` | Stop all running attacks |
| `monsterctl flash <src>` | Flash firmware (web/local/cardputer) |
| `monsterctl passwords` | Show captured credentials |
| `monsterctl hosts` | Show discovered hosts |
| `monsterctl wifi_connect "SSID" "pass"` | Connect board to WiFi |
| `monsterctl wifi_disconnect` | Disconnect board from WiFi |

### JanOS Interactive Controller
| Command | Description |
|---|---|
| `install-janos install` | Install JanOS-app TUI |
| `install-janos status` | Check installation status |
| `install-janos run [port]` | Launch interactive TUI |
| `install-janos update` | Pull latest from GitHub |

### Ragnar Reconnaissance
| Command | Description |
|---|---|
| `ragnar-ctl start` | Start Ragnar autonomous scanner (web UI) |
| `ragnar-ctl status` | Show scanner status + dashboard URL |
| `ragnar-ctl scan` | Trigger Ragnar network scan |
| `ragnar-ctl vuln [target]` | Trigger vulnerability scan |
| `ragnar-ctl auto` | Enable auto-scan + auto-attack |
| `ragnar-ctl manual` | Manual mode (no auto-attack) |
| `ragnar-scan <iface> <profile>` | Lightweight 3-phase recon (quick/full/vuln/stealth) |

### Device Lock & WebUI
| Command | Description |
|---|---|
| `device-lock lock` | Lock device (PIN/sequence) |
| `device-lock unlock` | Unlock device |
| `device-lock set-pin <pin>` | Set 4-digit PIN |
| `device-lock auto <sec>` | Set auto-lock timeout |
| `webui` | Start WebUI dashboard (HTTPS :8443) |

### NFC & Sub-GHz
| Command | Description |
|---|---|
| `sudo nfc-read` | Read NFC tags (Proxmark3 / nfcpy) |
| `sudo nfc-clone <uid|dump>` | Clone NFC tags |
| `sudo nfc-emulate <type>` | Emulate MIFARE/NTAG/EM4100 |
| `sudo subghz-scan [band]` | Scan Sub-GHz frequencies (RTL-433/CC1101) |
| `sudo subghz-record <freq> <time>` | Record Sub-GHz signals |
| `sudo subghz-replay <file>` | Replay captured Sub-GHz signals |

### Mesh / LoRa
| Command | Description |
|---|---|
| `mesh-chat chat` | Interactive Meshtastic LoRa chat |
| `mesh-chat send <channel> <msg>` | Send message to channel |
| `mesh-chat listen <channel>` | Continuous message monitoring |
| `mesh-chat nodes` | List discovered nodes |
| `mesh-chat info` | Show local node status |
| `mesh-setup install` | Full Meshtastic setup (CLI + dependencies) |
| `mesh-setup init` | Initialize and configure LoRa node |
| `mesh-setup info` | Node info, battery, signal, GPS |
| `mesh-setup send <msg> [node]` | Send encrypted message |
| `mesh-setup relay` | Enable mesh relay / internet bridge |

### IR — Infrared Hacking
| Command | Description |
|---|---|
| `sudo ir-scan` | Capture and decode IR signals |
| `sudo ir-replay <signal_file>` | Replay captured IR signals |
| `sudo ir-brute <protocol> [device]` | Brute-force IR power codes |

### Camera
| Command | Description |
|---|---|
| `cam-snap [output]` | Capture still image |
| `cam-stream [duration]` | Record video clip |
| `cam-ocr [output]` | Capture + Tesseract OCR |

### YouTube
| Command | Description |
|---|---|
| `yt search <query>` | Search and play YouTube videos |
| `yt play <url|id>` | Play YouTube video |
| `yt audio <url|id>` | Play audio only (saves battery) |
| `yt download <url>` | Download video to SD card |
| `yt download-audio <url>` | Download audio only (OPUS) |
| `yt trending` | Browse trending videos |
| `yt history` | Show play history |
| `export ZERODAY_DISPLAY=hdmi` | Enable HDMI video output for playback |

### Gaming — DOOM
| Command | Description |
|---|---|
| `doom-play play [wad]` | Launch DOOM (auto-detect WAD) |
| `doom-play shareware` | Download/setup DOOM shareware WAD |
| `doom-play list` | List installed WAD files |

### Retro Gaming
| Command | Description |
|---|---|
| `retro-play` | Interactive game picker (all systems) |
| `retro-play nes <rom>` | Launch NES game |
| `retro-play snes <rom>` | Launch SNES game |
| `retro-play gb <rom>` | Launch Game Boy game |
| `retro-play gba <rom>` | Launch Game Boy Advance game |
| `retro-play genesis <rom>` | Launch Sega Genesis game |
| `retro-play list [system]` | List available ROMs |
| `retro-play cores` | Check installed emulator cores |
| `retro-play setup` | Configure RetroArch for LCD |

### Hardware & Radio
| Command | Description |
|---|---|
| `sudo sdr-scan [freq_range]` | RTL-SDR frequency scan |
| `sudo rf-capture [freq]` | Raw RF capture and analysis |
| `sudo gpio-probe` | Enumerate I2C/SPI/UART devices |
| `sudo cardputer-battery` | BQ27220 fuel gauge readout |
| `sudo dongle-setup <cmd>` | RTL8821CU dongle manager |

### Terminal — zeroday-term

ZERO-DAY OS includes **zeroday-term**, a custom terminal emulator built in Rust and optimized for the Cardputer Zero's 1.9" display and 46-key keyboard. It replaces `st` as the primary terminal for wayland/GUI mode and falls back to `st` under X11.

| Feature | Description |
|---|---|
| **Status bar** | Battery%, WiFi IP, CPU temp, load avg, time — always visible |
| **Fn-key shortcuts** | Fn+Enter (new terminal), Fn+Esc (close), Fn+PgUp/PgDn (font size) |
| **Scrollback** | Ctrl+Shift+Up/Down for scroll, Ctrl+Shift+C/V for copy/paste |
| **Tiny footprint** | ~1.2MB stripped binary, no desktop dependencies, no Smithay dependency |
| **256-color** | Full xterm-256color support for hacking tools (vte-based parser) |
| **No mouse needed** | Full keyboard-driven workflow |
| **PTY-based** | Uses portable-pty for proper pseudo-terminal management |
| **DRM-ready** | Renderer designed for direct framebuffer/KMS output (WIP) |

Installed to `/usr/local/bin/zeroday-term` with symlink `st → zeroday-term` for compatibility. Falls back to `stterm` if zeroday-term is unavailable.

Config: `/etc/zeroday/term.env` — font size (default 8), dimensions (40x19), shell, status bar toggle.

### Reverse Shells & Exploitation
| Command | Description |
|---|---|
| `quick-c2 listen [port]` | Encrypted C2 listener (socat + OpenSSL) |
| `quick-c2 payload <type>` | Generate shell one-liners |
| `revshell-stabilize` | Cheatsheet for shell stabilization |
| `john hash <file>` | John the Ripper password cracker |
| `hydra <mode> <target>` | Hydra online credential brute-force |
| `gobuster dir <url>` | Web directory brute-force |
| `arpspoof start <target> <gw>` | ARP spoof MITM |
| `ntlmrelayx relay <target>` | NTLM authentication relay |

### System & Field Ops
| Command | Description |
|---|---|
| `panic` | KILL EVERYTHING — kill processes, wipe history, sanitize |
| `stealth-backlight-toggle` | Kill/restore LCD backlight (stealth mode) |
| `zeroday-boot` | Boot orchestration (drivers, CPU gov, compositor start) |
| `zeroday-bootanim` | Boot animation (glitch ASCII) |
| `zeroday-term` | Custom terminal emulator (320x170, 46-key optimized, status bar) |
| `power-mode <profile>` | performance / balanced / stealth |
| `cardputer-wifi-toggle` | Toggle wlan0 on/off |
| `cardputer-wifi-setup` | Interactive WiFi configurator |
| `usb-gadget-mode <type>` | USB device mode (HID/serial/NCM/storage) |
| `first-boot` | First-boot wizard (filesystem expand, password, WiFi) |
| `opencode-session` | tmux split-screen IDE |
| `opencode-ask "<question>"` | Inline AI query |
| `tamper-watch` | BMI270 tamper detection daemon |
| `mac-rotate <iface> <action>` | MAC randomization (random/restore/status) |
| `loot-organize` | Organize loot directory by type |

### Entertainment
| Command | Description |
|---|---|
| `webradio-danish [STATION]` | Stream Danish web radio |
| `music-player [dir]` | Play local MP3/FLAC files |
| `yt search <query>` | YouTube search and play |
| `doom-play play` | Launch DOOM |
| `retro-play` | Interactive retro game launcher |

---

## Display System — zeroday-comp Compositor

ZERO-DAY OS uses **zeroday-comp**, a custom Rust Wayland compositor purpose-built for the Cardputer Zero's 320x170 screen, 46-key keyboard, and 512MB RAM. It replaces cage as the primary display server with lower memory usage and native Fn-key compositor bindings. The companion **zeroday-term** terminal emulator provides an optimized terminal for the same hardware constraints.

| Priority | Interface | Compositor | Terminal | RAM | Use Case |
|---|---|---|---|---|---|
| **Primary** | GUI Launcher (Pygame) | zeroday-comp (custom Wayland) | zeroday-term | ~2MB + ~1.2MB | Daily use, Fn-keys, big icons |
| **Fallback 1** | GUI Launcher (Pygame) | cage (Wayland kiosk) | foot/st | ~3MB | If zeroday-comp fails |
| **Fallback 2** | TUI Launcher (Pygame) | Xorg + i3 + st | stterm | ~30MB | If all Wayland fails |

### zeroday-comp Features
- **Single-client kiosk**: No window management overhead — runs `cyber_launcher` fullscreen
- **Native Fn-key bindings**: Fn+P (panic), Fn+Space (stealth/backlight), Fn+Tab (launcher), Fn+Q (kill), plus all quick-launch combos — handled at the compositor level
- **DRM/KMS direct rendering**: Zero-copy GPU path for ST7789V LCD (WIP — currently launches client directly)
- **Sub-2MB RAM**: Written in Rust, stripped release binary (~1.0MB), no desktop shell protocol overhead
- **Clean shutdown**: SIGTERM kills child processes (cyber_launcher, zeroday-term) before compositor exit
- **Automatic backlight control**: Fn+Space toggles LCD backlight for stealth mode
- **Fallback chain**: If zeroday-comp crashes, systemd automatically starts cage; if cage fails, Xorg+i3 takes over

### zeroday-term Features
- **Optimized for 320x170**: Default 40x19 columns, 8pt font, status bar with system info
- **Status bar**: Battery%, WiFi IP, CPU temp, load avg, clock — always visible
- **Fn-key compositor shortcuts**: Work at the compositor level regardless of running app
- **~1.2MB stripped**: Rust binary, panic=abort, LTO, opt-level=z
- **vte-based terminal parser**: Full xterm-256color, proper escape sequences
- **PTY management**: portable-pty for proper process groups and signal handling

If zeroday-comp and cage both fail to start, `zeroday-tui.service` automatically takes over with Xorg + i3 + stterm.

Set `ZERODAY_DISPLAY=hdmi` for fullscreen video/gaming on external HDMI output.

---

## Keyboard Map — The Omni-Key System

46 keys. One `Fn` key. Zero mouse. Every action is 2 keypresses from anywhere. Fn-key combos are handled at the **compositor level** by `zeroday-comp` — they work even if the GUI launcher crashes.

```
 ┌─────────────────────────────────────────────┐
 │  Fn + Tab   → GUI Launcher toggle           │
 │  Fn + P     → PANIC (kill all + wipe)       │
 │  Fn + Space → STEALTH (kill backlight)       │
 │  Fn + Return→ Quick terminal                │
 │  Fn + Q     → Close tile                    │
 │  Fn + O     → OpenCode                      │
 │  Fn + L     → Device lock                    │
 │                                              │
 │  Fn + N     → Nmap QuickScan                 │
 │  Fn + B     → Bluetooth scan                │
 │  Fn + S     → Shell listener                │
 │  Fn + W     → WiFi monitor toggle           │
 │  Fn + C     → Camera snap                   │
 │  Fn + I     → IR scan                       │
 │  Fn + A     → opencode-ask                  │
 │  Fn + G     → DOOM                           │
 │  Fn + R     → Retro games                    │
 │  Fn + Y     → YouTube search                │
 │  Fn + U     → WebUI dashboard               │
 └─────────────────────────────────────────────┘
```

Key bindings are intercepted by the compositor's input handler (`/compositor/src/input.rs`), not by i3 or the Pygame launcher. This means `Fn+P` (panic) and `Fn+Space` (stealth backlight toggle) work regardless of application state.

---

## Building the OS Image

Built from scratch using Docker for full reproducibility. aarch64 cross-compilation via QEMU. The `zeroday-comp` Rust compositor and `zeroday-term` Rust terminal emulator are cross-compiled on the host and installed as pre-built binaries into the image.

### Prerequisites (x86 Linux Host)

```bash
# Arch Linux / CachyOS
sudo pacman -S docker qemu-user-static
sudo systemctl enable --now docker

# Debian / Ubuntu
sudo apt install docker.io qemu-user-static binfmt-support
sudo systemctl enable --now docker
```

### Cross-compile Rust Components (before pi-gen build)

```bash
# Install cross-rs (Docker-based cross-compilation)
cargo install cross

# Build compositor (currently a stub launcher — DRM backend WIP)
cd compositor
make deps          # Build custom cross-rs Docker image with Wayland/DRM libs
make cross-build   # Cross-compile for aarch64

# Build terminal emulator
cd ../terminal
make cross-build   # Cross-compile zeroday-term for aarch64

# Binaries land in:
#   compositor/target/aarch64-unknown-linux-gnu/release/zeroday-comp  (~1.0MB)
#   terminal/target/aarch64-unknown-linux-gnu/release/zeroday-term     (~1.2MB)
```

### Build OS Image

```bash
cd pi-gen
chmod +x build-docker.sh build.sh
./build-docker.sh
# ~25-30min. Downloads Debian arm64 base + Kali tools.
# Pre-built zeroday-comp and zeroday-term binaries are copied into the rootfs.
```

Retrieve `.img` from `pi-gen/deploy/` and flash to a **microSD card**:
```bash
sudo dd if=zeroday-os.img of=/dev/sdX bs=4M status=progress conv=fsync
```

### Building Individual Components

To cross-compile `zeroday-comp` separately:
```bash
cd compositor
make deps          # Build cross-rs Docker image
make cross-build   # Cross-compile for aarch64
make strip         # Strip binary (~1.0MB)
```

To cross-compile `zeroday-term` separately:
```bash
cd terminal
make cross-build   # Cross-compile for aarch64 (~1.2MB stripped)
```

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

### First Boot
1. Login: `root` / `zeroday` — **change immediately**: `passwd`
2. Configure WiFi: `cardputer-wifi-setup`
3. Launch the GUI: `Fn + Tab` or run `cyber_launcher`
4. Open OpenCode: `Fn + O` or run `opencode-session`

The boot chain tries `zeroday-comp` first, falls back to `cage` if it fails, then `Xorg+i3` if Wayland is unavailable.

---

## Threat Model & Ethics

ZERO-DAY OS is a professional tool for **authorized security testing**. The panic key exists because real pentesters sometimes need to disappear fast. All actions are logged locally for your engagement report.

**Do not use this on networks or devices you don't own or have explicit written authorization to test.**

---

## Credits

- **M5Stack** — Cardputer Zero hardware and official DT overlays
- **Raspberry Pi Foundation** — RP3A0 SoC and pi-gen build system
- **Kali Linux** — Tool repositories
- **Smithay** — Rust Wayland compositor library (zeroday-comp backend)
- **portable-pty** — Cross-platform PTY library (zeroday-term backend)
- **vte-rs** — Terminal escape sequence parser (zeroday-term)
- **OpenCode** — On-device AI-assisted code editor (v1.14.49)
- **dianjixz** — CM0 firmware reference
- **Offensive Security** — Training and tool ecosystem
- **The Flipper Zero community** — TUI design inspiration

---

<p align="center">
<strong>Built for the field. Designed for the edge. Fits in your wallet.</strong>
</p>