/**
 * gps_passthrough.c — GPS Module v1.1 passthrough (AT6558)
 *
 * Uses UART1 in GPS mode (9600 baud). UART1 is time-multiplexed
 * between GPS and C6L — only one active at a time.
 */

#include "zeroday_monsterc5.h"

static const char *TAG = "GPS";
static bool gps_passthrough_active = false;
static TaskHandle_t gps_task_handle = NULL;

static void gps_passthrough_task(void *arg)
{
    char nmea_buf[256];
    ESP_LOGI(TAG, "GPS passthrough task started on UART1 @ 9600 baud");

    while (1) {
        int len = uart_read_bytes(PERIPH_UART_NUM, (uint8_t *)nmea_buf,
                                  sizeof(nmea_buf) - 1, pdMS_TO_TICKS(100));
        if (len > 0 && gps_passthrough_active) {
            nmea_buf[len] = '\0';
            serial_mux_send_prefixed("GPS:", nmea_buf);
        }
        vTaskDelay(pdMS_TO_TICKS(10));
    }
}

void gps_passthrough_init(void)
{
    /* UART1 is configured on-demand when GPS passthrough starts.
     * Initial state: idle. */
    ESP_LOGI(TAG, "GPS passthrough initialized (UART1 @ 9600 baud, Grove IN)");
}

void gps_passthrough_start(void)
{
    if (gps_passthrough_active) return;

    /* Reconfigure UART1 for GPS baud rate */
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
    ESP_ERROR_CHECK(uart_driver_install(PERIPH_UART_NUM, PERIPH_UART_RX_BUF, 0, 0, NULL, 0));

    g_periph_mode = PERIPH_MODE_GPS;
    gps_passthrough_active = true;
    xTaskCreate(gps_passthrough_task, "gps_pass", 4096, NULL, 5, &gps_task_handle);
    ESP_LOGI(TAG, "GPS passthrough started");
}

void gps_passthrough_stop(void)
{
    if (!gps_passthrough_active) return;
    gps_passthrough_active = false;
    g_periph_mode = PERIPH_MODE_IDLE;
    if (gps_task_handle) {
        vTaskDelete(gps_task_handle);
        gps_task_handle = NULL;
    }
    uart_driver_delete(PERIPH_UART_NUM);
    ESP_LOGI(TAG, "GPS passthrough stopped");
}

bool gps_passthrough_is_active(void)
{
    return gps_passthrough_active;
}