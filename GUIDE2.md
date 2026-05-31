# ZERO-DAY OS — M5MonsterC5 Firmware Guide

**Complete guide to building, flashing, and operating the custom ZERO-DAY firmware for the M5MonsterC5 (ESP32C5).**

*Last updated: 2026-05-31*

---

## Table of Contents

1. [Overview](#1-overview)
2. [Hardware Topology](#2-hardware-topology)
3. [Simultaneous Operation](#3-simultaneous-operation)
4. [Firmware Architecture](#4-firmware-architecture)
5. [Serial Protocol](#5-serial-protocol)
6. [Building the Firmware](#6-building-the-firmware)
7. [Flashing](#7-flashing)
8. [Serial Console](#8-serial-console)
9. [Command Reference](#9-command-reference)
10. [GPS Passthrough](#10-gps-passthrough)
11. [Zigbee/Thread Hacking via C6L](#11-zigbeethread-hacking-via-c6l)
12. [C6L Command Routing](#12-c6l-command-routing)
13. [Meshtastic Mesh](#13-meshtastic-mesh)
14. [WiFi Attack Commands](#14-wifi-attack-commands)
15. [Board Detection](#15-board-detection)
16. [UART Multiplexing](#16-uart-multiplexing)
17. [OLED Status Display (GPIO Hat)](#17-oled-status-display-gpio-hat)
18. [Pin Configuration](#18-pin-configuration)
19. [Partition Table](#19-partition-table)
20. [Upstream vs ZERO-DAY](#20-upstream-vs-zeroday)
21. [Troubleshooting](#21-troubleshooting)
22. [File Layout](#22-file-layout)

---

## 1. Overview

The M5MonsterC5 runs custom ZERO-DAY firmware forked from [C5Lab/M5MonsterC5-CardputerADV](https://github.com/C5Lab/M5MonsterC5-CardputerADV). It serves as the **middle-manager hub** between the Cardputer Zero (main OS) and the peripheral modules:

- Retains all upstream WiFi attack capabilities (deauth, evil twin, SAE overflow, handshake capture, sniffer, karma, beacon spam, blackout, wardrive)
- Adds GPS passthrough via Grove IN (AT6558, UART 9600)
- Adds C6L routing via Grove OUT (ESP32-C6, I2C + UART 115200)
- Adds native Meshtastic LoRa mesh on the ESP32C5 radio
- Multiplexes all communication over a single USB/UART link to the Cardputer Zero

The firmware is written in C using ESP-IDF v5.4, targeting the ESP32C5 (160MHz, 400KB SRAM, 4MB Flash).

---

## 2. Hardware Topology

```
Cardputer Zero (aarch64, main OS)
  ├── USB/UART ──→ M5MonsterC5 (ESP32C5, middle manager)
  │                    ├── Grove IN  ← GPS Module v1.1 (AT6558 GNSS, UART 9600)
  │                    ├── Grove OUT → Unit C6L (ESP32-C6, Zigbee/Thread/BLE/LCD, I2C+UART)
  │                    └── LoRa radio → Meshtastic mesh node
  ├── 14-pin expansion → M5Stack NFC/CC1101 Module
  │    ├── CC1101 Sub-GHz transceiver (SPI, on the module PCB)
  │    └── Module Grove port → SH1107 OLED or PN532 NFC/RFID2 (I2C, time-shared)
  └── (LoRa hat module swaps with NFC/CC1101 module — only one at a time)
```

The M5Stack NFC/CC1101 module and LoRa module both connect to the Cardputer Zero's 14-pin expansion port — they swap the same slot. The M5Stack module has its own Grove port for I2C devices (OLED + NFC, time-shared). The MonsterC5 USB connection is separate.

---

## 3. Simultaneous Operation

All three radio domains can operate **simultaneously** because they use independent hardware paths:

```
┌──────────────────────────────────────────────────────────────┐
│                  Cardputer Zero (main OS)                     │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ WiFi attacks  │  │ Zigbee/Thread │  │  Meshtastic mesh  │  │
│  │ (monsterctl)  │  │ (c6l-ctl)    │  │  (monsterctl mesh) │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬──────────┘  │
│         │                  │                    │             │
│         │    ┌─────────────┴────────────────────┘             │
│         │    │  Single USB/UART to MonsterC5                  │
│         │    │  (serial multiplexing: no prefix, C6L:, MESH:) │
│         ▼    ▼                                                │
│  ┌──────────────────┐                                        │
│  │  M5MonsterC5     │                                        │
│  │  (ESP32C5)       │                                        │
│  │                   │                                        │
│  │  WiFi 6 radio ──────→ 2.4/5 GHz attacks (independent)    │
│  │  UART1 mux ────────→ GPS or C6L (time-shared)            │
│  │  I2C0 ─────────────→ C6L LCD (independent)               │
│  │  LoRa radio ───────→ Meshtastic mesh (independent)       │
│  │                   │                                        │
│  │  Grove IN ────────→ GPS Module v1.1 (UART @ 9600)        │
│  │  Grove OUT ───────→ Unit C6L (I2C + UART @ 115200)      │
│  └──────────────────┘                                        │
│                                                              │
│  ┌──────────────────┐                                        │
│  │  GPIO Hat        │                                        │
│  │  (direct I2C)   │                                        │
│  │                   │                                        │
│  │  Grove → OLED ─────→ SH1107 status display (independent)  │
│  │  Grove → NFC ──────→ PN532 RFID (same port, time-shared) │
│  │  14-pin → CC1101 ──→ Sub-GHz radio (independent)        │
│  └──────────────────┘                                        │
└──────────────────────────────────────────────────────────────┘
```

### What can run at the same time

| Combination | Simultaneous? | Why |
|---|---|---|
| WiFi attacks + Mesh | Yes | Different radios on ESP32C5 |
| WiFi attacks + C6L/Zigbee | Yes | WiFi on C5, 802.15.4 on C6 |
| WiFi attacks + GPS | Yes | WiFi on C5, UART1 for GPS |
| C6L/Zigbee + Mesh | Yes | C6L on C6 UART, mesh on C5 LoRa |
| C6L/Zigbee + GPS | **No** | Share UART1 (time-multiplexed) |
| Mesh + GPS | Yes | Mesh on C5 LoRa, GPS on UART1 |
| WiFi attacks + OLED display | Yes | Independent paths (MonsterC5 vs GPIO hat) |
| NFC/RFID + OLED | **No** | Share GPIO hat Grove I2C port |
| CC1101 Sub-GHz + OLED | Yes | SPI vs I2C, independent |
| CC1101 + NFC | **No** | Same Grove I2C port, time-shared |

### Command flow for simultaneous operation

```bash
# Terminal 1: WiFi attack via MonsterC5
monsterctl scan
monsterctl select 2
monsterctl deauth

# Terminal 2: Zigbee/Thread scan via C6L (through MonsterC5)
c6l-ctl zigbee scan

# Terminal 3: Meshtastic mesh (through MonsterC5)
monsterctl mesh start
monsterctl mesh send 1 "target acquired"

# Terminal 4: OLED status display (direct GPIO hat, independent)
oled-ctl daemon

# All three radios active simultaneously:
# - ESP32C5 WiFi 6 → deauth attack on 2.4/5 GHz
# - ESP32C6 802.15.4 → Zigbee/Thread scanning
# - ESP32C5 LoRa → Meshtastic mesh C2
# - AT6558 GNSS → GPS location (if GPS passthrough active)
# - SH1107 OLED → status dashboard (independent via GPIO hat)
```

---

## 4. Firmware Architecture

### Source files

| File | Purpose |
|---|---|
| `zeroday_monsterc5.c` | Main entry point — initializes NVS, netif, serial mux, WiFi, mesh |
| `serial_mux.c` | UART0 console multiplexer — sends/receives line-based data to Cardputer Zero |
| `gps_passthrough.c` | GPS passthrough — reads AT6558 NMEA on UART1, prefixes with `GPS:` |
| `c6l_routing.c` | C6L routing — configures UART1 for C6L at 115200, I2C for LCD at 0x3C |
| `mesh_node.c` | Meshtastic stub — starts/stops mesh, sends/receives messages |
| `wifi_attack.c` | WiFi attack command dispatcher — receives commands, dispatches to attack functions |
| `board_detect.c` | Hardware auto-detection on Grove ports (GPS, C6L LCD) |
| `include/zeroday_monsterc5.h` | Shared header — UART/I2C/pin defines, peripheral mode enum, function declarations |

### Boot sequence

1. `app_main()` initializes NVS flash
2. `esp_netif_init()` + `esp_event_loop_create_default()` — networking stack
3. `serial_mux_init()` — configure UART0 at 115200 baud (console to Cardputer Zero)
4. `board_detect_all()` — probe Grove IN for GPS, Grove OUT for C6L LCD
5. `wifi_attack_init()` — initialize WiFi attack engine
6. `mesh_node_init()` — initialize Meshtastic node (not started until `mesh_start` command)
7. Main loop — idle, all work happens in FreeRTOS tasks spawned by commands

### Peripheral mode state machine

UART1 is shared between GPS and C6L. Only one can be active at a time:

```
PERIPH_MODE_IDLE → gps_passthrough_start() → PERIPH_MODE_GPS
                                        stop → PERIPH_MODE_IDLE

PERIPH_MODE_IDLE → c6l_passthrough_start() → PERIPH_MODE_C6L
                                         stop → PERIPH_MODE_IDLE
```

Switching between GPS and C6L requires stopping the current passthrough first.

---

## 5. Serial Protocol

All communication with Cardputer Zero flows over UART0 at 115200 baud, 8N1. The firmware uses line-based text protocol with prefixes:

### Inbound (Cardputer Zero → MonsterC5)

Lines without a recognized prefix are treated as WiFi attack commands.

| Prefix | Target | Example |
|---|---|---|
| (none) | WiFi attack engine | `scan`, `deauth`, `status` |
| `gps_passthrough_start` | GPS module | `gps_passthrough_start` |
| `gps_passthrough_stop` | GPS module | `gps_passthrough_stop` |
| `c6l_passthrough_start` | C6L module | `c6l_passthrough_start` |
| `c6l_passthrough_stop` | C6L module | `c6l_passthrough_stop` |
| `c6l_cmd <cmd>` | C6L command | `c6l_cmd ZIGBEE_SCAN` |
| `mesh_start` | Mesh node | `mesh_start` |
| `mesh_stop` | Mesh node | `mesh_stop` |
| `mesh_send <dest> <msg>` | Mesh TX | `mesh_send 1 hello` |
| `mesh_status` | Mesh node | `mesh_status` |
| `ping` | Echo | `ping` → `pong` |
| `hub_status` | Board detect | `hub_status` |

### Outbound (MonsterC5 → Cardputer Zero)

| Prefix | Source | Example |
|---|---|---|
| (none) | WiFi attack output | `SCAN_START`, `DEAUTH_START` |
| `GPS:` | AT6558 NMEA data | `GPS:$GPGGA,...` |
| `C6L:` | C6L responses | `C6L:ZIGBEE_SCAN_RESULTS:...` |
| `MESH:` | Mesh messages | `MESH:NODE_STARTED`, `MESH:SEND_OK:1:hello` |

---

## 6. Building the Firmware

### Prerequisites

- ESP-IDF v5.4 installed at `~/esp/esp-idf`
- CMake + Ninja (Arch: `sudo pacman -S cmake ninja pkg-config`)
- Python 3.8+ (provided by ESP-IDF)
- Git (for version tagging)

### Build commands

```bash
# On fish shell, all commands must go through bash:
bash -c 'source ~/esp/esp-idf/export.sh && make setup'   # First time only: set target to esp32c5
bash -c 'source ~/esp/esp-idf/export.sh && make build'    # Build firmware
bash -c 'source ~/esp/esp-idf/export.sh && make clean'   # Clean build artifacts
bash -c 'source ~/esp/esp-idf/export.sh && make dist'    # Copy binary to deploy dir
```

Or use `idf.py` directly:

```bash
bash -c 'source ~/esp/esp-idf/export.sh && idf.py --preview set-target esp32c5'
bash -c 'source ~/esp/esp-idf/export.sh && idf.py build'
```

### Build output

```
firmware/monsterc5/build/
├── zeroday-monsterc5.bin         # Main firmware (~298KB)
├── bootloader/bootloader.bin     # ESP-IDF bootloader (~21KB)
└── partition_table/partition-table.bin  # Partition table (~3KB)
```

### Flash addresses

| Component | Address |
|---|---|
| Bootloader | `0x2000` |
| Partition table | `0x8000` |
| Factory app | `0x10000` |

---

## 7. Flashing

### Via USB (from development machine)

```bash
# Flash all three components:
bash -c 'source ~/esp/esp-idf/export.sh && idf.py -p /dev/ttyUSB0 flash'

# Or use esptool directly:
esptool.py --chip esp32c5 -b 460800 \
    --before default_reset --after hard_reset \
    write_flash \
    --flash_mode dio --flash_size 2MB --flash_freq 80m \
    0x2000 build/bootloader/bootloader.bin \
    0x8000 build/partition_table/partition-table.bin \
    0x10000 build/zeroday-monsterc5.bin
```

### Via monsterctl (from Cardputer Zero)

```bash
monsterctl flash local        # Flash ZERO-DAY firmware from /opt/cardputer/monsterc5/firmware/
monsterctl flash upstream     # Flash upstream JanOS firmware
```

### Deployment

Copy binaries to the Cardputer Zero deploy directory:

```bash
make dist    # Copies to /opt/cardputer/monsterc5/firmware/
# Or manually:
cp build/zeroday-monsterc5.bin /opt/cardputer/monsterc5/firmware/
```

---

## 8. Serial Console

Connect to the MonsterC5's USB serial port to see firmware output and send commands:

```bash
# From Cardputer Zero:
monsterctl ping                    # Test connection

# From any machine via USB:
minicom -D /dev/ttyUSB0 -b 115200
# Or:
picocom /dev/ttyUSB0 -b 115200

# From ESP-IDF monitor:
bash -c 'source ~/esp/esp-idf/export.sh && idf.py -p /dev/ttyUSB0 monitor'
```

The firmware prints boot diagnostics showing detected hardware and initialized subsystems. All commands are case-sensitive, line-delimited with `\r\n` terminators.

---

## 9. Command Reference

### System commands

| Command | Response | Description |
|---|---|---|
| `ping` | `pong` | Connection test |
| `status` | Status info | WiFi mode, attacks, GPS, C6L, mesh |
| `hub_status` | `HUB_STATUS:...` | Grove topology and device detection |

### GPS commands

| Command | Response | Description |
|---|---|---|
| `gps_passthrough_start` | Streams `GPS:` lines | Start GPS passthrough (UART1 @ 9600) |
| `gps_passthrough_stop` | `GPS:PASSTHROUGH_STOPPED` | Stop GPS passthrough |
| `wardrive` | Streams scan results | Scan APs with GPS coordinates |

### C6L commands

| Command | Response | Description |
|---|---|---|
| `c6l_passthrough_start` | Streams `C6L:` lines | Start C6L passthrough (UART1 @ 115200) |
| `c6l_passthrough_stop` | `C6L:PASSTHROUGH_STOPPED` | Stop C6L passthrough |
| `c6l_cmd <cmd>` | `C6L:<response>` | Send command to C6L unit |

### Mesh commands

| Command | Response | Description |
|---|---|---|
| `mesh_start` | `MESH:NODE_STARTED` | Start Meshtastic node |
| `mesh_stop` | `MESH:NODE_STOPPED` | Stop mesh node |
| `mesh_send <dest> <msg>` | `MESH:SEND_OK:<dest>:<msg>` | Send mesh message |
| `mesh_status` | `MESH:STATUS:RUNNING/STOPPED:CH6` | Check mesh status |

### WiFi attack commands

| Command | Response | Description |
|---|---|---|
| `scan_networks` | `SCAN_START` | Start WiFi scan |
| `start_deauth` | `DEAUTH_START` | Deauth attack |
| `start_evil_twin` | `EVILTWIN_START` | Evil twin AP |
| `start_sae_overflow` | `SAE_OVERFLOW_START` | WPA3 SAE overflow |
| `start_handshake` | `HANDSHAKE_START` | Handshake capture |
| `start_sniffer` | `SNIFFER_START` | WiFi sniffer |
| `start_blackout` | `BLACKOUT_START` | Mass deauth |
| `start_wardrive` | `WARDRIVE_START` | Wardrive scan |
| `stop` | `STOPPED` | Stop all attacks |

---

## 10. GPS Passthrough

The M5Stack GPS Module v1.1 (AT6558 GNSS chip) connects to Grove IN. It supports GPS, BDS, GLONASS, GALILEO, and QZSS.

### Wiring

```
GPS Module v1.1    M5MonsterC5 Grove IN
───────────────    ─────────────────────
VCC             →  VCC (3.3V/5V)
TX              →  RX (GPIO5)
RX              →  TX (GPIO4)
GND             →  GND
```

### Operation

GPS passthrough uses UART1 at 9600 baud. Only one peripheral can use UART1 at a time — GPS and C6L are time-multiplexed.

```bash
# Enable GPS passthrough:
gps_passthrough_start

# NMEA data streams to Cardputer Zero with GPS: prefix:
# GPS:$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,*47
# GPS:$GPGSA,A,3,08,11,23,28,32,17,19,30,,,,1.2,0.9,0.8*3A

# Disable GPS passthrough:
gps_passthrough_stop
```

The `gps-ctl` script on Cardputer Zero handles this automatically:

```bash
gps-ctl start        # Start GPS daemon (routes via monsterctl)
gps-ctl location     # Show current lat/lon/alt
gps-ctl track        # Start GPS tracking
```

---

## 11. Zigbee/Thread Hacking via C6L

The Unit C6L (ESP32-C6) is the **only device in the ZERO-DAY ecosystem with native 802.15.4** — making it the dedicated Zigbee 3.0 and Thread 1.3 radio. This is its primary value; WiFi attacks are handled by the MonsterC5's ESP32C5 (which also supports 5 GHz).

### C6L radio capabilities

| Protocol | Frequency | Range | Use Case |
|---|---|---|---|
| Zigbee 3.0 | 2.4 GHz | ~100m | Smart home hacking, IoT device interception |
| Thread 1.3 | 2.4 GHz | ~100m | HomePod, Nest, Matter device attacks |
| BLE 5.0 | 2.4 GHz | ~50m | BLE device discovery, GATT exploration |
| WiFi 6 (802.11ax) | 2.4/5 GHz | ~100m | Bonus — MonsterC5 is the primary WiFi radio |

### Command flow

All C6L commands route through the MonsterC5 middle manager:

```
Cardputer Zero ──USB/UART──→ MonsterC5 ──UART1──→ C6L (Zigbee/Thread/BLE)
                                  │                        │
                                  └──I2C0──→ C6L LCD ──────┘
```

### Commands

```bash
# Scan for Zigbee/Thread networks
c6l-ctl zigbee scan
# Output: PAN ID, channel, network name, device count, permit join status

# Start Zigbee packet capture (15 min default)
c6l-ctl zigbee sniffer
# Output saved to: /opt/cardputer/loot/zigbee/zigbee_capture_*.pcap

# Attack a Zigbee network
c6l-ctl zigbee attack <PAN_ID>
# Modes: join/intercept, replay, key decrypt

# BLE 5 device scan
c6l-ctl ble scan
# Output: MAC, name, RSSI, service UUIDs

# Connect to a BLE peripheral
c6l-ctl ble connect AA:BB:CC:DD:EE:FF
# Opens GATT session for reading/writing characteristics

# WiFi 6 scan (bonus — MonsterC5 is primary for attacks)
c6l-ctl wifi scan

# WiFi 6 deauth (for 802.11ax-specific targets)
c6l-ctl wifi deauth <BSSID> <CHANNEL>
```

### Display on C6L LCD

The C6L has a built-in 0.96" SSD1306 OLED (128x64) at I2C address 0x3C. Text is sent via the serial `C6L:` prefix:

```bash
# Display text on C6L LCD
c6l-ctl lcd text "ZIGBEE SCAN"
c6l-ctl lcd text "ATTACK ACTIVE"

# Show system status on C6L LCD
c6l-ctl lcd status
# Shows: BAT:87% IP:192.168.1.5 L:0.42

# Or directly via monsterctl:
monsterctl c6l_cmd 'LCD:1:hello'
monsterctl c6l_cmd 'LCD:4:BAT:87% IP:192.168.1.5'
```

### Zigbee/Thread attack modes

The C6L's 802.15.4 radio supports these attack scenarios:

| Attack | Description | Command |
|---|---|---|
| Network scan | Discover all Zigbee/Thread networks in range | `c6l-ctl zigbee scan` |
| Packet capture | Capture and decode Zigbee frames | `c6l-ctl zigbee sniffer` |
| Network join | Join a Zigbee network and intercept traffic | `c6l-ctl zigbee attack <PAN_ID>` |
| Key recovery | Capture key exchange, decrypt traffic | `c6l-ctl zigbee attack key <key>` |
| Replay attack | Replay captured Zigbee frames | `c6l-ctl zigbee attack replay <file>` |
| BLE scan | Discover BLE peripherals | `c6l-ctl ble scan` |
| BLE connect | Connect to BLE device, explore GATT | `c6l-ctl ble connect <MAC>` |

### Simultaneous Zigbee + Mesh + WiFi

Because Zigbee/Thread runs on the C6L's radio, mesh runs on the C5's LoRa, and WiFi attacks run on the C5's WiFi radio — **all three can operate simultaneously**:

```bash
# Terminal 1: WiFi deauth attack (ESP32C5 WiFi radio)
monsterctl deauth

# Terminal 2: Zigbee scan (C6L 802.15.4 radio, routed through MonsterC5)
c6l-ctl zigbee scan

# Terminal 3: Meshtastic mesh (ESP32C5 LoRa radio)
monsterctl mesh start

# All independent, no conflicts
```

---

## 12. C6L Command Routing

The Unit C6L (ESP32-C6) connects to Grove OUT via I2C (LCD) and UART (commands/data).

### Wiring

```
Unit C6L           M5MonsterC5 Grove OUT
────────           ──────────────────────
VCC             →  VCC (5V)
TX              →  RX (GPIO18)
RX              →  TX (GPIO17)
SDA             →  SDA (GPIO8)
SCL             →  SCL (GPIO9)
GND             →  GND
```

### I2C LCD

The C6L has a built-in SH1107 OLED (128x64) at I2C address 0x3C. Text is sent via the `c6l_cmd` interface:

```bash
c6l_cmd LCD:1:hello          # Display "hello" on line 1
c6l_cmd LCD:2:world          # Display "world" on line 2
```

### UART Commands

C6L commands are sent over UART1 at 115200 baud. The MonsterC5 reconfigures UART1 when switching from GPS mode:

```bash
# Enable C6L passthrough:
c6l_passthrough_start

# Send specific commands:
c6l_cmd ZIGBEE_SCAN
c6l_cmd BLE_SCAN
c6l_cmd LCD:1:SCANNING

# Disable C6L passthrough:
c6l_passthrough_stop
```

From Cardputer Zero, use `c6l-ctl` or `monsterctl`:

```bash
c6l-ctl zigbee scan           # Routes through MonsterC5 automatically
c6l-ctl ble scan
c6l-ctl lcd text "OK"

# Or directly:
monsterctl c6l_cmd ZIGBEE_SCAN
```

---

## 13. Meshtastic Mesh

The ESP32C5 has a built-in 2.4 GHz radio capable of WiFi and LoRa-like mesh. The Meshtastic node runs concurrently with WiFi attacks.

### Operation

```bash
mesh_start                    # Start node on channel 6
mesh_send 1 "hello"           # Send to channel 1
mesh_status                   # Check status
mesh_stop                     # Stop node
```

From Cardputer Zero:

```bash
monsterctl mesh start
monsterctl mesh send 1 "target found"
monsterctl mesh status
monsterctl mesh stop
```

### Configuration

- Default channel: 6
- Meshtastic protocol version: compatible with any Meshtastic node
- Mesh messages use the `MESH:` prefix in the serial protocol

---

## 14. WiFi Attack Commands

All original upstream attack commands work unchanged. These are sent without a prefix — the MonsterC5 treats unprefixed lines as WiFi commands:

| Attack | Command | What it does |
|---|---|---|
| Scan | `scan_networks` | Scan all visible APs |
| Deauth | `start_deauth` | Deauth clients from selected AP |
| Evil twin | `start_evil_twin` | Clone AP, start captive portal |
| SAE overflow | `start_sae_overflow` | WPA3 SAE DoS |
| Handshake | `start_handshake` | Capture WPA/WPA2 4-way handshake |
| Sniffer | `start_sniffer` | Capture all WiFi frames |
| Blackout | `start_blackout` | Mass deauth all visible APs |
| Wardrive | `start_wardrive` | Scan APs + GPS coordinates |
| Stop | `stop` | Stop all running attacks |

### 5 GHz capability

The ESP32C5 supports dual-band WiFi (2.4 GHz + 5 GHz, 802.11ax WiFi 6). All attack modes can target 5 GHz networks, unlike ESP32 and ESP32-S3 variants which are limited to 2.4 GHz.

---

## 15. Board Detection

On boot, `board_detect_all()` probes the Grove ports:

1. **GPS detection** — Configures UART1 at 9600 baud, listens for `$G` NMEA prefix within 2 seconds
2. **C6L LCD detection** — Probes I2C address 0x3C for acknowledgment
3. **Hub status** — Sends `HUB_STATUS:GPS=OK/N/A,C6L_LCD=OK/N/A,MESH=ready` to Cardputer Zero

Run manually:

```bash
hub_status
```

---

## 16. UART Multiplexing

The ESP32C5 has only 2 hardware UARTs. UART1 is time-multiplexed between GPS and C6L:

```
┌─────────────┐     UART0 115200      ┌──────────────────┐
│  Cardputer   │◄────────────────────►│  M5MonsterC5      │
│  Zero        │  (console + mux)     │  (ESP32C5)        │
└─────────────┘                       │                   │
                                      │  UART1 (muxed)    │
                                      │  ┌─ GPS (9600)    │
                                      │  └─ C6L (115200)  │
                                      │                   │
                                      │  I2C0 → C6L LCD   │
                                      │  (0x3C, 400kHz)   │
                                      └──────────────────┘
```

### Switching rules

- Switching from GPS to C6L (or vice versa) requires stopping the current passthrough first
- The `g_periph_mode` state variable tracks: `IDLE`, `GPS`, or `C6L`
- Each passthrough start reconfigures UART1 baud rate and pins, and installs/removes the UART driver
- Passthrough stop deletes the UART driver task and returns to `IDLE`

### Priority

- GPS passthrough has higher priority in `wifi_attack.c` — `gps_passthrough_start`/`stop` commands are checked before C6L
- C6L passthrough has its own start/stop commands
- The `hub_status` command shows current peripheral mode

---

## 17. OLED Status Display (M5Stack Module Grove)

The M5Stack NFC/CC1101 module plugs into the Cardputer Zero's 14-pin expansion port and has its own Grove port for I2C devices. The SH1107 OLED (128x64) and PN532 NFC share this Grove port but cannot be used simultaneously.

### Wiring

```
SH1107 OLED       M5Stack Module Grove Port
──────────       ──────────────────────────
VCC (3.3V)    →   VCC
SDA            →   SDA (I2C data)
SCL            →   SCL (I2C clock)
GND            →   GND

Address: 0x3C (default) or 0x3D

The M5Stack module itself connects to the Cardputer Zero's 14-pin
expansion port. The Grove port is on the module, not the Cardputer.
```

### Commands

```bash
oled-ctl install          # Install luma.oled + dependencies
oled-ctl status           # Show OLED detection status
oled-ctl test             # Display test pattern
oled-ctl text "ATTACK ON" # Display text
oled-ctl text-rows "Line1" "Line2" "Line3" "Line4"
oled-ctl trail            # Show Trail navigation direction
oled-ctl overwatch        # Show Overwatch threat level
oled-ctl ip               # Show WiFi IP
oled-ctl battery          # Show battery status
oled-ctl sysinfo          # Show CPU/memory/disk
oled-ctl clock            # Show clock
oled-ctl qr "text"        # Generate QR code
oled-ctl clear            # Clear display
oled-ctl off               # Turn off display
oled-ctl daemon           # Rotating status display (clock → sysinfo → trail → overwatch)
```

### Status dashboard

The `oled-ctl daemon` command rotates through status screens every 5 seconds:
1. Clock (current time)
2. System info (memory, disk, load, temperature)
3. Trail navigation (breadcrumb direction, if running)
4. Overwatch threat level (OK/WATCH/WARN/CRIT)

### OLED + NFC time-sharing

The OLED and PN532 NFC module share the same Grove I2C port on the M5Stack module. They cannot be used simultaneously:

```bash
# OLED mode (I2C address 0x3C):
oled-ctl sysinfo          # Works

# NFC mode (I2C address 0x24):
rfid2-ctl scan            # Works, but OLED must be off

# Switch between them:
oled-ctl off               # Turn off OLED
rfid2-ctl scan             # Now NFC works
rfid2-ctl stop              # Done with NFC
oled-ctl daemon            # Back to OLED mode
```

The CC1101 Sub-GHz transceiver is on the same M5Stack module but uses SPI via the 14-pin connector, so CC1101 + OLED can work simultaneously (different buses).

The OLED can also display status from the MonsterC5 hub, C6L Zigbee scans, and mesh activity:

```bash
# Show hub status on OLED
oled-ctl text-rows "HUB" "GPS:OK" "C6L:Zigbee" "MESH:CH6"

# Show Zigbee scan results
oled-ctl text-rows "ZIGBEE" "PAN:0xABCD" "CH:15" "3 devices"

# Show mesh status
oled-ctl text-rows "MESH" "CH:6" "3 nodes" "online"
```

## 18. Pin Configuration

| Function | Pin | Peripheral | Notes |
|---|---|---|---|
| UART0 TX (console) | Default | UART_NUM_0 | USB/Serial to Cardputer Zero |
| UART0 RX (console) | Default | UART_NUM_0 | USB/Serial from Cardputer Zero |
| GPS TX → C5 RX | GPIO5 | UART_NUM_1 | Grove IN |
| GPS RX ← C5 TX | GPIO4 | UART_NUM_1 | Grove IN |
| C6L TX → C5 RX | GPIO18 | UART_NUM_1 | Grove OUT (shared with GPS) |
| C6L RX ← C5 TX | GPIO17 | UART_NUM_1 | Grove OUT (shared with GPS) |
| C6L LCD SDA | GPIO8 | I2C_NUM_0 | Grove OUT |
| C6L LCD SCL | GPIO9 | I2C_NUM_0 | Grove OUT |

> GPIO4/GPIO5 are used for GPS (Grove IN), and GPIO17/GPIO18 for C6L (Grove OUT). Both pairs connect to UART_NUM_1 — only one pair is active at a time based on `g_periph_mode`.

---

## 19. Partition Table

4MB Flash layout (`partitions.csv`):

| Name | Type | Offset | Size | Notes |
|---|---|---|---|---|
| nvs | data/nvs | 0x9000 | 16KB | Non-volatile storage |
| phy_init | data/phy | 0xD000 | 4KB | PHY calibration |
| factory | app/factory | 0x10000 | 1920KB | Main firmware |
| storage | data/spiffs | 0x1F0000 | 64KB | Captures, configs |

Current firmware binary: **~298KB** (85% of partition free).

---

## 20. Upstream vs ZERO-DAY

| Feature | Upstream (JanOS/CardputerADV) | ZERO-DAY Fork |
|---|---|---|
| WiFi deauth/evil twin | Yes | Yes |
| 5 GHz WiFi 6 support | Yes (ESP32C5) | Yes (ESP32C5) |
| WPA3 SAE overflow | Yes | Yes |
| Handshake capture | Yes | Yes |
| Sniffer/karma/beacon | Yes | Yes |
| Wardrive + GPS | Yes (onboard) | Yes (AT6558 Grove IN) |
| GPS passthrough | No | Yes (Grove IN, UART mux) |
| C6L passthrough | No | Yes (Grove OUT, I2C+UART mux) |
| Zigbee/Thread hacking | No | Yes (via C6L ESP32-C6) |
| BLE 5 hacking | No | Yes (via C6L ESP32-C6) |
| C6L command routing | No | Yes (`c6l_cmd` prefix) |
| C6L LCD control | No | Yes (I2C 0x3C) |
| Meshtastic LoRa mesh | No | Yes (native ESP32C5) |
| Serial multiplexing | Simple text | Prefix-based (GPS/C6L/MESH) |
| UART1 time-multiplexing | No | Yes (GPS 9600 ↔ C6L 115200) |
| Board auto-detection | No | Yes (GPS + C6L LCD probe) |
| OLED status display | No | Yes (M5Stack module Grove, SH1107) |
| Simultaneous WiFi+Zigbee+Mesh | N/A | Yes (3 independent radios) |
| JanOS-app TUI | Yes | No (use `monsterctl` CLI) |

---

## 21. Troubleshooting

### Firmware won't build

```bash
# Ensure ESP-IDF v5.4 is sourced:
bash -c 'source ~/esp/esp-idf/export.sh && idf.py --version'
# Should show v5.4.x

# Clean build:
bash -c 'source ~/esp/esp-idf/export.sh && idf.py fullclean && idf.py build'
```

### Can't flash via USB

```bash
# Check serial port:
ls /dev/ttyUSB* /dev/ttyACM*

# Flash manually:
esptool.py --chip esp32c5 -p /dev/ttyUSB0 -b 460800 \
    --before default_reset --after hard_reset \
    write_flash --flash_mode dio --flash_size 2MB --flash_freq 80m \
    0x2000 build/bootloader/bootloader.bin \
    0x8000 build/partition_table/partition-table.bin \
    0x10000 build/zeroday-monsterc5.bin
```

### No response from MonsterC5

```bash
# Verify serial connection:
monsterctl ping
# Expected: pong

# Check serial port:
stty -F /dev/ttyUSB0 115200 raw -echo
cat /dev/ttyUSB0  # Should see boot messages on reset
```

### GPS not detected

- Check Grove IN wiring (TX↔RX, RX↔TX)
- GPS module needs clear sky for first fix (up to 5 minutes cold start)
- Verify AT6558 is outputting NMEA: `gps_passthrough_start` then watch for `GPS:$GP` lines

### C6L not responding

- Check Grove OUT wiring (4 pins: VCC, TX, RX, GND + I2C SDA/SCL)
- Ensure `c6l_passthrough_start` is sent before C6L commands
- C6L LCD detection: run `hub_status` — should show `C6L_LCD=OK`
- Verify I2C address 0x3C is responding

### GPS and C6L conflict

GPS and C6L share UART1. Only one can be active at a time:

```bash
# Stop GPS before using C6L:
gps_passthrough_stop
c6l_passthrough_start

# Stop C6L before using GPS:
c6l_passthrough_stop
gps_passthrough_start
```

---

## 22. File Layout

```
firmware/monsterc5/
├── CMakeLists.txt                  # Top-level CMake
├── Makefile                         # Build wrapper (bash -c 'source ... && make ...')
├── README.md                        # Firmware spec
├── sdkconfig.defaults               # ESP-IDF config (esp32c5, 115200 baud, WiFi)
├── partitions.csv                   # 4MB flash partition table
├── main/
│   ├── CMakeLists.txt               # Component registration
│   ├── zeroday_monsterc5.c          # Main entry point
│   ├── serial_mux.c                 # UART0 console multiplexer
│   ├── gps_passthrough.c            # GPS passthrough (UART1 @ 9600)
│   ├── c6l_routing.c                # C6L routing (UART1 @ 115200, I2C LCD)
│   ├── mesh_node.c                  # Meshtastic node (stub)
│   ├── wifi_attack.c                # WiFi attack command dispatcher
│   ├── board_detect.c               # Hardware auto-detection
│   └── include/
│       └── zeroday_monsterc5.h      # Shared header (UART/I2C/pin defines)

deploy/firmware/
├── bootloader.bin                   # ESP-IDF bootloader (~21KB)
├── partition-table.bin              # Partition table (~3KB)
└── zeroday-monsterc5.bin            # Main firmware (~298KB)
```