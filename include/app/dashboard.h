/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Dashboard layout rendering of the task snapshot into the framebuffer
*/

#ifndef DASHBOARD_H_
    #define DASHBOARD_H_

    #include <time.h>
    #include "gfx.h"
    #include "refresh.h"

#define DASH_BANNER_H 60
#define DASH_ROWS_TOP 70
#define DASH_ROW_H 32
#define DASH_FOOTER_Y 458

struct dashboard_data {
    struct snapshot const *snap;
    struct tm today;
    char banner_date[16];
    char updated_hhmm[8];
    char offline_since[8];
};

void dashboard_render(uint8_t *fb, struct dashboard_data const *d);

#endif /* !DASHBOARD_H_ */
