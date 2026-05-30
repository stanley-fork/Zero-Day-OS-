#!/bin/bash
set -euo pipefail
# /opt/cardputer/scripts/install_ragnar_port.sh
# Install Ragnar port for ZERO-DAY OS on M5Stack Cardputer Zero
# Adapted from Raspyjack's vendored Ragnar port

RAGNAR_DIR="/opt/cardputer/ragnar"
RAGNAR_VENV="${RAGNAR_DIR}/venv"
RAGNAR_PORT="${RAGNAR_PORT:-8091}"
RAGNAR_USER="cardputer"

echo "=========================================="
echo "  ZERO-DAY OS — Ragnar Port Installer"
echo "=========================================="
echo ""

# Check if already installed
if [ -d "$RAGNAR_DIR" ] && [ -d "$RAGNAR_VENV" ]; then
    echo "[*] Ragnar already installed at $RAGNAR_DIR"
    echo "[*] To update, run: ragnar-ctl update"
    echo "[*] To reinstall, remove $RAGNAR_DIR first"
    exit 0
fi

# ── System dependencies ──
echo "[*] Installing system dependencies..."
apt-get update -qq
apt-get install -y -qq \
    python3 python3-pip python3-venv \
    nmap tcpdump arp-scan \
    nikto whatweb \
    hostapd dnsmasq \
    bluez bluez-tools \
    rfkill iproute2 \
    sqlite3 \
    git wget curl \
    libffi-dev libssl-dev libjpeg-dev zlib1g-dev \
    build-essential 2>&1 | tail -5

echo "[+] System dependencies installed"

# ── Clone Ragnar ──
echo "[*] Cloning Ragnar repository..."
if [ ! -d "$RAGNAR_DIR" ]; then
    git clone --depth 1 https://github.com/PierreGode/Ragnar.git "$RAGNAR_DIR" 2>&1 | tail -3
fi
echo "[+] Ragnar cloned to $RAGNAR_DIR"

# ── Create virtual environment ──
echo "[*] Creating Python virtual environment..."
python3 -m venv "$RAGNAR_VENV" --system-site-packages
source "$RAGNAR_VENV/bin/activate"

# ── Install core Python dependencies (lightweight for 512MB RAM) ──
echo "[*] Installing Python dependencies (core only)..."
pip install --no-cache-dir --upgrade pip 2>&1 | tail -2
pip install --no-cache-dir \
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
    cryptography>=41.0.0 \
    pyserial>=3.5 2>&1 | tail -5

echo "[+] Core dependencies installed"

# ── Skip heavy optional deps (not suitable for 512MB RAM) ──
echo "[*] Skipping heavy dependencies (numpy, pandas, openai, Pillow) — not needed for headless mode"
echo "[*] Skip: RPi.GPIO, spidev, pisugar, luma — Pi-specific, not needed on Cardputer"

# ── Create data directories ──
echo "[*] Creating data directories..."
mkdir -p "${RAGNAR_DIR}/data/intelligence"
mkdir -p "${RAGNAR_DIR}/data/logs"
mkdir -p "${RAGNAR_DIR}/data/loot"
mkdir -p /opt/cardputer/loot/general

# ── Initialize data files ──
if [ -f "${RAGNAR_DIR}/scripts/init_data_files.sh" ]; then
    echo "[*] Initializing data files..."
    cd "$RAGNAR_DIR"
    bash scripts/init_data_files.sh 2>&1 | tail -3 || true
fi

# ── Configure for headless mode ──
echo "[*] Configuring headless mode for Cardputer Zero..."

# Set the port
echo "RAGNAR_PORT=$RAGNAR_PORT" > "${RAGNAR_DIR}/.env"

# Disable EPD display and Pi-specific features
cat > "${RAGNAR_DIR}/config/cardputer.env" << 'EOF'
# ZERO-DAY OS Ragnar configuration
# Headless mode — no e-paper display, no PiSugar, no EPD
RAGNAR_HEADLESS=1
RAGNAR_NO_EPD=1
RAGNAR_NO_PISUGAR=1
RAGNAR_NO_GPIO=1
RAGNAR_NO_LED_MATRIX=1
RAGNAR_PORT=8091
RAGNAR_SEMAPHORE_LIMIT=1
RAGNAR_MAX_MEMORY_MB=384
EOF

# ── Install systemd service ──
echo "[*] Installing systemd service..."
cat > /etc/systemd/system/ragnar.service << SVEOF
[Unit]
Description=Ragnar Autonomous Network Scanner (ZERO-DAY OS)
After=network-online.target zeroday-boot.service
Wants=network-online.target

[Service]
Type=simple
User=root
WorkingDirectory=${RAGNAR_DIR}
Environment=RAGNAR_PORT=${RAGNAR_PORT}
Environment=RAGNAR_HEADLESS=1
Environment=RAGNAR_NO_EPD=1
Environment=RAGNAR_NO_PISUGAR=1
Environment=RAGNAR_NO_GPIO=1
ExecStartPre=/bin/sleep 5
ExecStart=${RAGNAR_VENV}/bin/python3 ${RAGNAR_DIR}/headlessRagnar.py
Restart=on-failure
RestartSec=10
StandardOutput=append:/opt/cardputer/loot/general/ragnar.log
StandardError=append:/opt/cardputer/loot/general/ragnar_error.log

[Install]
WantedBy=multi-user.target
SVEOF

systemctl daemon-reload
echo "[+] systemd service installed (ragnar.service)"

# ── Create shortcut ──
if [ ! -L /usr/local/bin/ragnar-ctl ]; then
    ln -sf /opt/cardputer/scripts/network/ragnar-ctl /usr/local/bin/ragnar-ctl 2>/dev/null || true
fi

# ── Summary ──
LOCAL_IP=$(ip -4 route get 1.1.1.1 2>/dev/null | grep -oP 'src \K\S+' | head -1 || echo "localhost")
echo ""
echo "=========================================="
echo "  Ragnar Port Installation Complete!"
echo "=========================================="
echo ""
echo "  Dashboard:  http://${LOCAL_IP}:${RAGNAR_PORT}"
echo "  Service:    systemctl start ragnar"
echo "  Control:    ragnar-ctl start|stop|status|scan|vuln"
echo "  Logs:        ragnar-ctl logs"
echo "  Config:      ${RAGNAR_DIR}/config/cardputer.env"
echo ""
echo "  NOTE: Ragnar runs headless (no e-paper display)"
echo "  Access the web dashboard from a browser on your network"
echo ""
echo "  Start manually:  ragnar-ctl start"
echo "  Enable on boot:  systemctl enable ragnar"
echo ""