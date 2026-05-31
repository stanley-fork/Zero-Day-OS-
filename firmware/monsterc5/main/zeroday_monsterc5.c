/**
 * zeroday_monsterc5.c — Main entry point
 *
 * ZERO-DAY OS custom firmware for M5MonsterC5 (ESP32C5)
 * Forked from C5Lab/M5MonsterC5-CardputerADV
 */

#include "zeroday_monsterc5.h"

static const char *TAG = "ZERODAY-C5";

periph_mode_t g_periph_mode = PERIPH_MODE_IDLE;

void app_main(void)
{
    ESP_LOGI(TAG, "ZERO-DAY OS — M5MonsterC5 middle manager");
    ESP_LOGI(TAG, "Firmware: zeroday-monsterc5 v0.1.0");
    ESP_LOGI(TAG, "Hardware: ESP32C5 (160MHz, 400KB SRAM, 4MB Flash)");

    esp_err_t ret = nvs_flash_init();
    if (ret == ESP_ERR_NVS_NO_FREE_PAGES || ret == ESP_ERR_NVS_NEW_VERSION_FOUND) {
        ESP_ERROR_CHECK(nvs_flash_erase());
        ret = nvs_flash_init();
    }
    ESP_ERROR_CHECK(ret);

    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());

    serial_mux_init();
    board_detect_all();
    wifi_attack_init();
    mesh_node_init();

    ESP_LOGI(TAG, "All subsystems initialized");
    ESP_LOGI(TAG, "Grove topology:");
    ESP_LOGI(TAG, "  Cardputer Zero <-> UART0 (console + multiplexed data)");
    ESP_LOGI(TAG, "  UART1 multiplexed: GPS (9600) / C6L (115200)");
    ESP_LOGI(TAG, "  C6L LCD on I2C0 (0x3C)");

    serial_mux_send(MONSTER_UART_NUM, "ZERO-DAY M5MonsterC5 ready\r\n");

    while (1) {
        vTaskDelay(pdMS_TO_TICKS(1000));
    }
}