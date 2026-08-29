/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Host tool rendering a sample dashboard frame to a raw 4bpp dump
*/

#include <stdio.h>
#include <string.h>
#include "dashboard.h"

static void add_act(struct odoo_activity *act, char const *name,
    char const *record, char const *info)
{
    snprintf(act->name, sizeof(act->name), "%s", name);
    snprintf(act->record, sizeof(act->record), "%s", record);
    snprintf(act->deadline, sizeof(act->deadline), "%.10s", info);
    snprintf(act->icon, sizeof(act->icon), "%s", info + 11);
}

static void fill_acts(struct odoo_activity_list *list)
{
    list->count = 6;
    add_act(&list->items[0], "Rappeler pour le devis",
        "Jean Dupont", "2026-08-25 fa-phone");
    add_act(&list->items[1],
        "Relire et valider la proposition commerciale complete",
        "Societe Martin & Fils", "2026-08-27 fa-code");
    add_act(&list->items[2], "Facture F0042",
        "", "2026-08-19 fa-cube");
    add_act(&list->items[3], "Envoyer le contrat signe",
        "Dossier 2318", "2026-09-02 fa-microchip");
    add_act(&list->items[4], "Point hebdo equipe",
        "", "2026-08-28 fa-lightbulb-o");
    add_act(&list->items[5], "Preparer la demo",
        "Projet Alpha", "2026-09-05 fa-calendar-check-o");
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
    snprintf(d->updated_hhmm, sizeof(d->updated_hhmm), "03:15");
    fill_acts(&snap->list);
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
