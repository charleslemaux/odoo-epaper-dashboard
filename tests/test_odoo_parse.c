/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for Odoo JSON-RPC response parsing
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
static const char SAMPLE_TASKS[] = "{\"jsonrpc\":\"2.0\",\"id\":2,"
    "\"result\":[{\"id\":7,\"name\":\"R\\u00e9parer le module\","
    "\"project_id\":[3,\"Compta\"],\"date_deadline\":\"2026-08-30\","
    "\"stage_id\":[2,\"En cours\"],\"priority\":\"1\"},"
    "{\"id\":8,\"name\":\"Deuxieme tache\",\"project_id\":false,"
    "\"date_deadline\":false,\"stage_id\":[1,\"A faire\"],"
    "\"priority\":\"0\"}]}";

static void test_auth(void)
{
    int uid = 0;

    assert(odoo_parse_auth(SAMPLE_AUTH, strlen(SAMPLE_AUTH), &uid) == 0);
    assert(uid == 42);
    assert(odoo_parse_auth(SAMPLE_AUTH_FAIL, strlen(SAMPLE_AUTH_FAIL),
        &uid) == -1);
    assert(odoo_parse_auth("garbage", 7, &uid) == -1);
}

static void test_tasks_nominal(void)
{
    struct odoo_task_list list;

    assert(odoo_parse_tasks(SAMPLE_TASKS, strlen(SAMPLE_TASKS),
        &list) == 0);
    assert(list.count == 2);
    assert(list.overflow == 0);
    assert(strcmp(list.tasks[0].name, "Reparer le module") == 0);
    assert(strcmp(list.tasks[0].project, "Compta") == 0);
    assert(strcmp(list.tasks[0].deadline, "2026-08-30") == 0);
    assert(strcmp(list.tasks[0].stage, "En cours") == 0);
    assert(list.tasks[0].priority == 1);
}

static void test_tasks_false_fields(void)
{
    struct odoo_task_list list;

    assert(odoo_parse_tasks(SAMPLE_TASKS, strlen(SAMPLE_TASKS),
        &list) == 0);
    assert(strcmp(list.tasks[1].project, "") == 0);
    assert(strcmp(list.tasks[1].deadline, "") == 0);
    assert(strcmp(list.tasks[1].stage, "A faire") == 0);
    assert(list.tasks[1].priority == 0);
}

static void test_error_and_garbage(void)
{
    struct odoo_task_list list;

    assert(odoo_parse_tasks(SAMPLE_ERROR, strlen(SAMPLE_ERROR),
        &list) == -2);
    assert(odoo_parse_tasks("not json at all", 15, &list) == -1);
}

static void build_many(char *dst, size_t size, int n)
{
    size_t pos = 0;

    pos += (size_t)snprintf(dst, size, "{\"result\":[");
    for (int i = 0; i < n; i++)
        pos += (size_t)snprintf(dst + pos, size - pos,
            "%s{\"id\":%d,\"name\":\"T%d\",\"priority\":\"0\"}",
            i > 0 ? "," : "", i, i);
    snprintf(dst + pos, size - pos, "]}");
}

static void test_overflow(void)
{
    static char json[4096];
    struct odoo_task_list list;

    build_many(json, sizeof(json), 13);
    assert(odoo_parse_tasks(json, strlen(json), &list) == 0);
    assert(list.count == 12);
    assert(list.overflow == 1);
    assert(strcmp(list.tasks[11].name, "T11") == 0);
}

int main(void)
{
    test_auth();
    test_tasks_nominal();
    test_tasks_false_fields();
    test_error_and_garbage();
    test_overflow();
    printf("test_odoo_parse: OK\n");
    return 0;
}
