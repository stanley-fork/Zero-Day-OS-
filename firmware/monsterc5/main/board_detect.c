/**
 * board_detect.c — Auto-detect hardware on Grove ports
 */

#include "zeroday_monsterc5.h"
#include "driver/uart.h"
#include "driver/i2c.h"
#include <string.h>

static const char *TAG = "BOARD";

bool board_detect_gps(void)
{
    /* Brief UART1 test at GPS baud */
    uart_config_t uart_config = {
        .baud_rate = GPS_UART_BAUD,
        .data_bits = UART_DATA_8_BITS,
        .parity = UART_PARITY_DISABLE,
        .stop_bits = UART_STOP_BITS_1,
        .flow_ctrl = UART_HW_FLOWCTRL_DISABLE,
        .source_clk = UART_SCLK_DEFAULT,
    };
    ESP_ERROR_CHECK(uart_param_config(PERIPH_UART_NUM, &uart_config));
    ESP_ERROR_CHECK(uart_set_pin(PERIPH_UART_NUM, GPS_TX_PIN,
                                  GPS_RX_PIN, UART_PIN_NO_CHANGE, UART_PIN_NO_CHANGE));
    ESP_ERROR_CHECK(uart_driver_install(PERIPH_UART_NUM, 256, 0, 0, NULL, 0));

    uint8_t data[64];
    int len = uart_read_bytes(PERIPH_UART_NUM, data, sizeof(data), pdMS_TO_TICKS(2000));
    uart_driver_delete(PERIPH_UART_NUM);

    if (len > 0) {
        data[len < sizeof(data) ? len : sizeof(data) - 1] = '\0';
        if (strstr((char *)data, "$G") != NULL) {
            ESP_LOGI(TAG, "GPS Module v1.1 (AT6558) detected on Grove IN");
            return true;
        }
    }
    ESP_LOGW(TAG, "No GPS detected on Grove IN");
    return false;
}

bool board_detect_c6l_i2c(void)
{
    i2c_cmd_handle_t cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, (C6L_I2C_ADDR << 1) | I2C_MASTER_WRITE, true);
    i2c_master_stop(cmd);
    esp_err_t ret = i2c_master_cmd_begin(C6L_I2C_NUM, cmd, pdMS_TO_TICKS(100));
    i2c_cmd_link_delete(cmd);

    if (ret == ESP_OK) {
        ESP_LOGI(TAG, "Unit C6L LCD detected on I2C (0x%02X)", C6L_I2C_ADDR);
        return true;
    }
    ESP_LOGW(TAG, "No C6L LCD on I2C");
    return false;
}

void board_detect_all(void)
{
    ESP_LOGI(TAG, "Detecting hardware on Grove ports...");

    bool gps_ok = board_detect_gps();
    bool c6l_i2c = board_detect_c6l_i2c();

    char status[256];
    snprintf(status, sizeof(status),
             "HUB_STATUS:GPS=%s,C6L_LCD=%s,MESH=ready\r\n",
             gps_ok ? "OK" : "N/A",
             c6l_i2c ? "OK" : "N/A");
    serial_mux_send(MONSTER_UART_NUM, status);
}