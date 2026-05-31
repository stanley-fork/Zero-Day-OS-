# ZERO-DAY OS — M5MonsterC5 firmware build
#
# ESP-IDF must be sourced before building. On fish shell, use:
#   bash -c 'source ~/esp/esp-idf/export.sh && cd /path/to/monsterc5 && make build'
#
# Prerequisites:
#   1. ESP-IDF v5.4 installed at ~/esp/esp-idf
#   2. cmake installed (sudo apt install cmake)
#
# Usage:
#   make setup     — Set up ESP-IDF target (esp32c5)
#   make build     — Build the firmware
#   make flash     — Flash via esptool (USB)
#   make monitor   — Open serial monitor
#   make clean     — Clean build artifacts
#   make dist      — Copy binary to deployment dir

IDF_SH := ~/esp/esp-idf/export.sh
BUILD_DIR := build
BINARY    := $(BUILD_DIR)/zeroday-monsterc5.bin
DEPLOY_DIR := /opt/cardputer/monsterc5/firmware

.PHONY: setup build flash monitor clean dist deps help

help:
	@echo "ZERO-DAY OS — M5MonsterC5 firmware build"
	@echo ""
	@echo "Targets:"
	@echo "  deps       — Install build dependencies (cmake, ninja)"
	@echo "  setup     — Set up ESP-IDF target (esp32c5)"
	@echo "  build     — Build the firmware"
	@echo "  flash     — Flash firmware via USB (esptool)"
	@echo "  monitor   — Open serial monitor"
	@echo "  clean     — Clean build artifacts"
	@echo "  dist      — Copy binary to deployment dir"
	@echo ""
	@echo "On fish shell, run everything through bash:"
	@echo "  bash -c 'source ~/esp/esp-idf/export.sh && make build'"

deps:
	@echo "Installing build dependencies..."
	sudo pacman -S --needed cmake ninja pkg-config

setup:
	@echo "Setting up ESP-IDF target esp32c5..."
	bash -c 'source $(IDF_SH) && idf.py --preview set-target esp32c5'

build:
	bash -c 'source $(IDF_SH) && idf.py build'

flash:
	bash -c 'source $(IDF_SH) && idf.py -p /dev/ttyUSB0 flash'

monitor:
	bash -c 'source $(IDF_SH) && idf.py -p /dev/ttyUSB0 monitor'

clean:
	bash -c 'source $(IDF_SH) && idf.py fullclean'

dist: build
	mkdir -p $(DEPLOY_DIR)
	cp $(BUILD_DIR)/zeroday-monsterc5.bin $(DEPLOY_DIR)/
	@echo "Binary copied to $(DEPLOY_DIR)/zeroday-monsterc5.bin"
	@echo "Use 'monsterctl flash local' to flash from Cardputer Zero"