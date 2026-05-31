#!/usr/bin/env python3
"""
ZERO-DAY BLE Remote API — Flipper Zero-style GATT Server

BLE GATT server that exposes characteristics for Android/iOS companion
app remote control. Uses BlueZ D-Bus API for native Linux BLE support.

GATT Service: ZERO-DAY Remote (UUID: 0000fe5e-0000-1000-8000-00805f9b34fb)
  Command RX  (fe5e0001) — Write     — receive commands from app
  Command TX  (fe5e0002) — Notify    — send responses to app
  File TX     (fe5e0003) — Notify    — stream file data to app
  File RX     (fe5e0004) — Write     — receive file data from app
  Status      (fe5e0005) — Read/Notify — device status JSON
  Screen      (fe5e0006) — Notify    — screen capture stream
"""

import dbus
import dbus.service
import dbus.mainloop.glib
from gi.repository import GLib
import json
import os
import subprocess
import base64
import struct
import logging
import threading
import time
from pathlib import Path

logging.basicConfig(level=logging.INFO, format='%(asctime)s [BLE] %(levelname)s: %(message)s')
log = logging.getLogger('zeroday-ble')

BLUEZ_SERVICE_NAME = 'org.bluez'
GATT_MANAGER_IFACE = 'org.bluez.GattManager1'
LE_ADVERTISING_MANAGER_IFACE = 'org.bluez.LEAdvertisingManager1'
GATT_SERVICE_IFACE = 'org.bluez.GattService1'
GATT_CHRC_IFACE = 'org.bluez.GattCharacteristic1'
GATT_DESC_IFACE = 'org.bluez.GattDescription1'
LE_ADVERTISEMENT_IFACE = 'org.bluez.LEAdvertisement1'
DBUS_OM_IFACE = 'org.freedesktop.DBus.ObjectManager'
DBUS_PROP_IFACE = 'org.freedesktop.DBus.Properties'

BASE_UUID = '0000xxxx-0000-1000-8000-00805f9b34fb'
SERVICE_UUID = '0000fe5e-0000-1000-8000-00805f9b34fb'
CHRC_CMD_RX_UUID = 'fe5e0001-0000-1000-8000-00805f9b34fb'
CHRC_CMD_TX_UUID = 'fe5e0002-0000-1000-8000-00805f9b34fb'
CHRC_FILE_TX_UUID = 'fe5e0003-0000-1000-8000-00805f9b34fb'
CHRC_FILE_RX_UUID = 'fe5e0004-0000-1000-8000-00805f9b34fb'
CHRC_STATUS_UUID = 'fe5e0005-0000-1000-8000-00805f9b34fb'
CHRC_SCREEN_UUID = 'fe5e0006-0000-1000-8000-00805f9b34fb'

BT_NAME = 'Cardputer-Zero'
MAX_NOTIFY_LEN = 512
MAX_WRITE_LEN = 512


