/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Waveshare 7.3 inch e-Paper HAT (E) Spectra 6 driver over paced bit-bang
*/

#ifndef EPD_H_
    #define EPD_H_

    #include <stddef.h>
    #include <stdint.h>

    #ifndef EPD_CLK_HALF_PERIOD_US
        #define EPD_CLK_HALF_PERIOD_US 1
    #endif

#define EPD_RST_PIN 12
#define EPD_DC_PIN 8
#define EPD_CS_PIN 9
#define EPD_BUSY_PIN 13
#define EPD_CLK_PIN 10
#define EPD_DIN_PIN 11

struct epd_cmd {
    uint8_t cmd;
    uint8_t len;
    uint8_t data[6];
};

void epd_io_init(void);
void epd_io_reset(void);
void epd_io_command(uint8_t cmd);
void epd_io_data(uint8_t const *data, size_t len);
int epd_io_wait_idle(uint32_t timeout_ms);
int epd_init(void);
int epd_display(uint8_t const *fb);
void epd_sleep(void);

#endif /* !EPD_H_ */
