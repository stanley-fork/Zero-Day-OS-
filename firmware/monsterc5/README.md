# ZERO-DAY OS — M5MonsterC5 Custom Firmware

Forked from [C5Lab/M5MonsterC5-CardputerADV](https://github.com/C5Lab/M5MonsterC5-CardputerADV)

## What This Is

Custom ESP32C5 firmware for the M5MonsterC5 that acts as a **middle-manager hub** —
retaining all upstream WiFi attack capabilities while adding GPS passthrough,
C6L routing, and native Meshtastic.

## Hardware Topology

```
Cardputer Zero (aarch64, main OS)
  └── USB/UART ──→ M5MonsterC5 (ESP32C5, middle manager)
                      ├── Grove IN  ← GPS Module v1.1 (AT6558 GNSS, UART 9600)
                      └── Grove OUT → Unit C6L (ESP32-C6, Zigbee/BLE/LCD, I2C+UART)
```

## Firmware Architecture

### Core (from upstream M5MonsterC5-CardputerADV)
- WiFi attack engine (deauth, evil twin, WPA3 SAE overflow, handshake capture)
- Sniffer, karma, beacon spam, blackout, rogue AP
- Wardrive with GPS tagging
- SD card storage for captures
- Serial command interface (115200 baud, line-based text protocol)

### Additions (ZERO-DAY fork)

#### 1. GPS Passthrough (Grove IN, UART)
- AT6558 GNSS on Grove UART at 9600 baud
- NMEA sentences forwarded to Cardputer Zero via multiplexed serial
- Command: `gps_passthrough_start` / `gps_passthrough_stop`
- Wardrive GPS data uses the same AT6558 (no external GPS needed)
- Protocol: GPS NMEA lines prefixed with `GPS:` in serial output

#### 2. C6L Routing (Grove OUT, I2C + UART)
- ESP32-C6 on Grove OUT port (I2C for LCD, UART for command/data)
- Commands forwarded from Cardputer Zero via serial multiplexing
- Command prefix: `C6L:` — e.g., `C6L:ZIGBEE_SCAN`, `C6L:BLE_SCAN`
- Responses prefixed with `C6L:` back to Cardputer Zero
- I2C LCD text: `C6L:LCD:1:text` → forwarded to C6L LCD via I2C
- Command: `c6l_passthrough_start` / `c6l_passthrough_stop`
- Quick command: `c6l_cmd <command>` → sends to C6L, returns response

#### 3. Meshtastic LoRa Mesh (native ESP32C5 radio)
- ESP32C5 has built-in 2.4 GHz radio (WiFi + LoRa-like)
- Meshtastic node runs concurrently with WiFi attacks
- Commands: `mesh_start`, `mesh_stop`, `mesh_send <dest> <msg>`, `mesh_status`, `mesh_config`
- LoRa mesh for off-grid C2 communication
- Compatible with any Meshtastic node in range

### Serial Protocol Multiplexing

All communication with Cardputer Zero goes over a single USB/UART connection
at 115200 baud. The firmware multiplexes different data streams using line prefixes:

```
Cardputer Zero ──→ MonsterC5          MonsterC5 ──→ Cardputer Zero
──────────────                          ──────────
scan\r\n            →  WiFi scan        scan results (line-delimited)
deauth\r\n          →  Start deauth     [attack output]
gps_passthrough_start → GPS passthrough  GPS:$GPGGA...
c6l_cmd ZIGBEE_SCAN → Forward to C6L    C6L:[zigbee scan results]
mesh_start\r\n      →  Start mesh       [mesh status]
mesh_send 1 hello   →  Mesh TX          [mesh ack]
ping\r\n            →  Ping              pong\r\n
status\r\n         →  Status            [status info]
```

Line format:
- `GPS:` prefix = NMEA data from AT6558
- `C6L:` prefix = data from/to C6L
- `MESH:` prefix = Meshtastic mesh data
- No prefix = MonsterC5 WiFi attack output (upstream protocol)

### Grove Port Configuration

| Port      | Device          | Protocol | Pins                    | Baud    |
|-----------|-----------------|----------|-------------------------|---------|
| Grove IN  | GPS Module v1.1 | UART     | TX/RX + VCC/GND        | 9600    |
| Grove OUT | Unit C6L        | I2C+UART | SDA/SCL + TX/RX + VCC/GND | 115200 |

ESP32C5 has two hardware UARTs:
- UART0: USB/UART to Cardputer Zero (console + multiplexed data)
- UART1: Grove IN (GPS, 9600 baud)
- UART2: Grove OUT (C6L data, 115200 baud)
- I2C0: Grove OUT (C6L LCD control, 0x3C)

## Building

```bash
# Clone the fork
git clone https://github.com/jayis1/M5MonsterC5-zeroday.git
cd M5MonsterC5-zeroday

# Install ESP-IDF (esp32c5 target)
# See: https://docs.espressif.com/projects/esp-idf/en/latest/esp32c5/

# Build
idf.py set-target esp32c5
idf.py build

# Flash
esptool.py --chip esp32c5 --port /dev/ttyUSB0 --baud 460800 \
    write_flash -z 0x0 build/zeroday_monsterc5.bin

# Or use: monsterctl flash local
```

## Upstream vs ZERO-DAY

| Feature              | Upstream (JanOS) | ZERO-DAY Fork |
|----------------------|------------------|---------------|
| WiFi deauth/evil twin| Yes              | Yes           |
| WPA3 SAE overflow    | Yes              | Yes           |
| Handshake capture    | Yes              | Yes           |
| Sniffer/karma/beacon | Yes              | Yes           |
| Wardrive + GPS       | Yes (onboard)    | Yes (AT6558 Grove) |
| GPS passthrough      | No               | Yes (Grove IN)|
| C6L passthrough      | No               | Yes (Grove OUT)|
| C6L command routing  | No               | Yes           |
| Meshtastic LoRa mesh | No               | Yes (native)  |
| Serial multiplexing  | Simple text      | Prefix-based  |
| JanOS-app TUI        | Yes              | No (use monsterctl) |

## File Layout

```
firmware/monsterc5/
├── README.md              # This file
├── zeroday_monsterc5.bin  # Pre-built binary (place here after build)
└── upstream/              # Upstream JanOS firmware (for flash upstream)
    └── janos_esp32c5.bin
```