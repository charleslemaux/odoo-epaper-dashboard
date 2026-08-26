/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** WiFi station bring-up, reconnection and status
*/

#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "config.h"
#include "net_wifi.h"

int net_wifi_init(void)
{
    if (cyw43_arch_init_with_country(WIFI_COUNTRY) != 0)
        return -1;
    cyw43_arch_enable_sta_mode();
    return 0;
}

int net_wifi_connect(void)
{
    absolute_time_t deadline = make_timeout_time_ms(30000);
    int status = 0;

    if (cyw43_arch_wifi_connect_async(WIFI_SSID, WIFI_PASSWORD,
        CYW43_AUTH_WPA2_AES_PSK) != 0)
        return -1;
    while (absolute_time_diff_us(get_absolute_time(), deadline) > 0) {
        cyw43_arch_poll();
        watchdog_update();
        status = cyw43_tcpip_link_status(&cyw43_state, CYW43_ITF_STA);
        if (status == CYW43_LINK_UP)
            return 0;
        if (status < 0)
            return -1;
        sleep_ms(50);
    }
    return -1;
}

int net_wifi_up(void)
{
    return cyw43_tcpip_link_status(&cyw43_state, CYW43_ITF_STA)
        == CYW43_LINK_UP;
}

void net_wifi_led(int on)
{
    cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, on != 0);
}
