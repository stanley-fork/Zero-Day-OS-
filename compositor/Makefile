.PHONY: build build-release clean install check fmt cross-build release

TARGET ?= aarch64-unknown-linux-gnu
RUSTFLAGS ?=
SRC_DIR := $(shell pwd)

build:
	cargo build

build-release:
	cargo build --release

cross-build:
	cross build --release --target $(TARGET)

release: cross-build
	@echo "Binary: target/$(TARGET)/release/zeroday-comp"
	@ls -lh target/$(TARGET)/release/zeroday-comp 2>/dev/null || echo "Build failed"

strip: build-release
	@strip target/$(TARGET)/release/zeroday-comp 2>/dev/null || true
	@echo "Stripped binary:"
	@ls -lh target/$(TARGET)/release/zeroday-comp

clean:
	cargo clean

check:
	cargo check --target $(TARGET)

fmt:
	cargo fmt

install: build-release
	install -m 755 target/$(TARGET)/release/zeroday-comp /usr/local/bin/zeroday-comp
	install -m 755 -d /etc/zeroday
	install -m 644 ../configs/zeroday/comp.env /etc/zeroday/comp.env
	install -m 644 ../configs/systemd/zeroday-comp.service /etc/systemd/system/zeroday-comp.service
	systemctl enable zeroday-comp.service
	@echo "Installed zeroday-comp — reboot or 'systemctl start zeroday-comp'"

deps:
	@echo "Installing cross-compilation dependencies..."
	sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
		libwayland-dev libdrm-dev libgbm-dev libegl-dev libgles2-dev \
		libinput-dev libudev-dev libevdev-dev libxkbcommon-dev \
		libpixman-1-dev libseat-dev libsystemd-dev wayland-protocols pkg-config