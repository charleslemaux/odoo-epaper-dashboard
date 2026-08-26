/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Panel init sequence, refresh cycle and deep sleep (port of main.py)
*/

#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include "hardware/watchdog.h"
#include "epd.h"
#include "gfx.h"

static const struct epd_cmd INIT_SEQUENCE[] = {
    {0xAA, 6, {0x49, 0x55, 0x20, 0x08, 0x09, 0x18}},
    {0x01, 1, {0x3F, 0, 0, 0, 0, 0}},
    {0x00, 2, {0x5F, 0x69, 0, 0, 0, 0}},
    {0x03, 4, {0x00, 0x54, 0x00, 0x44, 0, 0}},
    {0x05, 4, {0x40, 0x1F, 0x1F, 0x2C, 0, 0}},
    {0x06, 4, {0x6F, 0x1F, 0x17, 0x49, 0, 0}},
    {0x08, 4, {0x6F, 0x1F, 0x1F, 0x22, 0, 0}},
    {0x30, 1, {0x03, 0, 0, 0, 0, 0}},
    {0x50, 1, {0x3F, 0, 0, 0, 0, 0}},
    {0x60, 2, {0x02, 0x00, 0, 0, 0, 0}},
    {0x61, 4, {0x03, 0x20, 0x01, 0xE0, 0, 0}},
    {0x84, 1, {0x01, 0, 0, 0, 0, 0}},
    {0xE3, 1, {0x2F, 0, 0, 0, 0, 0}},
};

static void send_init_sequence(void)
{
    size_t total = sizeof(INIT_SEQUENCE) / sizeof(INIT_SEQUENCE[0]);

    for (size_t i = 0; i < total; i++) {
        epd_io_command(INIT_SEQUENCE[i].cmd);
        epd_io_data(INIT_SEQUENCE[i].data, INIT_SEQUENCE[i].len);
    }
}

static int power_on_reacts(void)
{
    absolute_time_t deadline = make_timeout_time_ms(3000);

    epd_io_command(0x04);
    while (absolute_time_diff_us(get_absolute_time(), deadline) > 0) {
        if (gpio_get(EPD_BUSY_PIN) == 0)
            return epd_io_wait_idle(60000);
        watchdog_update();
        sleep_ms(2);
    }
    return -1;
}

int epd_init(void)
{
    for (int attempt = 0; attempt < 3; attempt++) {
        epd_io_reset();
        epd_io_wait_idle(5000);
        send_init_sequence();
        if (power_on_reacts() == 0)
            return 0;
        sleep_ms(200);
    }
    return -1;
}

static int turn_on_display(void)
{
    uint8_t zero = 0x00;

    epd_io_command(0x04);
    if (epd_io_wait_idle(60000) != 0)
        return -1;
    epd_io_command(0x12);
    epd_io_data(&zero, 1);
    if (epd_io_wait_idle(60000) != 0)
        return -1;
    epd_io_command(0x02);
    epd_io_data(&zero, 1);
    return epd_io_wait_idle(60000);
}

int epd_display(uint8_t const *fb)
{
    epd_io_command(0x10);
    epd_io_data(fb, GFX_BUFFER_SIZE);
    return turn_on_display();
}

void epd_sleep(void)
{
    uint8_t code = 0xA5;

    epd_io_command(0x07);
    epd_io_data(&code, 1);
    sleep_ms(20);
}
