/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Unit tests for the include-dated-todos toggle (built with flag = 1)
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "odoo_request.h"

static void test_dated_todo_filter_when_enabled(void)
{
    char body[2048];
    int len = odoo_build_tasks(body, sizeof(body), 42);

    assert(len > 0);
    assert(strstr(body, "[[\"user_ids\",\"in\",[42]],\"|\","
        "[\"project_id\",\"!=\",false],"
        "[\"date_deadline\",\"!=\",false]]") != 0);
}

int main(void)
{
    test_dated_todo_filter_when_enabled();
    printf("test_odoo_request_todos: OK\n");
    return 0;
}
