/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC response parsing into the task data model
*/

#ifndef ODOO_PARSE_H_
    #define ODOO_PARSE_H_

    #include <stddef.h>
    #include "odoo.h"

#define ODOO_MAX_JSON_TOKENS 512

int odoo_parse_auth(char const *json, size_t len, int *uid);
int odoo_parse_tasks(char const *json, size_t len,
    struct odoo_task_list *list);

#endif /* !ODOO_PARSE_H_ */
