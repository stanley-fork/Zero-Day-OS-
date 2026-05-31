#!/bin/bash -e
set -euo pipefail
# stage4/03-bluetooth-tools/01-run.sh — Install Bluetooth tools and scripts

BIN="${ROOTFS_DIR}/usr/local/bin"

# Install Bluetooth wrapper scripts
for script in bt-scan bt-deep bt-attack ble-gatt ble-spam bettercap; do
    if [ -f "${PROJECT_ROOT}/scripts/bluetooth/${script}" ]; then
        cp "${PROJECT_ROOT}/scripts/bluetooth/${script}" "${BIN}/${script}"
        chmod +x "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    else
        echo "[zeroday] WARNING: Missing script: ${script}"
    fi
done

# bettercap — Swiss-army MITM framework (Kali armhf, best-effort)
on_chroot << EOF
apt-get -y -t kali-rolling install --no-install-recommends bettercap 2>/dev/null || echo "[zeroday] bettercap not available from Kali armhf — install manually if needed"
EOF

# BLE Remote API — Flipper Zero-style GATT server for Android/iOS companion
BLE_REMOTE_DIR="${ROOTFS_DIR}/opt/cardputer/ble-remote"
mkdir -p "${BLE_REMOTE_DIR}"

if [ -f "${PROJECT_ROOT}/scripts/hardware/ble-remote/gatt_server.py" ]; then
    cp "${PROJECT_ROOT}/scripts/hardware/ble-remote/gatt_server.py" "${BLE_REMOTE_DIR}/gatt_server.py"
    chmod +x "${BLE_REMOTE_DIR}/gatt_server.py"
    echo "[zeroday] Installed: ble-remote/gatt_server.py"
else
    echo "[zeroday] WARNING: Missing ble-remote/gatt_server.py"
fi

if [ -f "${PROJECT_ROOT}/scripts/hardware/zeroday-ble-remote" ]; then
    cp "${PROJECT_ROOT}/scripts/hardware/zeroday-ble-remote" "${BIN}/zeroday-ble-remote"
    chmod +x "${BIN}/zeroday-ble-remote"
    echo "[zeroday] Installed: zeroday-ble-remote"
else
    echo "[zeroday] WARNING: Missing zeroday-ble-remote"
fi

# Install systemd service
if [ -f "${PROJECT_ROOT}/configs/systemd/zeroday-ble-remote.service" ]; then
    cp "${PROJECT_ROOT}/configs/systemd/zeroday-ble-remote.service" \
       "${ROOTFS_DIR}/etc/systemd/system/zeroday-ble-remote.service"
    on_chroot << EOF
systemctl enable zeroday-ble-remote.service 2>/dev/null || echo "[zeroday] BLE remote service will need manual enable"
EOF
    echo "[zeroday] Installed: zeroday-ble-remote.service"
fi

# Python dependencies for GATT server (dbus + gobject)
on_chroot << EOF
apt-get -y install --no-install-recommends python3-dbus python3-gi python3-gi-cairo 2>/dev/null || echo "[zeroday] Python DBus/GI packages from pi-os repo"
EOF

echo "[zeroday] Bluetooth tools + BLE Remote API installed."