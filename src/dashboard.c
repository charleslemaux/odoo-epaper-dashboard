/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Dashboard layout rendering of the task snapshot into the framebuffer
*/

#include <stdio.h>
#include "dashboard.h"
#include "time_fmt.h"

static void draw_banner(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_rect bar = {0, 0, GFX_WIDTH, DASH_BANNER_H};
    struct gfx_style title = {16, 18, GFX_WHITE, 3};
    struct gfx_style date = {360, 22, GFX_WHITE, 2};
    struct gfx_style count = {0, 22, GFX_WHITE, 2};
    char text[24];

    gfx_fill_rect(fb, &bar, GFX_BLUE);
    gfx_text(fb, &title, "MES TACHES");
    gfx_text(fb, &date, d->banner_date);
    snprintf(text, sizeof(text), "%u ouvertes", d->snap->list.count);
    count.x = GFX_WIDTH - 16 - gfx_text_width(text, 2);
    gfx_text(fb, &count, text);
}

static int deadline_color(struct odoo_task const *task,
    struct tm const *today)
{
    int cls = time_fmt_deadline_class(task->deadline, today);

    if (cls == DL_OVERDUE)
        return GFX_RED;
    if (cls == DL_SOON)
        return GFX_BLUE;
    return GFX_BLACK;
}

static void draw_row(uint8_t *fb, struct odoo_task const *task, int y,
    struct tm const *today)
{
    struct gfx_style star = {8, y + 8, GFX_RED, 2};
    struct gfx_style name = {32, y + 8, GFX_BLACK, 2};
    struct gfx_style proj = {424, y + 8, GFX_BLACK, 2};
    struct gfx_style date = {576, y + 8, deadline_color(task, today), 2};
    struct gfx_style stage = {664, y + 8, GFX_BLACK, 2};
    char field[32];

    if (task->priority > 0)
        gfx_text(fb, &star, "*");
    snprintf(field, sizeof(field), "%.24s", task->name);
    gfx_text(fb, &name, field);
    snprintf(field, sizeof(field), "%.9s", task->project);
    gfx_text(fb, &proj, field);
    time_fmt_ddmm(field, sizeof(field), task->deadline);
    gfx_text(fb, &date, field);
    snprintf(field, sizeof(field), "%.8s", task->stage);
    gfx_text(fb, &stage, field);
}

static void draw_rows(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_rect sep = {8, 0, GFX_WIDTH - 16, 1};
    int y = DASH_ROWS_TOP;

    for (unsigned int i = 0; i < d->snap->list.count; i++) {
        draw_row(fb, &d->snap->list.tasks[i], y, &d->today);
        sep.y = y + DASH_ROW_H - 1;
        gfx_fill_rect(fb, &sep, GFX_BLACK);
        y += DASH_ROW_H;
    }
}

static void draw_footer(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_style left = {8, DASH_FOOTER_Y, GFX_BLACK, 2};
    struct gfx_style right = {0, DASH_FOOTER_Y, GFX_BLACK, 2};
    char text[48];

    snprintf(text, sizeof(text), "mise a jour %s", d->updated_hhmm);
    right.x = GFX_WIDTH - 8 - gfx_text_width(text, 2);
    gfx_text(fb, &right, text);
    if (d->snap->offline != 0) {
        left.color = GFX_RED;
        snprintf(text, sizeof(text), "HORS LIGNE depuis %s",
            d->offline_since);
        gfx_text(fb, &left, text);
        return;
    }
    if (d->snap->list.overflow != 0)
        gfx_text(fb, &left, "+ d'autres taches");
}

static void draw_empty(uint8_t *fb)
{
    struct gfx_style style = {0, 220, GFX_GREEN, 3};

    gfx_text_centered(fb, &style, "Aucune tache ouverte");
}

void dashboard_render(uint8_t *fb, struct dashboard_data const *d)
{
    gfx_fill(fb, GFX_WHITE);
    draw_banner(fb, d);
    if (d->snap->list.count == 0)
        draw_empty(fb);
    else
        draw_rows(fb, d);
    draw_footer(fb, d);
}
