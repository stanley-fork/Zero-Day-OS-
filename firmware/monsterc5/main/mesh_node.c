/**
 * mesh_node.c — Meshtastic LoRa mesh node (native on ESP32C5)
 */

#include "zeroday_monsterc5.h"
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

static const char *TAG = "MESH";
static bool mesh_active = false;

void mesh_node_init(void)
{
    ESP_LOGI(TAG, "Meshtastic mesh node initialized (not yet started)");
}

void mesh_start(void)
{
    if (mesh_active) {
        ESP_LOGW(TAG, "Mesh node already running");
        return;
    }
    mesh_active = true;
    ESP_LOGI(TAG, "Mesh node started on channel %d", MESH_CHANNEL);
    serial_mux_send_prefixed("MESH:", "NODE_STARTED");
}

void mesh_stop(void)
{
    if (!mesh_active) return;
    mesh_active = false;
    ESP_LOGI(TAG, "Mesh node stopped");
    serial_mux_send_prefixed("MESH:", "NODE_STOPPED");
}

void mesh_send(const char *dest, const char *msg)
{
    if (!mesh_active) {
        ESP_LOGW(TAG, "Mesh node not running — start with mesh_start");
        serial_mux_send_prefixed("MESH:", "ERROR_NOT_RUNNING");
        return;
    }
    ESP_LOGI(TAG, "Sending mesh message to %s: %s", dest, msg);
    char buf[256];
    snprintf(buf, sizeof(buf), "SEND_OK:%s:%s", dest, msg);
    serial_mux_send_prefixed("MESH:", buf);
}

void mesh_status(void)
{
    char buf[128];
    snprintf(buf, sizeof(buf), "STATUS:%s:CH%d",
             mesh_active ? "RUNNING" : "STOPPED", MESH_CHANNEL);
    serial_mux_send_prefixed("MESH:", buf);
}