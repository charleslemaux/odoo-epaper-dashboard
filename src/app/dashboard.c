/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Dashboard layout rendering of the task snapshot into the framebuffer
*/

#include <stdio.h>
#include <string.h>
#include "dashboard.h"
#include "time_fmt.h"

static void draw_banner(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_rect bar = {0, 0, GFX_WIDTH, DASH_BANNER_H};
    struct gfx_style title = {DASH_MARGIN, 12, GFX_WHITE, 3};
    struct gfx_style date = {0, 24, GFX_WHITE, 2};
    struct gfx_style count = {0, 24, GFX_WHITE, 2};
    char text[24];

    gfx_fill_rect(fb, &bar, GFX_GREEN);
    gfx_text(fb, &title, "MES TACHES");
    gfx_text_centered(fb, &date, d->banner_date);
    snprintf(text, sizeof(text), "%u ouvertes", d->snap->list.count);
    count.x = GFX_WIDTH - DASH_MARGIN - gfx_text_width(text, 2);
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

static int draw_name(uint8_t *fb, char const *name, int y, int max_w)
{
    struct gfx_style st = {DASH_NAME_X, y, GFX_BLACK, 2};
    char text[72];
    int fit = gfx_text_fit(name, 2, max_w);

    if (name[fit] == '\0') {
        gfx_text(fb, &st, name);
        return st.x + gfx_text_width(name, 2);
    }
    fit = gfx_text_fit(name, 2, max_w - gfx_text_width("...", 2));
    snprintf(text, sizeof(text), "%.*s...", fit, name);
    gfx_text(fb, &st, text);
    return -1;
}

static void append_fit(char *dst, size_t size, char const *piece, int max)
{
    size_t len = strlen(dst);

    if (piece[0] == '\0')
        return;
    snprintf(dst + len, size - len, "%s%s", len > 0 ? " - " : "", piece);
    if (gfx_text_width(dst, 2) > max)
        dst[len] = '\0';
}

static void draw_extra(uint8_t *fb, struct odoo_task const *task,
    struct tm const *today, struct dash_span const *span)
{
    struct gfx_style st = {0, span->y, GFX_BLACK, 2};
    char info[96];
    char late[24];
    int max = span->to - span->from;
    int days = time_fmt_days_late(task->deadline, today);

    info[0] = '\0';
    late[0] = '\0';
    if (days > 0)
        snprintf(late, sizeof(late), "retard %dj", days);
    append_fit(info, sizeof(info), late, max);
    append_fit(info, sizeof(info), task->project, max);
    append_fit(info, sizeof(info), task->stage, max);
    if (info[0] == '\0')
        return;
    st.x = span->to - gfx_text_width(info, 2);
    gfx_text(fb, &st, info);
}

static void draw_row(uint8_t *fb, struct odoo_task const *task, int y,
    struct tm const *today)
{
    struct gfx_style star = {8, y + 8, GFX_RED, 2};
    struct gfx_style date = {0, y + 8, deadline_color(task, today), 2};
    struct dash_span span = {y + 8, 0, 0};
    char ddmm[8];
    int name_end = 0;

    if (task->priority > 0)
        gfx_text(fb, &star, "*");
    time_fmt_ddmm(ddmm, sizeof(ddmm), task->deadline);
    date.x = GFX_WIDTH - DASH_MARGIN - gfx_text_width(ddmm, 2);
    gfx_text(fb, &date, ddmm);
    name_end = draw_name(fb, task->name, y + 8,
        date.x - DASH_GAP - DASH_NAME_X);
    if (name_end < 0)
        return;
    span.from = name_end + DASH_GAP;
    span.to = date.x - DASH_GAP;
    if (span.to > span.from)
        draw_extra(fb, task, today, &span);
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
    struct gfx_style style = {0, 210, GFX_GREEN, 3};

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
