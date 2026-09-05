/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Dashboard layout rendering of the task snapshot into the framebuffer
*/

#include <stdio.h>
#include <string.h>
#include "dashboard.h"
#include "activity_icon.h"
#include "time_fmt.h"

static void draw_banner(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_rect bar = {0, 0, GFX_WIDTH, DASH_BANNER_H};
    struct gfx_style title = {DASH_MARGIN, 12, GFX_WHITE, 3};
    struct gfx_style date = {0, 24, GFX_WHITE, 2};
    struct gfx_style count = {0, 24, GFX_WHITE, 2};
    char text[24];

    gfx_fill_rect(fb, &bar, GFX_GREEN);
    gfx_text(fb, &title, "Planning");
    date.x = title.x + gfx_text_width("Planning", 3) + 40;
    gfx_text(fb, &date, d->banner_date);
    snprintf(text, sizeof(text), "%u activites", d->snap->list.total);
    count.x = GFX_WIDTH - DASH_MARGIN - gfx_text_width(text, 2);
    gfx_text(fb, &count, text);
}

static int deadline_color(struct odoo_activity const *act,
    struct tm const *today)
{
    int cls = time_fmt_deadline_class(act->deadline, today);

    if (cls == DL_OVERDUE)
        return GFX_RED;
    return GFX_BLACK;
}

static int draw_name(uint8_t *fb, char const *name, int y, int max_w)
{
    struct gfx_style st = {DASH_TEXT_X, y, GFX_BLACK, 2};
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

static void draw_extra(uint8_t *fb, struct odoo_activity const *act,
    struct dash_span const *span)
{
    struct gfx_style st = {0, span->y, GFX_BLACK, 2};
    int width = gfx_text_width(act->record, 2);

    if (act->record[0] == '\0' || width > span->to - span->from)
        return;
    st.x = span->to - width;
    gfx_text(fb, &st, act->record);
}

static int draw_late(uint8_t *fb, struct odoo_activity const *act,
    struct tm const *today, struct dash_span const *span)
{
    struct gfx_style st = {0, span->y, GFX_RED, 2};
    char chip[16];
    int days = time_fmt_days_late(act->deadline, today);

    if (days <= 0)
        return span->to;
    snprintf(chip, sizeof(chip), "%dj", days);
    st.x = span->to - 12 - gfx_text_width(chip, 2);
    gfx_text(fb, &st, chip);
    return st.x;
}

static void draw_row(uint8_t *fb, struct odoo_activity const *act,
    int y, struct tm const *today)
{
    struct gfx_style icon = {DASH_MARGIN, y + 8, GFX_BLACK, 2};
    struct gfx_style date = {0, y + 8, deadline_color(act, today), 2};
    struct dash_span span = {y + 8, 0, 0};
    char ddmm[8];
    int name_end = 0;

    gfx_icon(fb, &icon, activity_icon_for(act->icon));
    time_fmt_ddmm(ddmm, sizeof(ddmm), act->deadline);
    date.x = GFX_WIDTH - DASH_MARGIN - gfx_text_width(ddmm, 2);
    gfx_text(fb, &date, ddmm);
    span.to = date.x;
    span.to = draw_late(fb, act, today, &span);
    name_end = draw_name(fb, act->name, y + 8,
        span.to - DASH_GAP - DASH_TEXT_X);
    if (name_end < 0)
        return;
    span.from = name_end + DASH_GAP;
    span.to -= DASH_GAP;
    if (span.to > span.from)
        draw_extra(fb, act, &span);
}

static void draw_rows(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_rect sep = {0, 0, GFX_WIDTH, 2};
    int y = DASH_ROWS_TOP;

    for (unsigned int i = 0; i < d->snap->list.count; i++) {
        draw_row(fb, &d->snap->list.items[i], y, &d->today);
        sep.y = y + DASH_ROW_H - 2;
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
    if (d->snap->list.total > d->snap->list.count) {
        snprintf(text, sizeof(text), "+ d'autres activites (%u)",
            d->snap->list.total - d->snap->list.count);
        gfx_text(fb, &left, text);
    }
}

static void draw_empty(uint8_t *fb)
{
    struct gfx_style style = {0, 210, GFX_GREEN, 3};

    gfx_text_centered(fb, &style, "Aucune activite");
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
