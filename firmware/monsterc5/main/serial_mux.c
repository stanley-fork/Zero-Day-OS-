/**
 * serial_mux.c — Serial multiplexer for UART0 to Cardputer Zero
 */

#include "zeroday_monsterc5.h"

static const char *TAG = "MUX";

void serial_mux_init(void)
{
    uart_config_t uart_config = {
        .baud_rate = MONSTER_UART_BAUD,
        .data_bits = UART_DATA_8_BITS,
        .parity = UART_PARITY_DISABLE,
        .stop_bits = UART_STOP_BITS_1,
        .flow_ctrl = UART_HW_FLOWCTRL_DISABLE,
        .source_clk = UART_SCLK_DEFAULT,
    };
    ESP_ERROR_CHECK(uart_param_config(MONSTER_UART_NUM, &uart_config));
    ESP_ERROR_CHECK(uart_set_pin(MONSTER_UART_NUM, UART_PIN_NO_CHANGE,
                                  UART_PIN_NO_CHANGE, UART_PIN_NO_CHANGE, UART_PIN_NO_CHANGE));
    ESP_ERROR_CHECK(uart_driver_install(MONSTER_UART_NUM, MONSTER_UART_RX_BUF, 0, 0, NULL, 0));

    ESP_LOGI(TAG, "Console UART0 initialized @ %d baud", MONSTER_UART_BAUD);
}

int serial_mux_send(uart_port_t uart_num, const char *data)
{
    int len = strlen(data);
    return uart_write_bytes(uart_num, data, len);
}

int serial_mux_send_prefixed(const char *prefix, const char *data)
{
    char buf[MAX_SERIAL_LINE + 16];
    snprintf(buf, sizeof(buf), "%s%s\r\n", prefix, data);
    return serial_mux_send(MONSTER_UART_NUM, buf);
}

int serial_mux_read_line(char *buf, int buf_size)
{
    int len = uart_read_bytes(MONSTER_UART_NUM, (uint8_t *)buf, buf_size - 1, pdMS_TO_TICKS(10));
    if (len <= 0) return 0;
    buf[len] = '\0';

    char *nl = strchr(buf, '\n');
    if (nl) *nl = '\0';
    char *cr = strchr(buf, '\r');
    if (cr) *cr = '\0';

    return len;
}