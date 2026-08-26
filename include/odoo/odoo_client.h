/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Odoo sync orchestration: authenticate, fetch tasks, re-auth on error
*/

#ifndef ODOO_CLIENT_H_
    #define ODOO_CLIENT_H_

    #include "odoo.h"

int odoo_client_sync(int *uid, struct odoo_task_list *list);

#endif /* !ODOO_CLIENT_H_ */
