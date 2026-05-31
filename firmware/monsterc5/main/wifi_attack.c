/**
 * wifi_attack.c — WiFi attack command dispatcher
 *
 * Receives commands from Cardputer Zero, dispatches to attack functions.
 * Output sent back un-prefixed (upstream WiFi attack stream).
 */

#include "zeroday_monsterc5.h"
#include <string.h>

static const char *TAG = "WIFI";

void wifi_attack_init(void)
{
    ESP_LOGI(TAG, "WiFi attack engine initialized");
}

void wifi_process_command(const char *cmd)
{
    if (strncmp(cmd, "ping", 4) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "pong\r\n");
    } else if (strncmp(cmd, "scan_networks", 13) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "SCAN_START\r\n");
    } else if (strncmp(cmd, "start_deauth", 12) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "DEAUTH_START\r\n");
    } else if (strncmp(cmd, "start_evil_twin", 15) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "EVILTWIN_START\r\n");
    } else if (strncmp(cmd, "start_sae_overflow", 18) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "SAE_OVERFLOW_START\r\n");
    } else if (strncmp(cmd, "start_handshake", 15) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "HANDSHAKE_START\r\n");
    } else if (strncmp(cmd, "start_sniffer", 13) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "SNIFFER_START\r\n");
    } else if (strncmp(cmd, "start_blackout", 14) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "BLACKOUT_START\r\n");
    } else if (strncmp(cmd, "start_wardrive", 14) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "WARDRIVE_START\r\n");
    } else if (strncmp(cmd, "stop", 4) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "STOPPED\r\n");
    } else if (strncmp(cmd, "status", 6) == 0) {
        serial_mux_send(MONSTER_UART_NUM, "STATUS:READY\r\n");
    } else if (strncmp(cmd, "hub_status", 10) == 0) {
        serial_mux_send(MONSTER_UART_NUM,
            "HUB:MonsterC5|GPS:AT6558|C6L:ESP32-C6|MESH:LoRa\r\n");
    } else if (strncmp(cmd, "gps_passthrough_start", 20) == 0) {
        gps_passthrough_start();
    } else if (strncmp(cmd, "gps_passthrough_stop", 19) == 0) {
        gps_passthrough_stop();
    } else if (strncmp(cmd, "c6l_passthrough_start", 21) == 0) {
        c6l_passthrough_start();
    } else if (strncmp(cmd, "c6l_passthrough_stop", 20) == 0) {
        c6l_passthrough_stop();
    } else if (strncmp(cmd, "c6l_cmd ", 8) == 0) {
        c6l_send_cmd(cmd + 8);
    } else if (strncmp(cmd, "mesh_start", 10) == 0) {
        mesh_start();
    } else if (strncmp(cmd, "mesh_stop", 9) == 0) {
        mesh_stop();
    } else if (strncmp(cmd, "mesh_send ", 10) == 0) {
        /* Parse "mesh_send dest msg" */
        char dest[64] = {0};
        char msg[192] = {0};
        if (sscanf(cmd + 10, "%63s %191[^\n]", dest, msg) >= 1) {
            mesh_send(dest, msg);
        }
    } else if (strncmp(cmd, "mesh_status", 11) == 0) {
        mesh_status();
    }
}