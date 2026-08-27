/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Unit tests for the panel refresh decision policy
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "refresh.h"

static void test_first_display(void)
{
    struct snapshot a;
    struct snapshot b;
    struct refresh_times t = {30, 0, 0};

    memset(&a, 0, sizeof(a));
    memset(&b, 0, sizeof(b));
    assert(refresh_needed(&a, &b, &t) == 1);
}

static void test_unchanged_skips(void)
{
    struct snapshot a;
    struct refresh_times t = {10000, 5000, 1};

    memset(&a, 0, sizeof(a));
    assert(refresh_needed(&a, &a, &t) == 0);
}

static void test_change_triggers_when_spaced(void)
{
    struct snapshot a;
    struct snapshot b;
    struct refresh_times spaced = {1000, 500, 1};
    struct refresh_times close = {600, 500, 1};

    memset(&a, 0, sizeof(a));
    memset(&b, 0, sizeof(b));
    snprintf(b.list.items[0].name, sizeof(b.list.items[0].name), "X");
    b.list.count = 1;
    assert(refresh_needed(&a, &b, &spaced) == 1);
    assert(refresh_needed(&a, &b, &close) == 0);
}

static void test_daily_health_refresh(void)
{
    struct snapshot a;
    struct refresh_times t = {90000, 100, 1};

    memset(&a, 0, sizeof(a));
    assert(refresh_needed(&a, &a, &t) == 1);
}

static void test_offline_flip_triggers(void)
{
    struct snapshot a;
    struct snapshot b;
    struct refresh_times t = {1000, 500, 1};

    memset(&a, 0, sizeof(a));
    memset(&b, 0, sizeof(b));
    b.offline = 1;
    assert(refresh_needed(&a, &b, &t) == 1);
}

int main(void)
{
    test_first_display();
    test_unchanged_skips();
    test_change_triggers_when_spaced();
    test_daily_health_refresh();
    test_offline_flip_triggers();
    printf("test_refresh: OK\n");
    return 0;
}