class CommandHandler:
    """Process incoming BLE commands and return responses."""

    @staticmethod
    def get_status() -> dict:
        wifi_ip = CommandHandler._run('ip -4 addr show wlan0 2>/dev/null | grep -oP "inet \\K[\\d.]+" | head -1') or 'OFF'
        wifi_ssid = CommandHandler._run('iwgetid -r 2>/dev/null') or 'OFF'
        bat = CommandHandler._read('/sys/class/power_supply/bq27220/capacity', '?')
        bat_volt = CommandHandler._read('/sys/class/power_supply/bq27220/voltage_now', '?')
        if bat_volt != '?':
            try:
                bat_volt = f'{int(bat_volt)/1000000:.2f}V'
            except ValueError:
                pass
        temp = CommandHandler._read('/sys/class/thermal/thermal_zone0/temp', '?')
        if temp != '?':
            try:
                temp = f'{int(temp)/1000:.1f}C'
            except ValueError:
                pass
        load = CommandHandler._read('/proc/loadavg', '?')
        if load != '?':
            load = load.split()[0]
        mem_free = CommandHandler._run('free -m | awk \'/Mem:/{print $4}\'', '?')
        disk_free = CommandHandler._run('df -h / | awk \'/root/{print $4}\'', '?')
        hdmi = CommandHandler._read('/sys/class/drm/card0-HDMI-A-1/status', 'disconnected')
        uptime = CommandHandler._read('/proc/uptime', '0')
        if uptime != '0':
            uptime = str(int(float(uptime.split()[0])))

        return {
            'device': 'cardputer-zero',
            'hostname': os.uname().nodename,
            'wifi_ip': wifi_ip,
            'wifi_ssid': wifi_ssid,
            'battery': {'percent': bat, 'voltage': bat_volt},
            'cpu': {'temp': temp, 'load': load},
            'memory': {'free_mb': mem_free},
            'disk': {'free': disk_free},
            'hdmi': hdmi,
            'uptime_sec': uptime,
            'ble_remote': 'v1.0',
        }

    @staticmethod
    def handle(cmd: str) -> str:
        if cmd == 'ping':
            return 'pong'
        if cmd == 'status':
            return json.dumps(CommandHandler.get_status())
        if cmd == 'panic':
            CommandHandler._run('/usr/local/bin/panic', 'error:panic_not_found')
            return 'panic:executed'
        if cmd == 'stealth':
            bl_path = '/sys/class/backlight'
            bl_dirs = list(Path(bl_path).glob('*/brightness')) if Path(bl_path).exists() else []
            if bl_dirs:
                cur = bl_dirs[0].read_text().strip()
                new = '0' if cur != '0' else '1'
                bl_dirs[0].write_text(new)
                return f'backlight:{"on" if new == "1" else "off"}'
            return 'error:no_backlight'
        if cmd.startswith('wifi:'):
            action = cmd[5:]
            if action == 'on':
                CommandHandler._run('cardputer-wifi-toggle on')
                return 'wifi:on'
            if action == 'off':
                CommandHandler._run('cardputer-wifi-toggle off')
                return 'wifi:off'
            if action == 'scan':
                CommandHandler._run('iw dev wlan0 scan trigger 2>/dev/null')
                return 'wifi:scanning'
            return f'wifi:unknown_action:{action}'
        if cmd.startswith('bt:'):
            action = cmd[3:]
            if action == 'on':
                CommandHandler._run('rfkill unblock bluetooth')
                return 'bt:on'
            if action == 'off':
                CommandHandler._run('rfkill block bluetooth')
                return 'bt:off'
            return f'bt:unknown_action:{action}'
        if cmd.startswith('shell:'):
            return CommandHandler._run(cmd[6:], max_out=4096) or 'error:shell_failed'
        if cmd.startswith('file:ls:'):
            return CommandHandler._run(f'ls -la {cmd[8:]} 2>/dev/null | head -50', 'error:cannot_list')
        if cmd.startswith('file:get:'):
            filepath = cmd[9:]
            if os.path.isfile(filepath):
                with open(filepath, 'rb') as f:
                    return 'file:base64:' + base64.b64encode(f.read(MAX_NOTIFY_LEN * 4)).decode()
            return 'error:file_not_found'
        if cmd.startswith('file:put:'):
            parts = cmd[9:]
            sep = parts.index(':')
            filepath = parts[:sep]
            data = base64.b64decode(parts[sep+1:])
            with open(filepath, 'wb') as f:
                f.write(data)
            return f'file:saved:{filepath}'
        if cmd.startswith('c6l:'):
            return CommandHandler._run(f'C6L_MODE=ble c6l-ctl {cmd[4:]}', 'error:c6l_failed')
        if cmd.startswith('mesh:'):
            return CommandHandler._run(f'mesh-chat {cmd[5:]}', 'error:mesh_failed')
        if cmd == 'screen':
            return 'error:screen_capture_not_available'
        if cmd == 'reboot':
            subprocess.Popen(['shutdown', '-r', 'now'])
            return 'reboot:acknowledged'
        if cmd == 'shutdown':
            subprocess.Popen(['shutdown', '-h', 'now'])
            return 'shutdown:acknowledged'
        if cmd == 'help':
            return 'commands:ping,status,panic,stealth,wifi:on|off|scan,bt:on|off,shell:<cmd>,file:ls|get|put,c6l:<cmd>,mesh:<cmd>,screen,reboot,shutdown'
        return f'error:unknown_command:{cmd}'

    @staticmethod
    def _run(cmd: str, fallback: str = '') -> str:
        try:
            r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=10)
            return (r.stdout.strip() or r.stderr.strip())[:4096] or fallback
        except Exception:
            return fallback

    @staticmethod
    def _read(path: str, fallback: str = '?') -> str:
        try:
            return Path(path).read_text().strip()
        except Exception:
            return fallback


