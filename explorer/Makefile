.PHONY: build build-release clean cross-build release check fmt

TARGET ?= aarch64-unknown-linux-gnu
SRC_DIR := $(shell pwd)

build:
	cargo build

build-release:
	cargo build --release

cross-build:
	cross build --release --target $(TARGET)

release: cross-build
	@echo "Binary: target/$(TARGET)/release/zeroday-fm"
	@ls -lh target/$(TARGET)/release/zeroday-fm 2>/dev/null || echo "Build failed"

strip: build-release
	@strip target/$(TARGET)/release/zeroday-fm 2>/dev/null || true
	@echo "Stripped binary:"
	@ls -lh target/$(TARGET)/release/zeroday-fm

clean:
	cargo clean

check:
	cargo check

fmt:
	cargo fmt

install: build-release
	install -m 755 target/$(TARGET)/release/zeroday-fm /usr/local/bin/zeroday-fm
	@echo "Installed zeroday-fm"