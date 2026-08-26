/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Watchdog-fed idle wait that keeps the wifi driver polled
*/

#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "sys_idle.h"

void sys_idle_ms(uint32_t ms)
{
    absolute_time_t deadline = make_timeout_time_ms(ms);

    while (absolute_time_diff_us(get_absolute_time(), deadline) > 0) {
        cyw43_arch_poll();
        watchdog_update();
        sleep_ms(10);
    }
}
