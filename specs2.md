# Hardware Specification Sheet

A comprehensive technical overview of the portable computing and cyber-deck module.

## Core Features & Connectivity Protocols
* **CAMERA:** Integrated High-Resolution Visual Capture
* **CLI:** Native Command Line Interface Access
* **FILE:** Dedicated Filesystem and Storage Management
* **LoRa MESH:** Long-Range Low-Power Decentralized Mesh Networking
* **PYTHON:** Native Python Execution Environment
* **SSH:** Secure Shell Remote Access Daemon

---

## Technical Specifications

### 1. Compute & Core Architecture
* **Processor (SoC):** Raspberry Pi Compute Module 0 (CM0)
  * **Chipset:** Broadcom BCM2837
  * **Core Configuration:** Quad-Core Cortex-A53
  * **Clock Speed:** 1.0 GHz
* **Memory (RAM):** 512 MB LPDDR2
* **Storage Extension:** MicroSD card slot (dedicated)

### 2. Display & Optical Systems
* **Onboard Display:** 1.9" LCD Screen
  * **Display Controller:** ST7789v3
* **External Video Output:** HDMI Port (Supports up to 1080p @ 30fps)
* **Camera Module:** Sony IMX219
  * **Resolution:** 8 Megapixel ($3200 \times 2464$)
  * **Interface Bus:** CSI 4-Lane

### 3. Interface, Audio & Sensors
* **User Input:** Integrated 46-key mechanical matrix keyboard
* **Audio Architecture:**
  * **Audio Codec:** ES8389 Codec
  * **Microphone:** Integrated MEMS microphone
  * **Speaker:** Onboard 1W speaker
  * **Analog Output:** 3.5mm TRS audio out jack
* **Sensors & Telemetry:**
  * **Inertial Measurement Unit (IMU):** BMI270 (Integrated 3-axis gyroscope + 3-axis accelerometer)
  * **Real-Time Clock (RTC):** RX8130CE with dedicated battery backup

### 4. Power & Energy Management
* **Battery Configuration:** 3.7V / 1500 mAh Lithium-Polymer (Li-Po) battery
* **Fuel Gauge Integrated Circuit:** BQ27220 for precise state of charge (SoC) telemetry

### 5. Connectivity & Networking
* **Wireless Transceiver:** 2.4GHz Wi-Fi (802.11 b/g/n)
* **Bluetooth Engine:** Bluetooth 4.2 / BLE (Bluetooth Low Energy)
* **Wired Ethernet:** 10/100M Native Ethernet controller
* **USB Architecture:** USB 2.0 Engine (Host / Slave switchable)
  * **Physical Ports:** 2x USB Type-C, 1x USB Type-A

### 6. Peripheral Expansion & Signal I/O
* **HY2.0-4P Interface:** Switchable I2C / UART port
* **General Purpose Expansion Bus:** 2.54mm pitch 14-pin header block
  * **Supported Protocols:** SPI, UART, I2C, USB, GPIO, and 5V Power Rails
* **Infrared System:** Dedicated IR Transceiver (Transmitter TX & Receiver RX)

### 7. Media & Video Codecs
* **Hardware Decoding:** H.264 / MPEG-4 @ 1080p 30fps
* **Hardware Encoding:** H.264 @ 1080p 30fps

---

> *Disclaimer: Specifications are preliminary and subject to change. Final hardware specifications will be confirmed before commercial launch.*