class Advertisement(dbus.service.Object):
    """BLE Advertisement for ZERO-DAY Remote Service."""

    PATH = '/zeroday/advertisement'

    def __init__(self, bus):
        self.path = self.PATH
        self.bus = bus
        super().__init__(bus, self.path)
        self.ad_type = 'peripheral'
        self.service_uuids = [SERVICE_UUID]
        self.local_name = BT_NAME
        self.tx_power = 0
        self.manufacturer_data = dbus.Dictionary({}, signature='qv')

    @dbus.service.method('org.bluez.LEAdvertisement1',
                          in_signature='', out_signature='a{sv}')
    def GetProperties(self):
        props = {
            'Type': dbus.String(self.ad_type),
            'ServiceUUIDs': dbus.Array(self.service_uuids, signature='s'),
            'LocalName': dbus.String(self.local_name),
            'TxPower': dbus.Int16(self.tx_power),
            'ManufacturerData': self.manufacturer_data,
        }
        return dbus.Dictionary(props, signature='sv')

    @dbus.service.method('org.bluez.LEAdvertisement1',
                          in_signature='', out_signature='')
    def Release(self):
        log.info('Advertisement released')


class Characteristic(dbus.service.Object):
    """Base GATT Characteristic."""

    def __init__(self, bus, index, uuid, flags, service_path, value=b''):
        self.path = f'{service_path}/char{index:04x}'
        self.bus = bus
        self.uuid = uuid
        self.flags = flags
        self.service_path = service_path
        self.value = list(value)
        self.notifying = False
        super().__init__(bus, self.path)

    def get_properties(self):
        return {
            'Service': dbus.ObjectPath(self.service_path),
            'UUID': dbus.String(self.uuid),
            'Flags': dbus.Array(self.flags, signature='s'),
        }

    @dbus.service.method(GATT_CHRC_IFACE, in_signature='a{sv}', out_signature='ay')
    def ReadValue(self, options):
        return dbus.Array(self.value, signature='y')

    @dbus.service.method(GATT_CHRC_IFACE, in_signature='aya{sv}', out_signature='')
    def WriteValue(self, value, options):
        self.value = list(value)
        self._on_write(bytes(value))

    @dbus.service.method(GATT_CHRC_IFACE, in_signature='', out_signature='')
    def StartNotify(self):
        self.notifying = True
        log.info('Notifications started for %s', self.uuid[:8])

    @dbus.service.method(GATT_CHRC_IFACE, in_signature='', out_signature='')
    def StopNotify(self):
        self.notifying = False
        log.info('Notifications stopped for %s', self.uuid[:8])

    def _on_write(self, data: bytes):
        pass

    def notify(self, value: bytes):
        if not self.notifying:
            return
        self.PropertiesChanged(
            GATT_CHRC_IFACE,
            {'Value': dbus.Array(list(value), signature='y')},
            [])

    @dbus.service.signal(DBUS_PROP_IFACE, signature='sa{sv}as')
    def PropertiesChanged(self, iface, changed, invalidated):
        pass


