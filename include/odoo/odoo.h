/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Odoo activity data model shared by parsing, requests, refresh, rendering
*/

#ifndef ODOO_H_
    #define ODOO_H_

#define ODOO_MAX_ACTIVITIES 7
#define ODOO_FETCH_LIMIT (ODOO_MAX_ACTIVITIES + 1)
#define ODOO_REQ_CAP 2048

struct odoo_activity {
    char name[64];
    char record[32];
    char deadline[11];
    char icon[24];
};

struct odoo_activity_list {
    unsigned int count;
    unsigned int overflow;
    struct odoo_activity items[ODOO_MAX_ACTIVITIES];
};

#endif /* !ODOO_H_ */
