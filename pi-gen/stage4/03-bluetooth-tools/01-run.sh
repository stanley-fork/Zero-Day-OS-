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

echo "[zeroday] Bluetooth tools installed."