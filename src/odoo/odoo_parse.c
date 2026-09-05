/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC response parsing into the activity data model
*/

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "jsmn_util.h"
#include "json_str.h"
#include "odoo_parse.h"

static void copy_tok(struct jsmn_ctx const *ctx, int idx, char *dst,
    size_t size)
{
    jsmntok_t const *tok = &ctx->toks[idx];

    dst[0] = '\0';
    if (idx < ctx->count && tok->type == JSMN_STRING)
        json_str_fold(dst, size, ctx->json + tok->start,
            (size_t)(tok->end - tok->start));
}

static void set_field(struct jsmn_ctx const *ctx, int key,
    struct odoo_activity *act)
{
    int val = key + 1;

    if (jsmn_tok_eq(ctx, key, "summary"))
        copy_tok(ctx, val, act->name, sizeof(act->name));
    if (jsmn_tok_eq(ctx, key, "res_name"))
        copy_tok(ctx, val, act->record, sizeof(act->record));
    if (jsmn_tok_eq(ctx, key, "date_deadline"))
        copy_tok(ctx, val, act->deadline, sizeof(act->deadline));
    if (jsmn_tok_eq(ctx, key, "icon"))
        copy_tok(ctx, val, act->icon, sizeof(act->icon));
}

static void parse_activity(struct jsmn_ctx const *ctx, int idx,
    struct odoo_activity *act)
{
    int child = idx + 1;
    int pairs = ctx->toks[idx].size;

    for (int n = 0; n < pairs; n++) {
        set_field(ctx, child, act);
        child = jsmn_next_sibling(ctx, child + 1);
    }
    if (act->name[0] != '\0')
        return;
    snprintf(act->name, sizeof(act->name), "%s", act->record);
    act->record[0] = '\0';
}

static void parse_activity_array(struct jsmn_ctx const *ctx, int idx,
    struct odoo_activity_list *list)
{
    int child = idx + 1;
    int size = ctx->toks[idx].size;

    for (int n = 0; n < size; n++) {
        if (n < ODOO_MAX_ACTIVITIES)
            parse_activity(ctx, child, &list->items[n]);
        child = jsmn_next_sibling(ctx, child);
    }
    list->count = size > ODOO_MAX_ACTIVITIES ? ODOO_MAX_ACTIVITIES
        : (unsigned int)size;
}

static int parse_result_int(char const *json, size_t len, int *out)
{
    static jsmntok_t toks[64];
    struct jsmn_ctx ctx = {json, len, 0, 0};
    int val = 0;

    if (jsmn_util_parse(&ctx, toks, 64) < 0)
        return -1;
    val = jsmn_find_key(&ctx, "result");
    if (val < 0 || val >= ctx.count
        || ctx.toks[val].type != JSMN_PRIMITIVE)
        return -1;
    if (json[ctx.toks[val].start] < '0'
        || json[ctx.toks[val].start] > '9')
        return -1;
    *out = atoi(json + ctx.toks[val].start);
    return 0;
}

int odoo_parse_auth(char const *json, size_t len, int *uid)
{
    return parse_result_int(json, len, uid);
}

int odoo_parse_count(char const *json, size_t len, unsigned int *total)
{
    int value = 0;

    if (parse_result_int(json, len, &value) != 0 || value < 0)
        return -1;
    *total = (unsigned int)value;
    return 0;
}

int odoo_parse_activities(char const *json, size_t len,
    struct odoo_activity_list *list)
{
    static jsmntok_t toks[ODOO_MAX_JSON_TOKENS];
    struct jsmn_ctx ctx = {json, len, 0, 0};
    int val = 0;

    memset(list, 0, sizeof(*list));
    if (jsmn_util_parse(&ctx, toks, ODOO_MAX_JSON_TOKENS) < 0)
        return -1;
    if (jsmn_find_key(&ctx, "error") >= 0)
        return -2;
    val = jsmn_find_key(&ctx, "result");
    if (val < 0 || val >= ctx.count || ctx.toks[val].type != JSMN_ARRAY)
        return -1;
    parse_activity_array(&ctx, val, list);
    return 0;
}
