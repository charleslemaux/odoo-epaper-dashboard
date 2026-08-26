/*
** Charles Le Maux, 2026
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

static const char DATED_TODO_FILTER[] = ",\"|\","
    "[\"project_id\",\"!=\",false],[\"date_deadline\",\"!=\",false]]";

static const char PROJECT_ONLY_FILTER[] = ",[\"project_id\",\"!=\",false]]";

int odoo_build_auth(char *dst, size_t size)
{
    int written = snprintf(dst, size, AUTH_TEMPLATE, ODOO_DB, ODOO_LOGIN,
        ODOO_API_KEY);

    if (written < 0 || (size_t)written >= size)
        return -1;
    return written;
}

static int append_filter(char *dst, size_t size, int len, char const *filter)
{
    size_t room = size - (size_t)(len - 1);
    int written = 0;

    if (len < 2 || dst[len - 1] != ']')
        return -1;
    written = snprintf(dst + len - 1, room, "%s", filter);
    if (written < 0 || (size_t)written >= room)
        return -1;
    return len - 1 + written;
}

static int build_domain(char *dst, size_t size, int uid)
{
    int written = snprintf(dst, size, ODOO_TASK_DOMAIN, uid);

    if (written < 1 || (size_t)written >= size)
        return -1;
    if (ODOO_INCLUDE_DATED_TODOS != 0)
        return append_filter(dst, size, written, DATED_TODO_FILTER);
    return append_filter(dst, size, written, PROJECT_ONLY_FILTER);
}

int odoo_build_tasks(char *dst, size_t size, int uid)
{
    char domain[256];
    int written = build_domain(domain, sizeof(domain), uid);

    if (written < 0)
        return -1;
    written = snprintf(dst, size, TASKS_TEMPLATE, ODOO_DB, uid,
        ODOO_API_KEY, domain, ODOO_FETCH_LIMIT);
    if (written < 0 || (size_t)written >= size)
        return -1;
    return written;
}
