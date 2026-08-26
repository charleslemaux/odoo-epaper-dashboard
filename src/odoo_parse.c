/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC response parsing into the task data model
*/

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

static void copy_relation(struct jsmn_ctx const *ctx, int idx, char *dst,
    size_t size)
{
    dst[0] = '\0';
    if (ctx->toks[idx].type == JSMN_ARRAY && ctx->toks[idx].size >= 2)
        copy_tok(ctx, idx + 2, dst, size);
}

static unsigned int priority_of(struct jsmn_ctx const *ctx, int idx)
{
    jsmntok_t const *tok = &ctx->toks[idx];

    if (tok->type == JSMN_STRING || tok->type == JSMN_PRIMITIVE)
        return ctx->json[tok->start] == '1';
    return 0;
}

static void set_task_field(struct jsmn_ctx const *ctx, int key,
    struct odoo_task *task)
{
    int val = key + 1;

    if (jsmn_tok_eq(ctx, key, "name"))
        copy_tok(ctx, val, task->name, sizeof(task->name));
    if (jsmn_tok_eq(ctx, key, "date_deadline"))
        copy_tok(ctx, val, task->deadline, sizeof(task->deadline));
    if (jsmn_tok_eq(ctx, key, "project_id"))
        copy_relation(ctx, val, task->project, sizeof(task->project));
    if (jsmn_tok_eq(ctx, key, "stage_id"))
        copy_relation(ctx, val, task->stage, sizeof(task->stage));
    if (jsmn_tok_eq(ctx, key, "priority"))
        task->priority = priority_of(ctx, val);
}

static void parse_task(struct jsmn_ctx const *ctx, int idx,
    struct odoo_task *task)
{
    int child = idx + 1;
    int pairs = ctx->toks[idx].size;

    for (int n = 0; n < pairs; n++) {
        set_task_field(ctx, child, task);
        child = jsmn_next_sibling(ctx, child + 1);
    }
}

static void parse_task_array(struct jsmn_ctx const *ctx, int idx,
    struct odoo_task_list *list)
{
    int child = idx + 1;
    int total = ctx->toks[idx].size;

    for (int n = 0; n < total; n++) {
        if (n < ODOO_MAX_TASKS)
            parse_task(ctx, child, &list->tasks[n]);
        child = jsmn_next_sibling(ctx, child);
    }
    list->count = total > ODOO_MAX_TASKS ? ODOO_MAX_TASKS
        : (unsigned int)total;
    list->overflow = total > ODOO_MAX_TASKS;
}

int odoo_parse_auth(char const *json, size_t len, int *uid)
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
    *uid = atoi(json + ctx.toks[val].start);
    return 0;
}

int odoo_parse_tasks(char const *json, size_t len,
    struct odoo_task_list *list)
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
    parse_task_array(&ctx, val, list);
    return 0;
}
