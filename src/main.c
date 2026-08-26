/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Entry point (toolchain-check stub, replaced by the real superloop later)
*/

#include <stdio.h>
#include "pico/stdlib.h"

int main(void)
{
    stdio_init_all();
    for (;;) {
        printf("epaper_dashboard: toolchain OK\n");
        sleep_ms(1000);
    }
    return 0;
}
