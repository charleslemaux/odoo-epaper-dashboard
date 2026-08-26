/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Panel refresh decision policy: refresh only on change, daily, spaced
*/

#include <string.h>
#include "refresh.h"

int refresh_needed(struct snapshot const *prev, struct snapshot const *cur,
    struct refresh_times const *t)
{
    int changed = memcmp(prev, cur, sizeof(*prev)) != 0;
    int stale = (t->now_s - t->last_refresh_s) >= REFRESH_MAX_AGE_S;
    int spaced = (t->now_s - t->last_refresh_s) >= REFRESH_MIN_GAP_S;

    if (t->has_displayed == 0)
        return 1;
    return (changed || stale) && spaced;
}
