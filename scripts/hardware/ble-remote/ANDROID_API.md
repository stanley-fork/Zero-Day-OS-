# ZERO-DAY Remote — Android Companion App API Reference

Flipper Zero-style BLE companion app for Cardputer Zero.

## BLE Connection

```kotlin
val SERVICE_UUID = "0000fe5e-0000-1000-8000-00805f9b34fb"

// Characteristics
val CHRC_CMD_RX   = "fe5e0001-0000-1000-8000-00805f9b34fb" // Write
val CHRC_CMD_TX   = "fe5e0002-0000-1000-8000-00805f9b34fb" // Notify
val CHRC_FILE_TX  = "fe5e0003-0000-1000-8000-00805f9b34fb" // Notify
val CHRC_FILE_RX  = "fe5e0004-0000-1000-8000-00805f9b34fb" // Write
val CHRC_STATUS   = "fe5e0005-0000-1000-8000-00805f9b34fb" // Read + Notify
val CHRC_SCREEN   = "fe5e0006-0000-1000-8000-00805f9b34fb" // Notify
```

## Connect

```kotlin
// Scan for "Cardputer-Zero" or filter by SERVICE_UUID
val filter = ScanFilter.Builder()
    .setServiceUuid(ParcelUuid.fromString(SERVICE_UUID))
    .build()

bluetoothAdapter.bluetoothLeScanner.startScan(listOf(filter), settings, callback)

// On device found, connect:
val gatt = device.connectGatt(context, false, gattCallback)
```

## Command Protocol

Write UTF-8 command to `CHRC_CMD_RX`, receive response via `CHRC_CMD_TX` notification:

```kotlin
fun sendCommand(gatt: BluetoothGatt, cmd: String) {
    val service = gatt.getService(UUID.fromString(SERVICE_UUID))
    val cmdRx = service.getCharacteristic(UUID.fromString(CHRC_CMD_RX))
    cmdRx.value = cmd.toByteArray(Charsets.UTF_8)
    cmdRx.writeType = BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
    gatt.writeCharacteristic(cmdRx)
}

// Enable notifications on CHRC_CMD_TX first:
fun enableNotifications(gatt: BluetoothGatt) {
    val service = gatt.getService(UUID.fromString(SERVICE_UUID))
    val cmdTx = service.getCharacteristic(UUID.fromString(CHRC_CMD_TX))
    gatt.setCharacteristicNotification(cmdTx, true)

    val cccd = cmdTx.getDescriptor(UUID.fromString(CCCD_UUID))
    cccd.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
    gatt.writeDescriptor(cccd)
}

// Receive response in onCharacteristicChanged:
override fun onCharacteristicChanged(gatt: BluetoothGatt, chrc: BluetoothGattCharacteristic) {
    val response = String(chrc.value, Charsets.UTF_8)
    // Handle response...
}
```

## Commands

| Command | Response | Description |
|---------|----------|-------------|
| `ping` | `pong` | Connection test |
| `status` | JSON object | Device status dashboard |
| `panic` | `panic:executed` | Kill offensive processes |
| `stealth` | `backlight:on\|off` | Toggle LCD backlight |
| `wifi:on` | `wifi:on` | Enable WiFi |
| `wifi:off` | `wifi:off` | Disable WiFi |
| `wifi:scan` | `wifi:scanning` | Start WiFi scan |
| `bt:on` | `bt:on` | Enable Bluetooth |
| `bt:off` | `bt:off` | Disable Bluetooth |
| `shell:<cmd>` | Command output | Execute shell command |
| `file:ls:<path>` | Directory listing | List files |
| `file:get:<path>` | `file:base64:<data>` | Download file |
| `file:put:<path>:<b64>` | `file:saved:<path>` | Upload file |
| `c6l:<cmd>` | C6L response | C6L controller command |
| `mesh:<cmd>` | Mesh response | Meshtastic relay |
| `screen` | Base64 PNG | Capture screen |
| `reboot` | `reboot:acknowledged` | Reboot device |
| `shutdown` | `shutdown:acknowledged` | Power off |
| `help` | Command list | List all commands |

## Status JSON Schema

```json
{
  "device": "cardputer-zero",
  "hostname": "cardputer",
  "wifi_ip": "192.168.1.42",
  "wifi_ssid": "OFF",
  "battery": {"percent": "85", "voltage": "3.82V"},
  "cpu": {"temp": "42.3C", "load": "0.45"},
  "memory": {"free_mb": "384"},
  "disk": {"free": "4.2G"},
  "hdmi": "connected",
  "uptime_sec": "86400",
  "ble_remote": "v1.0"
}
```

Read from `CHRC_STATUS` or subscribe to `CHRC_STATUS` notifications (broadcast every 10s).

## File Transfer

### Download (device → app)
1. Write `file:get:/path/to/file` to `CHRC_CMD_RX`
2. Receive `file:base64:<data>` via `CHRC_CMD_TX` notification
3. Decode base64 payload

### Upload (app → device)
1. Write `file:put:/path/filename:<base64>` to `CHRC_FILE_RX`
2. Receive `file:saved:<path>` via `CHRC_FILE_TX` notification

Large file transfers use chunked notifications (512-byte MTU).

## App Screens

1. **Dashboard** — Live status via `CHRC_STATUS` notifications (battery, WiFi, CPU, disk)
2. **Terminal** — Command input via `CHRC_CMD_RX`, output via `CHRC_CMD_TX`
3. **Files** — Browse `/opt/cardputer/loot/`, download/upload via `CHRC_FILE_RX/TX`
4. **Quick Actions** — Panic, stealth, WiFi/BT toggle, reboot
5. **C6L Control** — BLE scan, Zigbee scan, GPS pass-through via `c6l:` commands
6. **Mesh Chat** — Relay messages via `mesh:send:<msg>` commands