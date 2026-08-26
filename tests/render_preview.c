/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Host tool rendering a sample dashboard frame to a raw 4bpp dump
*/

#include <stdio.h>
#include <string.h>
#include "dashboard.h"

static void add_task(struct odoo_task *task, char const *name,
    char const *deadline, char const *stage)
{
    snprintf(task->name, sizeof(task->name), "%s", name);
    snprintf(task->project, sizeof(task->project), "Compta");
    snprintf(task->deadline, sizeof(task->deadline), "%s", deadline);
    snprintf(task->stage, sizeof(task->stage), "%s", stage);
}

static void fill_tasks(struct odoo_task_list *list)
{
    list->count = 6;
    add_task(&list->tasks[0], "Corriger module facture",
        "2026-08-25", "En cours");
    add_task(&list->tasks[1],
        "Refonte complete du parcours de commande avec validation",
        "2026-08-27", "A faire");
    add_task(&list->tasks[2], "Migration serveur Odoo",
        "2026-09-02", "A faire");
    add_task(&list->tasks[3], "Preparer la demo client",
        "", "Backlog");
    add_task(&list->tasks[4], "Relancer le fournisseur",
        "2026-08-19", "En cours");
    add_task(&list->tasks[5], "Point equipe hebdo",
        "2026-08-28", "A faire");
    list->tasks[0].priority = 1;
    list->tasks[4].priority = 1;
}

static void fill_sample(struct dashboard_data *d, struct snapshot *snap)
{
    memset(snap, 0, sizeof(*snap));
    memset(d, 0, sizeof(*d));
    d->snap = snap;
    d->today.tm_year = 126;
    d->today.tm_mon = 7;
    d->today.tm_mday = 27;
    snprintf(d->banner_date, sizeof(d->banner_date), "jeu 27/08");
    snprintf(d->updated_hhmm, sizeof(d->updated_hhmm), "00:25");
    fill_tasks(&snap->list);
}

int main(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    static struct snapshot snap;
    struct dashboard_data data;
    FILE *out = fopen("preview.raw", "wb");

    if (out == 0)
        return 1;
    fill_sample(&data, &snap);
    dashboard_render(fb, &data);
    fwrite(fb, 1, sizeof(fb), out);
    fclose(out);
    printf("render_preview: wrote preview.raw\n");
    return 0;
}
