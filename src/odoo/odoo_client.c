/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Odoo sync orchestration: authenticate, fetch activities, re-auth
*/

#include <stdio.h>
#include "http_client.h"
#include "odoo_client.h"
#include "odoo_parse.h"
#include "odoo_request.h"

static int do_auth(int *uid)
{
    static char body[ODOO_REQ_CAP];
    struct http_response resp = {0, 0, 0};

    if (odoo_build_auth(body, sizeof(body)) < 0)
        return -1;
    if (http_post_json("/jsonrpc", body, &resp) != 0)
        return -1;
    if (odoo_parse_auth(resp.body, resp.body_len, uid) != 0)
        return -1;
    printf("odoo: authenticated uid=%d\n", *uid);
    return 0;
}

static int do_fetch(int uid, struct odoo_activity_list *list)
{
    static char body[ODOO_REQ_CAP];
    struct http_response resp = {0, 0, 0};

    if (odoo_build_activities(body, sizeof(body), uid) < 0)
        return -1;
    if (http_post_json("/jsonrpc", body, &resp) != 0)
        return -1;
    return odoo_parse_activities(resp.body, resp.body_len, list);
}

static int do_count(int uid, unsigned int *total)
{
    static char body[ODOO_REQ_CAP];
    struct http_response resp = {0, 0, 0};

    if (odoo_build_count(body, sizeof(body), uid) < 0)
        return -1;
    if (http_post_json("/jsonrpc", body, &resp) != 0)
        return -1;
    return odoo_parse_count(resp.body, resp.body_len, total);
}

int odoo_client_sync(int *uid, struct odoo_activity_list *list)
{
    int ret = 0;

    if (*uid <= 0 && do_auth(uid) != 0)
        return -1;
    ret = do_fetch(*uid, list);
    if (ret == -2 && do_auth(uid) == 0)
        ret = do_fetch(*uid, list);
    if (ret != 0)
        return -1;
    if (do_count(*uid, &list->total) != 0)
        return -1;
    if (list->total < list->count)
        list->total = list->count;
    return 0;
}
