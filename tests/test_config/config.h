/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Fake configuration used by host-side unit tests only
*/

#ifndef CONFIG_H_
    #define CONFIG_H_

#define WIFI_SSID "test-ssid"
#define WIFI_PASSWORD "test-pass"
#define ODOO_HOST "odoo.test.lan"
#define ODOO_PORT 443
#define ODOO_DB "testdb"
#define ODOO_LOGIN "tester@test.lan"
#define ODOO_API_KEY "test-key"
#define ODOO_TASK_DOMAIN "[[\"user_ids\",\"in\",[%d]]]"
    #ifndef ODOO_INCLUDE_DATED_TODOS
        #define ODOO_INCLUDE_DATED_TODOS 0
    #endif
#define POLL_INTERVAL_S 300
#define NTP_SERVER "pool.ntp.org"
#define TZ_OFFSET_MIN 120
#define EPD_CLK_HALF_PERIOD_US 1

#endif /* !CONFIG_H_ */
