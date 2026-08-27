/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Unit tests for Odoo JSON-RPC activity response parsing
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "odoo_parse.h"

static const char SAMPLE_AUTH[] =
    "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":42}";
static const char SAMPLE_AUTH_FAIL[] =
    "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":false}";
static const char SAMPLE_ERROR[] = "{\"jsonrpc\":\"2.0\",\"id\":2,"
    "\"error\":{\"code\":200,\"message\":\"Odoo Server Error\"}}";
static const char SAMPLE_ACTS[] = "{\"jsonrpc\":\"2.0\",\"id\":2,"
    "\"result\":[{\"id\":7,\"res_name\":\"Jean Dupont\","
    "\"summary\":\"Rappeler pour le devis\","
    "\"activity_type_id\":[1,\"Appel\"],"
    "\"date_deadline\":\"2026-08-30\"},"
    "{\"id\":8,\"res_name\":\"Facture F0042\",\"summary\":false,"
    "\"activity_type_id\":[4,\"\\u00c0 faire\"],"
    "\"date_deadline\":\"2026-09-01\"}]}";

static void test_auth(void)
{
    int uid = 0;

    assert(odoo_parse_auth(SAMPLE_AUTH, strlen(SAMPLE_AUTH), &uid) == 0);
    assert(uid == 42);
    assert(odoo_parse_auth(SAMPLE_AUTH_FAIL, strlen(SAMPLE_AUTH_FAIL),
        &uid) == -1);
    assert(odoo_parse_auth("garbage", 7, &uid) == -1);
}

static void test_activities_nominal(void)
{
    struct odoo_activity_list list;

    assert(odoo_parse_activities(SAMPLE_ACTS, strlen(SAMPLE_ACTS),
        &list) == 0);
    assert(list.count == 2);
    assert(list.overflow == 0);
    assert(strcmp(list.items[0].name, "Rappeler pour le devis") == 0);
    assert(strcmp(list.items[0].record, "Jean Dupont") == 0);
    assert(strcmp(list.items[0].kind, "Appel") == 0);
    assert(strcmp(list.items[0].deadline, "2026-08-30") == 0);
}

static void test_summary_fallback(void)
{
    struct odoo_activity_list list;

    assert(odoo_parse_activities(SAMPLE_ACTS, strlen(SAMPLE_ACTS),
        &list) == 0);
    assert(strcmp(list.items[1].name, "Facture F0042") == 0);
    assert(strcmp(list.items[1].record, "") == 0);
    assert(strcmp(list.items[1].kind, "A faire") == 0);
    assert(strcmp(list.items[1].deadline, "2026-09-01") == 0);
}

static void test_error_and_garbage(void)
{
    struct odoo_activity_list list;

    assert(odoo_parse_activities(SAMPLE_ERROR, strlen(SAMPLE_ERROR),
        &list) == -2);
    assert(odoo_parse_activities("not json at all", 15, &list) == -1);
}

static void build_many(char *dst, size_t size, int n)
{
    size_t pos = 0;

    pos += (size_t)snprintf(dst, size, "{\"result\":[");
    for (int i = 0; i < n; i++)
        pos += (size_t)snprintf(dst + pos, size - pos,
            "%s{\"id\":%d,\"res_name\":\"T%d\","
            "\"date_deadline\":\"2026-09-01\"}",
            i > 0 ? "," : "", i, i);
    snprintf(dst + pos, size - pos, "]}");
}

static void test_overflow(void)
{
    static char json[4096];
    struct odoo_activity_list list;

    build_many(json, sizeof(json), 8);
    assert(odoo_parse_activities(json, strlen(json), &list) == 0);
    assert(list.count == 7);
    assert(list.overflow == 1);
    assert(strcmp(list.items[6].name, "T6") == 0);
}

int main(void)
{
    test_auth();
    test_activities_nominal();
    test_summary_fallback();
    test_error_and_garbage();
    test_overflow();
    printf("test_odoo_parse: OK\n");
    return 0;
}
