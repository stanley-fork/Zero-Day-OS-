#!/bin/bash -e
set -euo pipefail
# stage4/15-ragnar/01-run.sh — Install Ragnar controller + Python deps

BIN="${ROOTFS_DIR}/usr/local/bin"
RAGNAR_DIR="${ROOTFS_DIR}/opt/cardputer/ragnar"

# Install ragnar-ctl controller script
if [ -f "${PROJECT_ROOT}/scripts/network/ragnar-ctl" ]; then
    cp "${PROJECT_ROOT}/scripts/network/ragnar-ctl" "${BIN}/ragnar-ctl"
    chmod +x "${BIN}/ragnar-ctl"
    echo "[zeroday] Installed: ragnar-ctl"
fi

# Install install script (for manual install later)
if [ -f "${PROJECT_ROOT}/scripts/network/install_ragnar_port.sh" ]; then
    cp "${PROJECT_ROOT}/scripts/network/install_ragnar_port.sh" "${BIN}/install_ragnar_port.sh"
    chmod +x "${BIN}/install_ragnar_port.sh"
    echo "[zeroday] Installed: install_ragnar_port.sh"
fi

# Install systemd service
if [ -f "${PROJECT_ROOT}/configs/systemd/ragnar.service" ]; then
    cp "${PROJECT_ROOT}/configs/systemd/ragnar.service" "${ROOTFS_DIR}/etc/systemd/system/ragnar.service"
    echo "[zeroday] Installed: ragnar.service (not enabled by default)"
fi

# Pre-install Python dependencies for Ragnar (lightweight subset)
on_chroot << EOF
pip3 install --no-cache-dir --break-system-packages \
    flask>=3.0.0 \
    flask-socketio>=5.3.0 \
    flask-cors>=4.0.0 \
    python-nmap>=0.7.0 \
    netifaces>=0.11.0 \
    psutil>=5.9.0 \
    ping3>=4.0.0 \
    get-mac>=0.9.0 \
    paramiko>=3.0.0 \
    sqlalchemy>=1.4.0 \
    rich>=13.0.0 \
    cryptography>=41.0.0 2>/dev/null || echo "[zeroday] Some Ragnar deps failed — will be installed at runtime"
EOF

# Create data directories
mkdir -p "${RAGNAR_DIR}/data/intelligence" "${RAGNAR_DIR}/data/logs" "${RAGNAR_DIR}/data/loot"

# Mark ragnar service as disabled by default (user must install Ragnar first)
on_chroot << EOF
systemctl disable ragnar.service 2>/dev/null || true
EOF

echo "[zeroday] Ragnar port installed (controller + deps). Run 'ragnar-ctl install' to clone the repo."