/**
 * c6l_routing.c — Unit C6L (ESP32-C6) passthrough routing
 *
 * Uses UART1 in C6L mode (115200 baud) and I2C for LCD.
 * UART1 is time-multiplexed between GPS and C6L.
 */

#include "zeroday_monsterc5.h"

static const char *TAG = "C6L";
static bool c6l_passthrough_active = false;
static TaskHandle_t c6l_uart_task_handle = NULL;

static void c6l_uart_rx_task(void *arg)
{
    char rx_buf[256];
    ESP_LOGI(TAG, "C6L UART1 RX task started @ 115200 baud");

    while (1) {
        int len = uart_read_bytes(PERIPH_UART_NUM, (uint8_t *)rx_buf,
                                  sizeof(rx_buf) - 1, pdMS_TO_TICKS(100));
        if (len > 0) {
            rx_buf[len] = '\0';
            serial_mux_send_prefixed("C6L:", rx_buf);
        }
        vTaskDelay(pdMS_TO_TICKS(10));
    }
}

void c6l_routing_init(void)
{
    /* I2C for C6L LCD */
    i2c_config_t i2c_conf = {
        .mode = I2C_MODE_MASTER,
        .sda_io_num = C6L_I2C_SDA_PIN,
        .scl_io_num = C6L_I2C_SCL_PIN,
        .sda_pullup_en = GPIO_PULLUP_ENABLE,
        .scl_pullup_en = GPIO_PULLUP_ENABLE,
        .master.clk_speed = C6L_I2C_FREQ_HZ,
    };
    ESP_ERROR_CHECK(i2c_param_config(C6L_I2C_NUM, &i2c_conf));
    ESP_ERROR_CHECK(i2c_driver_install(C6L_I2C_NUM, I2C_MODE_MASTER, 0, 0, 0));

    ESP_LOGI(TAG, "C6L I2C initialized @ %d Hz (addr 0x%02X)", C6L_I2C_FREQ_HZ, C6L_I2C_ADDR);
    ESP_LOGI(TAG, "C6L UART1 initialized (on-demand @ 115200 baud, Grove OUT)");
}

void c6l_passthrough_start(void)
{
    if (c6l_passthrough_active) return;

    /* Reconfigure UART1 for C6L baud rate */
    uart_config_t uart_config = {
        .baud_rate = C6L_UART_BAUD,
        .data_bits = UART_DATA_8_BITS,
        .parity = UART_PARITY_DISABLE,
        .stop_bits = UART_STOP_BITS_1,
        .flow_ctrl = UART_HW_FLOWCTRL_DISABLE,
        .source_clk = UART_SCLK_DEFAULT,
    };
    ESP_ERROR_CHECK(uart_param_config(PERIPH_UART_NUM, &uart_config));
    ESP_ERROR_CHECK(uart_set_pin(PERIPH_UART_NUM, C6L_TX_PIN,
                                  C6L_RX_PIN, UART_PIN_NO_CHANGE, UART_PIN_NO_CHANGE));
    ESP_ERROR_CHECK(uart_driver_install(PERIPH_UART_NUM, PERIPH_UART_RX_BUF, 0, 0, NULL, 0));

    g_periph_mode = PERIPH_MODE_C6L;
    c6l_passthrough_active = true;
    xTaskCreate(c6l_uart_rx_task, "c6l_uart", 4096, NULL, 5, &c6l_uart_task_handle);
    ESP_LOGI(TAG, "C6L passthrough started");
}

void c6l_passthrough_stop(void)
{
    if (!c6l_passthrough_active) return;
    c6l_passthrough_active = false;
    g_periph_mode = PERIPH_MODE_IDLE;
    if (c6l_uart_task_handle) {
        vTaskDelete(c6l_uart_task_handle);
        c6l_uart_task_handle = NULL;
    }
    uart_driver_delete(PERIPH_UART_NUM);
    ESP_LOGI(TAG, "C6L passthrough stopped");
}

void c6l_send_cmd(const char *cmd)
{
    char buf[256];
    snprintf(buf, sizeof(buf), "%s\r\n", cmd);
    uart_write_bytes(PERIPH_UART_NUM, buf, strlen(buf));
    ESP_LOGI(TAG, "Sent to C6L: %s", cmd);
}

void c6l_lcd_text(const char *text)
{
    char buf[256];
    snprintf(buf, sizeof(buf), "LCD:1:%s", text);
    c6l_send_cmd(buf);
}