/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Top-level application state owned by main
*/

#ifndef APP_H_
    #define APP_H_

    #include <stdint.h>
    #include "gfx.h"
    #include "refresh.h"

struct app {
    uint8_t fb[GFX_BUFFER_SIZE];
    struct snapshot displayed;
    struct snapshot current;
    int uid;
    unsigned int fails;
    unsigned int has_displayed;
    uint32_t last_refresh_s;
    char offline_since[8];
};

#endif /* !APP_H_ */
