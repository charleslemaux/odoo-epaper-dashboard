/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Entry point (wifi bring-up demo, replaced by the real superloop later)
*/

#include <stdio.h>
#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "net_wifi.h"
#include "sys_idle.h"

int main(void)
{
    stdio_init_all();
    watchdog_enable(8000, 1);
    if (net_wifi_init() != 0) {
        printf("wifi: init failed\n");
        return 1;
    }
    while (net_wifi_connect() != 0)
        printf("wifi: connect failed, retrying\n");
    printf("wifi: connected\n");
    for (;;) {
        net_wifi_led(1);
        sys_idle_ms(500);
        net_wifi_led(0);
        sys_idle_ms(500);
    }
    return 0;
}