class CommandRxCharacteristic(Characteristic):
    """Command RX — receives commands from the app."""

    def __init__(self, bus, service_path, cmd_tx):
        super().__init__(bus, 0, CHRC_CMD_RX_UUID,
                         ['write', 'write-without-response'],
                         service_path)
        self.cmd_tx = cmd_tx

    def _on_write(self, data: bytes):
        try:
            cmd = data.decode('utf-8')
        except UnicodeDecodeError:
            cmd = data.decode('latin-1')
        log.info('Command RX: %s', cmd[:80])
        response = CommandHandler.handle(cmd)
        log.info('Command TX: %s', response[:80])
        try:
            self.cmd_tx.value = list(response.encode('utf-8'))
        except Exception:
            self.cmd_tx.value = list(b'error:response_encode_failed')
        self.cmd_tx.notify(response.encode('utf-8'))


class CommandTxCharacteristic(Characteristic):
    """Command TX — sends responses to the app."""

    def __init__(self, bus, service_path):
        super().__init__(bus, 1, CHRC_CMD_TX_UUID,
                         ['notify', 'read'],
                         service_path)


class FileTxCharacteristic(Characteristic):
    """File TX — stream file data to app."""

    def __init__(self, bus, service_path):
        super().__init__(bus, 2, CHRC_FILE_TX_UUID,
                         ['notify', 'read'],
                         service_path)

    def stream_file(self, filepath: str):
        try:
            with open(filepath, 'rb') as f:
                data = f.read()
            b64 = base64.b64encode(data)
            offset = 0
            while offset < len(b64):
                chunk = b64[offset:offset + MAX_NOTIFY_LEN]
                self.notify(chunk)
                offset += MAX_NOTIFY_LEN
                time.sleep(0.01)
        except Exception as e:
            self.notify(f'error:{e}'.encode())


class FileRxCharacteristic(Characteristic):
    """File RX — receive file data from app."""

    def __init__(self, bus, service_path, file_tx):
        super().__init__(bus, 3, CHRC_FILE_RX_UUID,
                         ['write', 'write-without-response'],
                         service_path)
        self.file_tx = file_tx
        self._buffer = b''

    def _on_write(self, data: bytes):
        self._buffer += data
        if len(data) < MAX_WRITE_LEN:
            try:
                text = self._buffer.decode('latin-1')
                if text.startswith('file:put:'):
                    parts = text[9:]
                    sep = parts.index(':')
                    filepath = parts[:sep]
                    filedata = base64.b64decode(parts[sep+1:])
                    with open(filepath, 'wb') as f:
                        f.write(filedata)
                    self.file_tx.notify(f'file:saved:{filepath}'.encode())
            except Exception as e:
                self.file_tx.notify(f'error:{e}'.encode())
            self._buffer = b''


class StatusCharacteristic(Characteristic):
    """Status — device status JSON, readable and notifiable."""

    def __init__(self, bus, service_path):
        super().__init__(bus, 4, CHRC_STATUS_UUID,
                         ['read', 'notify'],
                         service_path)
        self._update_status()

    def _update_status(self):
        status = CommandHandler.get_status()
        self.value = list(json.dumps(status).encode('utf-8'))

    def ReadValue(self, options):
        self._update_status()
        return dbus.Array(self.value, signature='y')

    def broadcast_status(self):
        self._update_status()
        self.notify(bytes(self.value))


class ScreenCharacteristic(Characteristic):
    """Screen — screen capture stream."""

    def __init__(self, bus, service_path):
        super().__init__(bus, 5, CHRC_SCREEN_UUID,
                         ['notify', 'read'],
                         service_path)


class Service(dbus.service.Object):
    """ZERO-DAY Remote GATT Service."""

    PATH = '/zeroday/service'

    def __init__(self, bus):
        super().__init__(bus, self.PATH)
        self.bus = bus
        self.uuid = SERVICE_UUID
        self.primary = True
        self.path = self.PATH
        self.cmd_tx = CommandTxCharacteristic(bus, self.path)
        self.file_tx = FileTxCharacteristic(bus, self.path)
        self.cmd_rx = CommandRxCharacteristic(bus, self.path, self.cmd_tx)
        self.file_rx = FileRxCharacteristic(bus, self.path, self.file_tx)
        self.status = StatusCharacteristic(bus, self.path)
        self.screen = ScreenCharacteristic(bus, self.path)
        self.characteristics = [
            self.cmd_rx, self.cmd_tx,
            self.file_tx, self.file_rx,
            self.status, self.screen,
        ]

    def get_path(self):
        return dbus.ObjectPath(self.path)

    def get_properties(self):
        return {
            'UUID': dbus.String(self.uuid),
            'Primary': dbus.Boolean(self.primary),
            'Characteristics': dbus.Array(
                [dbus.ObjectPath(c.path) for c in self.characteristics],
                signature='o'),
        }


