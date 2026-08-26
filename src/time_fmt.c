/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Date and time formatting helpers plus deadline classification
*/

#include <stdio.h>
#include <string.h>
#include "time_fmt.h"

static const char *DAY_NAMES[7] = {
    "dim", "lun", "mar", "mer", "jeu", "ven", "sam"
};

void time_fmt_banner(char *dst, size_t size, struct tm const *lt)
{
    snprintf(dst, size, "%s %02d/%02d", DAY_NAMES[lt->tm_wday % 7],
        lt->tm_mday, lt->tm_mon + 1);
}

void time_fmt_hhmm(char *dst, size_t size, struct tm const *lt)
{
    snprintf(dst, size, "%02d:%02d", lt->tm_hour, lt->tm_min);
}

void time_fmt_ddmm(char *dst, size_t size, char const *iso)
{
    if (iso == 0 || strlen(iso) < 10) {
        snprintf(dst, size, "--");
        return;
    }
    snprintf(dst, size, "%c%c/%c%c", iso[8], iso[9], iso[5], iso[6]);
}

static void make_iso(char *dst, size_t size, struct tm const *t)
{
    snprintf(dst, size, "%04d-%02d-%02d", t->tm_year + 1900,
        t->tm_mon + 1, t->tm_mday);
}

int time_fmt_deadline_class(char const *iso, struct tm const *today)
{
    char today_iso[11];
    char tomorrow_iso[11];
    struct tm tomorrow = *today;

    if (iso == 0 || iso[0] == '\0')
        return DL_NONE;
    tomorrow.tm_mday++;
    mktime(&tomorrow);
    make_iso(today_iso, sizeof(today_iso), today);
    make_iso(tomorrow_iso, sizeof(tomorrow_iso), &tomorrow);
    if (strncmp(iso, today_iso, 10) < 0)
        return DL_OVERDUE;
    if (strncmp(iso, tomorrow_iso, 10) <= 0)
        return DL_SOON;
    return DL_NORMAL;
}
