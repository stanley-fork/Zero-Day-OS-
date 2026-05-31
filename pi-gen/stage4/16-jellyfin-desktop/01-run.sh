#!/bin/bash -e
set -euo pipefail
# Build jellyfin-media-player v1.12.0 from source for arm64
# This includes building libmpv from source (required for the embedded mpv player)
# All build deps are installed then purged after to keep image small
#
# Build time estimate: ~30-60 min on Pi (Qt5 WebEngine is the slow part)
# Resulting image size increase: ~150-200MB (Qt5 WebEngine runtime)

echo "[zeroday] Building jellyfin-media-player v1.12.0 from source (arm64)..."

# ── Install build dependencies ──
echo "[zeroday] Installing build dependencies..."
on_chroot << BUILDEPS
apt-get -y update
apt-get -y install --no-install-recommends \
    build-essential autoconf automake libtool pkg-config cmake ninja-build \
    meson nasm yasm git curl wget unzip ca-certificates \
    qtbase5-dev qtwebengine5-dev qtquickcontrols2-5-dev \
    libqt5x11extras5-dev libqt5svg5-dev qtbase5-private-dev \
    qml-module-qtwebengine qml-module-qtwebchannel qml-module-qtquick-controls \
    qml-module-qtquick-controls2 \
    libharfbuzz-dev libfreetype-dev libfontconfig1-dev \
    libx11-dev libxrandr-dev libxss-dev libxinerama-dev \
    libvdpau-dev libva-dev libegl-dev libgl1-mesa-dev \
    mesa-common-dev libdrm-dev libgbm-dev \
    libasound2-dev libpulse-dev libuchardet-dev \
    zlib1g-dev libfribidi-dev libgnutls28-dev \
    libsdl2-dev libcec-dev \
    libavcodec-dev libavformat-dev libavutil-dev libswscale-dev libswresample-dev \
    libavfilter-dev libpostproc-dev \
    libluajit-5.1-dev libv4l-dev libdvdnav-dev libdvdread-dev libbluray-dev \
    libarchive-dev libass-dev libmpv-dev \
    python3-pip
BUILDEPS

echo "[zeroday] Build dependencies installed."

# ── Upgrade meson for libplacebo (requires >=1.3.0, bookworm has 1.0.1) ──
echo "[zeroday] Upgrading meson via pip..."
on_chroot << MESONFIX
set -eu
pip3 install --break-system-packages --upgrade 'meson>=1.3.0'
meson --version
MESONFIX

echo "[zeroday] meson upgraded."

# ── Upgrade wayland-protocols for mpv (needs color-manager-v1, bookworm has 1.31) ──
echo "[zeroday] Upgrading wayland-protocols..."
on_chroot << 'WLPROTO'
set -eu
wget -q -O /tmp/wp.tar.xz "https://gitlab.freedesktop.org/wayland/wayland-protocols/-/releases/1.36/downloads/wayland-protocols-1.36.tar.xz"
tar xf /tmp/wp.tar.xz -C /tmp
cd /tmp/wayland-protocols-1.36
mkdir -p build && cd build
meson setup --prefix=/usr ..
ninja
ninja install
rm -rf /tmp/wp.tar.xz /tmp/wayland-protocols-1.36
WLPROTO

echo "[zeroday] wayland-protocols upgraded."

# ── Build libmpv from source ──
echo "[zeroday] Building libmpv..."
on_chroot << MPVBUILD
set -eu

mkdir -p /tmp/jmp-build
cd /tmp/jmp-build

git clone https://github.com/mpv-player/mpv-build.git
cd mpv-build

echo -Dlibmpv=true > mpv_options
echo -Dpipewire=disabled >> mpv_options

./update
./rebuild -j\$(nproc)
./install

ldconfig

MPVBUILD

echo "[zeroday] libmpv built and installed."

# ── Build jellyfin-media-player ──
echo "[zeroday] Building jellyfin-media-player v1.12.0..."
on_chroot << JMPBUILD
set -eu

cd /tmp/jmp-build

