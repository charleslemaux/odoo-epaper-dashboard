/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo task data model shared by parsing, requests, refresh and rendering
*/

#ifndef ODOO_H_
    #define ODOO_H_

#define ODOO_MAX_TASKS 12
#define ODOO_FETCH_LIMIT (ODOO_MAX_TASKS + 1)
#define ODOO_REQ_CAP 2048

struct odoo_task {
    char name[64];
    char project[32];
    char deadline[11];
    char stage[24];
    unsigned int priority;
};

struct odoo_task_list {
    unsigned int count;
    unsigned int overflow;
    struct odoo_task tasks[ODOO_MAX_TASKS];
};

#endif /* !ODOO_H_ */
