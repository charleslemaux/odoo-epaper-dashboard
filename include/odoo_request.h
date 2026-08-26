/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC request body builders
*/

#ifndef ODOO_REQUEST_H_
    #define ODOO_REQUEST_H_

    #include <stddef.h>

int odoo_build_auth(char *dst, size_t size);
int odoo_build_tasks(char *dst, size_t size, int uid);

#endif /* !ODOO_REQUEST_H_ */
