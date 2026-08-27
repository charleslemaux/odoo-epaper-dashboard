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

#define DASH_BANNER_H 72
#define DASH_ROWS_TOP 80
#define DASH_ROW_H 48
#define DASH_FOOTER_Y 444
#define DASH_MARGIN 16
#define DASH_GAP 24
#define DASH_ICON_W 32
#define DASH_TEXT_X 60

struct dash_span {
    int y;
    int from;
    int to;
};

struct dashboard_data {
    struct snapshot const *snap;
    struct tm today;
    char banner_date[16];
    char updated_hhmm[8];
    char offline_since[8];
};

void dashboard_render(uint8_t *fb, struct dashboard_data const *d);

#endif /* !DASHBOARD_H_ */