wget -q -O jmp.tar.gz "https://github.com/jellyfin-archive/jellyfin-desktop-qt/archive/refs/tags/v1.12.0.tar.gz"
mkdir -p jellyfin-media-player
tar xzf jmp.tar.gz -C jellyfin-media-player --strip-components=1
rm -f jmp.tar.gz

cd jellyfin-media-player
mkdir -p build
cd build

cmake \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=/usr/local \
    -G Ninja \
    ..

ninja -j\$(nproc)
ninja install

# Verify installation
which jellyfinmediaplayer 2>/dev/null && echo "[zeroday] jellyfinmediaplayer installed OK" || echo "[zeroday] WARNING: jellyfinmediaplayer binary not found"
ls -la /usr/local/bin/jellyfin* 2>/dev/null || echo "[zeroday] Checking install paths..."

JMPBUILD

echo "[zeroday] jellyfin-media-player built and installed."

# ── Create desktop entry and config ──
install -m 755 -d "${ROOTFS_DIR}/usr/share/applications"

cat > "${ROOTFS_DIR}/usr/share/applications/jellyfin-mediaplayer.desktop" << 'DESKEOF'
[Desktop Entry]
Name=Jellyfin Media Player
Comment=Jellyfin desktop media player
Exec=jellyfinmediaplayer
Icon=jellyfin
Type=Application
Categories=AudioVideo;Video;Player;TV;
Keywords=jellyfin;media;player;streaming;
StartupNotify=true
DESKEOF

install -m 755 -d "${ROOTFS_DIR}/etc/xdg/jellyfinmediaplayer"

cat > "${ROOTFS_DIR}/etc/xdg/jellyfinmediaplayer/mpv.conf" << 'MPVCONF'
vo=gpu
gpu-context=wayland
hwdec=auto
profile=gpu-hq
scale=ewa_lanczossharp
volume=80
audio-device=auto
video-sync=display-resample
interpolation=no
blend-subtitles=yes
sub-font-size=42
sub-border-size=3
cache=yes
demuxer-max-bytes=50MiB
demuxer-max-back-bytes=25MiB
MPVCONF

cat > "${ROOTFS_DIR}/etc/xdg/jellyfinmediaplayer/jellyfinmediaplayer.conf" << 'JMPCONF'
[general]
discover_mode=true

[mpv]
hwdec=auto
fs=yes
JMPCONF

# ── Clean up build artifacts to save image space ──
echo "[zeroday] Cleaning build artifacts..."
on_chroot << CLEANUP
set -eu

# Remove build directory
rm -rf /tmp/jmp-build

# Remove build-only packages (keep runtime deps)
apt-get -y purge \
    build-essential autoconf automake libtool meson nasm yasm cmake ninja-build python3-pip \
    qtbase5-dev qtwebengine5-dev qtquickcontrols2-5-dev \
    libqt5x11extras5-dev libqt5svg5-dev qtbase5-private-dev \
    libharfbuzz-dev libfreetype-dev libfontconfig1-dev \
    libx11-dev libxrandr-dev libxss-dev libxinerama-dev \
    libvdpau-dev libva-dev libegl-dev libgl1-mesa-dev \
    mesa-common-dev libdrm-dev libgbm-dev \
    libasound2-dev libpulse-dev libuchardet-dev \
    zlib1g-dev libfribidi-dev libgnutls28-dev \
    libsdl2-dev libcec-dev \
    libavcodec-dev libavformat-dev libavutil-dev libswscale-dev libswresample-dev \
    libavfilter-dev libpostproc-dev \
    libluajit-5.1-dev libv4l-dev libdvdnav-dev libdvdread-dev libbluray-dev \
    libarchive-dev libass-dev libmpv-dev \
    2>/dev/null || true

# Autoremove orphaned deps
apt-get -y autoremove --purge
apt-get -y clean

# Verify jellyfin-media-player survived cleanup
which jellyfinmediaplayer 2>/dev/null && echo "[zeroday] jellyfinmediaplayer survived cleanup" || echo "[zeroday] WARNING: jellyfinmediaplayer removed by cleanup!"

CLEANUP

echo "[zeroday] jellyfin-media-player v1.12.0 build complete."