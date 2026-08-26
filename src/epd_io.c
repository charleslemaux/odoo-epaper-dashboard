/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Paced bit-bang SPI transport for the e-paper panel (mode 0, MSB first)
*/

#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include "hardware/watchdog.h"
#include "config.h"
#include "epd.h"

void epd_io_init(void)
{
    gpio_init(EPD_RST_PIN);
    gpio_init(EPD_DC_PIN);
    gpio_init(EPD_CS_PIN);
    gpio_init(EPD_CLK_PIN);
    gpio_init(EPD_DIN_PIN);
    gpio_init(EPD_BUSY_PIN);
    gpio_set_dir(EPD_RST_PIN, GPIO_OUT);
    gpio_set_dir(EPD_DC_PIN, GPIO_OUT);
    gpio_set_dir(EPD_CS_PIN, GPIO_OUT);
    gpio_set_dir(EPD_CLK_PIN, GPIO_OUT);
    gpio_set_dir(EPD_DIN_PIN, GPIO_OUT);
    gpio_set_dir(EPD_BUSY_PIN, GPIO_IN);
    gpio_pull_up(EPD_BUSY_PIN);
    gpio_put(EPD_CS_PIN, 1);
    gpio_put(EPD_CLK_PIN, 0);
}

void epd_io_reset(void)
{
    gpio_put(EPD_RST_PIN, 1);
    sleep_ms(20);
    gpio_put(EPD_RST_PIN, 0);
    sleep_ms(2);
    gpio_put(EPD_RST_PIN, 1);
    sleep_ms(20);
}

static void tx_byte(uint8_t byte)
{
    for (int bit = 7; bit >= 0; bit--) {
        gpio_put(EPD_DIN_PIN, (byte >> bit) & 1);
        busy_wait_us(EPD_CLK_HALF_PERIOD_US);
        gpio_put(EPD_CLK_PIN, 1);
        busy_wait_us(EPD_CLK_HALF_PERIOD_US);
        gpio_put(EPD_CLK_PIN, 0);
    }
}

void epd_io_command(uint8_t cmd)
{
    gpio_put(EPD_DC_PIN, 0);
    gpio_put(EPD_CS_PIN, 0);
    tx_byte(cmd);
    gpio_put(EPD_CS_PIN, 1);
}

void epd_io_data(uint8_t const *data, size_t len)
{
    gpio_put(EPD_DC_PIN, 1);
    gpio_put(EPD_CS_PIN, 0);
    for (size_t i = 0; i < len; i++) {
        tx_byte(data[i]);
        if ((i & 0xFF) == 0)
            watchdog_update();
    }
    gpio_put(EPD_CS_PIN, 1);
}

int epd_io_wait_idle(uint32_t timeout_ms)
{
    absolute_time_t deadline = make_timeout_time_ms(timeout_ms);

    sleep_ms(30);
    while (gpio_get(EPD_BUSY_PIN) == 0) {
        if (absolute_time_diff_us(get_absolute_time(), deadline) < 0)
            return -1;
        watchdog_update();
        sleep_ms(10);
    }
    return 0;
}
