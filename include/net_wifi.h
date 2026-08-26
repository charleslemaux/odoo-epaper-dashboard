/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** WiFi station bring-up, reconnection and status
*/

#ifndef NET_WIFI_H_
    #define NET_WIFI_H_

int net_wifi_init(void);
int net_wifi_connect(void);
int net_wifi_up(void);
void net_wifi_led(int on);

#endif /* !NET_WIFI_H_ */
