/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for date formatting and deadline classification
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include "time_fmt.h"

static struct tm make_date(int year, int month, int day)
{
    struct tm t = {0};

    t.tm_year = year - 1900;
    t.tm_mon = month - 1;
    t.tm_mday = day;
    t.tm_hour = 12;
    mktime(&t);
    return t;
}

static void test_banner_and_hhmm(void)
{
    struct tm t = make_date(2026, 8, 26);
    char buf[16];

    t.tm_hour = 14;
    t.tm_min = 5;
    time_fmt_banner(buf, sizeof(buf), &t);
    assert(strcmp(buf, "mer 26/08") == 0);
    time_fmt_hhmm(buf, sizeof(buf), &t);
    assert(strcmp(buf, "14:05") == 0);
}

static void test_ddmm(void)
{
    char buf[8];

    time_fmt_ddmm(buf, sizeof(buf), "2026-08-30");
    assert(strcmp(buf, "30/08") == 0);
    time_fmt_ddmm(buf, sizeof(buf), "");
    assert(strcmp(buf, "--") == 0);
}

static void test_deadline_class(void)
{
    struct tm today = make_date(2026, 8, 26);

    assert(time_fmt_deadline_class("2026-08-25", &today) == DL_OVERDUE);
    assert(time_fmt_deadline_class("2026-08-26", &today) == DL_SOON);
    assert(time_fmt_deadline_class("2026-08-27", &today) == DL_SOON);
    assert(time_fmt_deadline_class("2026-08-28", &today) == DL_NORMAL);
    assert(time_fmt_deadline_class("", &today) == DL_NONE);
}

static void test_month_rollover(void)
{
    struct tm today = make_date(2026, 8, 31);

    assert(time_fmt_deadline_class("2026-09-01", &today) == DL_SOON);
    assert(time_fmt_deadline_class("2026-09-02", &today) == DL_NORMAL);
}

int main(void)
{
    test_banner_and_hhmm();
    test_ddmm();
    test_deadline_class();
    test_month_rollover();
    printf("test_time_fmt: OK\n");
    return 0;
}
