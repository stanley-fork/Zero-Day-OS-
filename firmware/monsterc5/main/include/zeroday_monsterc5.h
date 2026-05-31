#pragma once

#include "driver/uart.h"
#include "driver/i2c.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_log.h"
#include "esp_system.h"
#include "nvs_flash.h"
#include "esp_netif.h"
#include "esp_event.h"
#include "esp_wifi.h"
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

/* UART0: Console/USB to Cardputer Zero (115200 baud) */
#define MONSTER_UART_NUM      UART_NUM_0
#define MONSTER_UART_BAUD     115200
#define MONSTER_UART_RX_BUF   4096
#define MONSTER_UART_TX_BUF   0

/* UART1: Multiplexed between GPS (Grove IN) and C6L (Grove OUT)
 * ESP32C5 has only 2 hardware UARTs. We time-multiplex UART1:
 * - GPS passthrough: 9600 baud when active
 * - C6L communication: 115200 baud when active
 * Only one active at a time; managed by the command dispatcher. */
#define PERIPH_UART_NUM       UART_NUM_1
#define GPS_UART_BAUD         9600
#define C6L_UART_BAUD         115200
#define PERIPH_UART_RX_BUF    4096

/* GPS pins on Grove IN */
#define GPS_TX_PIN            4
#define GPS_RX_PIN            5

/* C6L pins on Grove OUT */
#define C6L_TX_PIN            17
#define C6L_RX_PIN            18

/* C6L I2C for LCD */
#define C6L_I2C_NUM          I2C_NUM_0
#define C6L_I2C_SDA_PIN      8
#define C6L_I2C_SCL_PIN       9
#define C6L_I2C_ADDR          0x3C
#define C6L_I2C_FREQ_HZ      400000

/* Serial protocol line prefixes */
#define SERIAL_MUX_TAG        "MUX"
#define WIFI_ATTACK_TAG       "WIFI"
#define GPS_TAG              "GPS"
#define C6L_TAG              "C6L"
#define MESH_TAG             "MESH"

#define MAX_SERIAL_LINE      512
#define MESH_CHANNEL         6

/* Peripheral mode for UART1 multiplexing */
typedef enum {
    PERIPH_MODE_IDLE,
    PERIPH_MODE_GPS,
    PERIPH_MODE_C6L
} periph_mode_t;

/* Global state */
extern periph_mode_t g_periph_mode;

/* Function declarations */
void serial_mux_init(void);
int serial_mux_send(uart_port_t uart_num, const char *data);
int serial_mux_send_prefixed(const char *prefix, const char *data);
int serial_mux_read_line(char *buf, int buf_size);

void gps_passthrough_init(void);
void gps_passthrough_start(void);
void gps_passthrough_stop(void);
bool gps_passthrough_is_active(void);

void c6l_routing_init(void);
void c6l_passthrough_start(void);
void c6l_passthrough_stop(void);
void c6l_send_cmd(const char *cmd);
void c6l_lcd_text(const char *text);

void mesh_node_init(void);
void mesh_start(void);
void mesh_stop(void);
void mesh_send(const char *dest, const char *msg);
void mesh_status(void);

void wifi_attack_init(void);
void wifi_process_command(const char *cmd);

void board_detect_all(void);