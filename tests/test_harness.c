/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Smoke test proving the host test harness compiles and runs
*/

#include <assert.h>
#include <stdio.h>
#include "config.h"

int main(void)
{
    assert(ODOO_PORT == 443);
    printf("test_harness: OK\n");
    return 0;
}
