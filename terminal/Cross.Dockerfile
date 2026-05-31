FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:latest

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        libwayland-dev:arm64 \
        libdrm-dev:arm64 \
        libgbm-dev:arm64 \
        libegl-dev:arm64 \
        libgles-dev:arm64 \
        libinput-dev:arm64 \
        libudev-dev:arm64 \
        libevdev-dev:arm64 \
        libxkbcommon-dev:arm64 \
        libpixman-1-dev:arm64 \
        libsystemd-dev:arm64 \
        wayland-protocols \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*