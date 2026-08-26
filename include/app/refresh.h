/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Panel refresh decision policy: refresh only on change, daily, spaced
*/

#ifndef REFRESH_H_
    #define REFRESH_H_

    #include <stdint.h>
    #include "odoo.h"

#define REFRESH_MIN_GAP_S 180
#define REFRESH_MAX_AGE_S 86400

struct snapshot {
    struct odoo_task_list list;
    unsigned int offline;
};

struct refresh_times {
    uint32_t now_s;
    uint32_t last_refresh_s;
    unsigned int has_displayed;
};

int refresh_needed(struct snapshot const *prev, struct snapshot const *cur,
    struct refresh_times const *t);

#endif /* !REFRESH_H_ */
