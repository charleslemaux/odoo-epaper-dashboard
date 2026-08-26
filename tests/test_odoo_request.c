/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for Odoo JSON-RPC request body builders
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "odoo_request.h"

static void test_auth_body(void)
{
    char body[512];
    int len = odoo_build_auth(body, sizeof(body));

    assert(len > 0);
    assert((size_t)len == strlen(body));
    assert(strstr(body, "\"service\":\"common\"") != 0);
    assert(strstr(body, "\"authenticate\"") != 0);
    assert(strstr(body, "\"testdb\"") != 0);
    assert(strstr(body, "\"tester@test.lan\"") != 0);
    assert(strstr(body, "\"test-key\"") != 0);
}

static void test_tasks_body(void)
{
    char body[2048];
    int len = odoo_build_tasks(body, sizeof(body), 42);

    assert(len > 0);
    assert(strstr(body, "\"execute_kw\"") != 0);
    assert(strstr(body, "\"project.task\"") != 0);
    assert(strstr(body, "\"search_read\"") != 0);
    assert(strstr(body, "[[\"user_ids\",\"in\",[42]]]") != 0);
    assert(strstr(body, "\"limit\":13") != 0);
    assert(strstr(body, "date_deadline asc, priority desc") != 0);
}

static void test_truncation(void)
{
    char body[32];

    assert(odoo_build_auth(body, sizeof(body)) == -1);
    assert(odoo_build_tasks(body, sizeof(body), 1) == -1);
}

int main(void)
{
    test_auth_body();
    test_tasks_body();
    test_truncation();
    printf("test_odoo_request: OK\n");
    return 0;
}
