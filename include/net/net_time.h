/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** SNTP time synchronisation backed by the always-on timer
*/

#ifndef NET_TIME_H_
    #define NET_TIME_H_

    #include <time.h>

void net_time_init(void);
int net_time_synced(void);
void net_time_local(struct tm *out);
void net_time_sntp_set(unsigned int sec);

#endif /* !NET_TIME_H_ */
