/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** SNTP time synchronisation backed by the always-on timer
*/

#include "pico/aon_timer.h"
#include "lwip/apps/sntp.h"
#include "config.h"
#include "net_time.h"

void net_time_init(void)
{
    sntp_setoperatingmode(SNTP_OPMODE_POLL);
    sntp_setservername(0, NTP_SERVER);
    sntp_init();
}

void net_time_sntp_set(unsigned int sec)
{
    struct timespec ts = {(time_t)sec, 0};

    if (aon_timer_is_running())
        aon_timer_set_time(&ts);
    else
        aon_timer_start(&ts);
}

int net_time_synced(void)
{
    return aon_timer_is_running() ? 1 : 0;
}

void net_time_local(struct tm *out)
{
    struct timespec ts = {0, 0};
    time_t local = 0;

    aon_timer_get_time(&ts);
    local = ts.tv_sec + TZ_OFFSET_MIN * 60;
    gmtime_r(&local, out);
}
