/*
** Charles Le Maux, 2026
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

static void test_activities_body(void)
{
    char body[2048];
    int len = odoo_build_activities(body, sizeof(body), 42);

    assert(len > 0);
    assert(strstr(body, "\"execute_kw\"") != 0);
    assert(strstr(body, "\"mail.activity\"") != 0);
    assert(strstr(body, "\"search_read\"") != 0);
    assert(strstr(body, "[[\"user_id\",\"=\",42]]") != 0);
    assert(strstr(body, "\"res_name\"") != 0);
    assert(strstr(body, "\"summary\"") != 0);
    assert(strstr(body, "\"icon\"") != 0);
    assert(strstr(body, "\"limit\":8") != 0);
    assert(strstr(body, "\"order\":\"date_deadline asc\"") != 0);
}

static void test_truncation(void)
{
    char body[32];

    assert(odoo_build_auth(body, sizeof(body)) == -1);
    assert(odoo_build_activities(body, sizeof(body), 1) == -1);
}

int main(void)
{
    test_auth_body();
    test_activities_body();
    test_truncation();
    printf("test_odoo_request: OK\n");
    return 0;
}
