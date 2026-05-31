set(COMPONENTS main)

set(zeroday_monsterc5_SRCS
    "zeroday_monsterc5.c"
    "serial_mux.c"
    "gps_passthrough.c"
    "c6l_routing.c"
    "mesh_node.c"
    "wifi_attack.c"
    "board_detect.c"
)

idf_component_register(SRCS ${zeroday_monsterc5_SRCS}
    INCLUDE_DIRS "include"
    REQUIRES driver esp_driver_uart esp_driver_i2c esp_wifi esp_event nvs_flash esp_netif lwip json)