class Application(dbus.service.Object):
    """ZERO-DAY BLE Remote Application — registers all GATT services."""

    def __init__(self, bus):
        super().__init__(bus, '/')
        self.bus = bus
        self.service = Service(bus)
        self.advertisement = Advertisement(bus)

    @dbus.service.method(DBUS_OM_IFACE, in_signature='', out_signature='a{oa{sa{sv}}}')
    def GetManagedObjects(self):
        objects = {}
        objects[self.service.path] = {
            GATT_SERVICE_IFACE: self.service.get_properties(),
        }
        for c in self.service.characteristics:
            objects[c.path] = {
                GATT_CHRC_IFACE: c.get_properties(),
            }
        objects[self.advertisement.path] = {
            LE_ADVERTISEMENT_IFACE: {
                'Type': dbus.String('peripheral'),
                'ServiceUUIDs': dbus.Array([SERVICE_UUID], signature='s'),
                'LocalName': dbus.String(BT_NAME),
                'TxPower': dbus.Int16(0),
            }
        }
        return dbus.Dictionary(objects, signature='oa{sa{sv}}')


def register_app(bus, app):
    """Register GATT application with BlueZ."""
    gatt_manager = bus.get_object(BLUEZ_SERVICE_NAME, '/org/bluez')
    manager = dbus.Interface(gatt_manager, GATT_MANAGER_IFACE)
    manager.RegisterApplication(app.get_path(), {},
                                 reply_handler=lambda: log.info('GATT app registered'),
                                 error_handler=lambda e: log.error('GATT register failed: %s', e))


def register_advertisement(bus, ad):
    """Register LE advertisement with BlueZ."""
    ad_manager = bus.get_object(BLUEZ_SERVICE_NAME, '/org/bluez')
    manager = dbus.Interface(ad_manager, LE_ADVERTISING_MANAGER_IFACE)
    manager.RegisterAdvertisement(ad.path, {},
                                   reply_handler=lambda: log.info('Advertisement registered'),
                                   error_handler=lambda e: log.error('Ad register failed: %s', e))


def status_broadcaster(status_chrc):
    """Periodically broadcast device status via BLE notification."""
    while True:
        time.sleep(10)
        try:
            status_chrc.broadcast_status()
        except Exception as e:
            log.debug('Status broadcast error: %s', e)


def main():
    dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
    bus = dbus.SystemBus()

    log.info('Starting ZERO-DAY BLE Remote API')
    log.info('Device name: %s', BT_NAME)
    log.info('Service UUID: %s', SERVICE_UUID)
    log.info('')
    log.info('GATT Characteristics:')
    log.info('  Command RX  (fe5e0001) — Write — receive commands from app')
    log.info('  Command TX  (fe5e0002) — Notify — send responses to app')
    log.info('  File TX     (fe5e0003) — Notify — stream file data to app')
    log.info('  File RX     (fe5e0004) — Write — receive file data from app')
    log.info('  Status      (fe5e0005) — Read/Notify — device status JSON')
    log.info('  Screen      (fe5e0006) — Notify — screen capture stream')
    log.info('')

    app = Application(bus)

    register_app(bus, app)
    register_advertisement(bus, app.advertisement)

    log.info('GATT server registered. Advertising as "%s"...', BT_NAME)

    status_thread = threading.Thread(
        target=status_broadcaster,
        args=(app.service.status,),
        daemon=True,
    )
    status_thread.start()

    loop = GLib.MainLoop()
    try:
        loop.run()
    except KeyboardInterrupt:
        log.info('Shutting down BLE Remote API')
        loop.quit()


if __name__ == '__main__':
    main()