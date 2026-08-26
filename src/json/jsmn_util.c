/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Generic helpers over the jsmn token array
*/

#define JSMN_STATIC
#include "jsmn.h"
#include <string.h>
#include "jsmn_util.h"

int jsmn_util_parse(struct jsmn_ctx *ctx, jsmntok_t *toks, int max)
{
    jsmn_parser parser;
    int count = 0;

    jsmn_init(&parser);
    count = jsmn_parse(&parser, ctx->json, ctx->len, toks,
        (unsigned int)max);
    if (count < 1)
        return -1;
    ctx->toks = toks;
    ctx->count = count;
    return count;
}

int jsmn_tok_eq(struct jsmn_ctx const *ctx, int idx, char const *s)
{
    jsmntok_t const *tok = &ctx->toks[idx];
    size_t len = (size_t)(tok->end - tok->start);

    if (tok->type != JSMN_STRING || strlen(s) != len)
        return 0;
    return strncmp(ctx->json + tok->start, s, len) == 0;
}

int jsmn_next_sibling(struct jsmn_ctx const *ctx, int idx)
{
    int end = ctx->toks[idx].end;
    int i = idx + 1;

    while (i < ctx->count && ctx->toks[i].start < end)
        i++;
    return i;
}

int jsmn_find_key(struct jsmn_ctx const *ctx, char const *key)
{
    int i = 1;

    if (ctx->count < 1 || ctx->toks[0].type != JSMN_OBJECT)
        return -1;
    while (i + 1 < ctx->count) {
        if (jsmn_tok_eq(ctx, i, key))
            return i + 1;
        i = jsmn_next_sibling(ctx, i + 1);
    }
    return -1;
}
