/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Date and time formatting helpers plus deadline classification
*/

#ifndef TIME_FMT_H_
    #define TIME_FMT_H_

    #include <stddef.h>
    #include <time.h>

enum deadline_class {
    DL_NONE = 0,
    DL_OVERDUE,
    DL_SOON,
    DL_NORMAL,
};

void time_fmt_banner(char *dst, size_t size, struct tm const *lt);
void time_fmt_hhmm(char *dst, size_t size, struct tm const *lt);
void time_fmt_ddmm(char *dst, size_t size, char const *iso);
int time_fmt_deadline_class(char const *iso, struct tm const *today);
int time_fmt_days_late(char const *iso, struct tm const *today);

#endif /* !TIME_FMT_H_ */
