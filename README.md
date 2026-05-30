# ZERO-DAY OS

<p align="center">
  <img src="assets/logo.png" alt="ZERO-DAY OS Logo" width="480">
</p>

**The first penetration testing OS built for a credit-card-sized computer you can hold in one hand.**

ZERO-DAY OS v2.0 turns the M5Stack Cardputer Zero — a quad-core ARM64 box with WiFi, BT, IR, a camera, a battery, and a built-in keyboard — into a pocketable offensive security weapon. Every byte of this distro is optimized for the constraints of 512MB RAM and a 1.9" screen. No desktop. No bloat. No compromises.

[![Release v2.0](https://img.shields.io/github/v/release/jayis1/Zero-Day-OS-?label=latest%20release)](https://github.com/jayis1/Zero-Day-OS-/releases/latest)

---

## What Makes This Different

You can install Kali on a Raspberry Pi. That's not what this is.

| Stock Pi + Kali | ZERO-DAY OS |
|---|---|
| Boots into a desktop you can't use on 1.9" | Boots straight into a Textual TUI launcher |
| 2GB+ RAM just for the DE | ~60MB idle, 512MB total — 450MB for tools |
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
| **512MB RAM** | `musl` where possible, `dropbear` over `sshd`, no `postgres`, no heavy daemons. Metasploit excluded. |
| **1.9" 320x170 display** | GUI launcher with big icons — Wayland kiosk (cage) primary, Xorg+i3 TUI fallback |
| **46-key matrix keyboard** | `Fn` Omni-Key system. Every tool is 2 keypresses from anywhere. |
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
 │   │     (Wayland kiosk via cage, primary display)    │    │
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
 │   │   Fallback: Xorg + i3 + st (TUI mode)            │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 │   ┌───────────┐  ┌───────────┐  ┌───────────────────┐   │
 │   │  i3 wm    │  │  OpenCode │  │  Panic System      │   │
 │   │ (tiling)  │  │  (editor) │  │  (kill + wipe)     │   │
 │   └───────────┘  └───────────┘  └───────────────────┘   │
 │                                                           │
 │   ┌─────────────────────────────────────────────────┐    │
     │   │   One-Key Hacking Scripts  /usr/local/bin       │    │
     │   │   wifi-* · net-* · bt-* · ir-* · cam-*          │    │
     │   │   nfc-* · subghz-* · mesh-* · dongle-* · sdr-* │    │
     │   │   arpspoof · ntlmrelayx · exfil-dns · gobuster   │    │
     │   │   john · hydra · bettercap · ragnar-ctl · panic   │    │
  │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   Debian Bookworm arm64  +  Kali Rolling repos  │    │
 │   │   aircrack · nmap · bettercap · sqlmap · john    │    │
 │   │   hydra · gobuster · dsniff · responder · curl   │    │
 │   │   hashcat-utils · hcxdumptool · meshtastic       │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   RP3A0 Device Tree Overlays                      │    │
 │   │   SPI (LCD) · I2C (kbd,IMU,battery,RTC,IO)      │    │
 │   │   I2S (audio) · CSI (camera) · GPIO (IR,USB)    │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
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
| `zeroday-boot` | Boot orchestration (drivers, CPU gov, Xorg start) |
| `zeroday-bootanim` | Boot animation (glitch ASCII) |
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

## Display System — GUI Primary

ZERO-DAY OS uses a **Wayland-based GUI launcher** as the primary interface, with a TUI fallback:

| Priority | Interface | Compositor | RAM | Use Case |
|---|---|---|---|---|
| **Primary** | GUI Launcher (Pygame) | cage (Wayland kiosk) | ~25MB | Daily use, big icons |
| **Fallback** | TUI Launcher (Pygame) | Xorg + i3 + st | ~30MB | If cage fails |

The GUI launcher features **large category icons** designed for the 1.9" (320x170) screen. Each app category is represented by a full-color icon that fills a grid cell, making navigation precise despite the small display.

If cage (Wayland) fails to start, `zeroday-tui.service` automatically takes over with Xorg + i3 + st.

Set `ZERODAY_DISPLAY=hdmi` for fullscreen video/gaming on external HDMI output.

---

## Keyboard Map — The Omni-Key System

46 keys. One `Fn` key. Zero mouse. Every action is 2 keypresses from anywhere.

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

---

## Building the OS Image

Built from scratch using Docker for full reproducibility. aarch64 cross-compilation via QEMU.

### Prerequisites (x86 Linux Host)

```bash
# Arch Linux / CachyOS
sudo pacman -S docker qemu-user-static
sudo systemctl enable --now docker

# Debian / Ubuntu
sudo apt install docker.io qemu-user-static binfmt-support
sudo systemctl enable --now docker
```

### Build
```bash
cd pi-gen
chmod +x build-docker.sh build.sh
./build-docker.sh
# ~20min. Downloads Debian arm64 base + Kali tools. Go get coffee.
```

Retrieve `.img` from `pi-gen/deploy/` and flash to a **microSD card**:
```bash
sudo dd if=zeroday-os.img of=/dev/sdX bs=4M status=progress conv=fsync
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
3. Launch the TUI: `Fn + Tab` or run `cyber_launcher`
4. Open OpenCode: `Fn + O` or run `opencode-session`

---

## Threat Model & Ethics

ZERO-DAY OS is a professional tool for **authorized security testing**. The panic key exists because real pentesters sometimes need to disappear fast. All actions are logged locally for your engagement report.

**Do not use this on networks or devices you don't own or have explicit written authorization to test.**

---

## Credits

- **M5Stack** — Cardputer Zero hardware and official DT overlays
- **Raspberry Pi Foundation** — RP3A0 SoC and pi-gen build system
- **Kali Linux** — Tool repositories
- **OpenCode** — On-device AI-assisted code editor (v1.14.49)
- **dianjixz** — CM0 firmware reference
- **Offensive Security** — Training and tool ecosystem
- **The Flipper Zero community** — TUI design inspiration

---

<p align="center">
<strong>Built for the field. Designed for the edge. Fits in your wallet.</strong>
</p>
