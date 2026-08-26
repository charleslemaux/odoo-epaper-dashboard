/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC request body builders
*/

#include <stdio.h>
#include "config.h"
#include "odoo.h"
#include "odoo_request.h"

static const char AUTH_TEMPLATE[] = "{\"jsonrpc\":\"2.0\","
    "\"method\":\"call\",\"params\":{\"service\":\"common\","
    "\"method\":\"authenticate\",\"args\":[\"%s\",\"%s\",\"%s\",{}]},"
    "\"id\":1}";

static const char TASKS_TEMPLATE[] = "{\"jsonrpc\":\"2.0\","
    "\"method\":\"call\",\"params\":{\"service\":\"object\","
    "\"method\":\"execute_kw\",\"args\":[\"%s\",%d,\"%s\","
    "\"project.task\",\"search_read\",[%s],{\"fields\":[\"name\","
    "\"project_id\",\"date_deadline\",\"stage_id\",\"priority\"],"
    "\"limit\":%d,\"order\":\"date_deadline asc, priority desc\"}]},"
    "\"id\":2}";

int odoo_build_auth(char *dst, size_t size)
{
    int written = snprintf(dst, size, AUTH_TEMPLATE, ODOO_DB, ODOO_LOGIN,
        ODOO_API_KEY);

    if (written < 0 || (size_t)written >= size)
        return -1;
    return written;
}

int odoo_build_tasks(char *dst, size_t size, int uid)
{
    char domain[256];
    int written = snprintf(domain, sizeof(domain), ODOO_TASK_DOMAIN, uid);

    if (written < 0 || (size_t)written >= sizeof(domain))
        return -1;
    written = snprintf(dst, size, TASKS_TEMPLATE, ODOO_DB, uid,
        ODOO_API_KEY, domain, ODOO_FETCH_LIMIT);
    if (written < 0 || (size_t)written >= size)
        return -1;
    return written;
}
