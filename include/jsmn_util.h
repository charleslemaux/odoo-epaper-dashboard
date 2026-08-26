/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Generic helpers over the jsmn token array
*/

#ifndef JSMN_UTIL_H_
    #define JSMN_UTIL_H_

    #include <stddef.h>
    #include "jsmn.h"

struct jsmn_ctx {
    char const *json;
    size_t len;
    jsmntok_t const *toks;
    int count;
};

int jsmn_util_parse(struct jsmn_ctx *ctx, jsmntok_t *toks, int max);
int jsmn_tok_eq(struct jsmn_ctx const *ctx, int idx, char const *s);
int jsmn_next_sibling(struct jsmn_ctx const *ctx, int idx);
int jsmn_find_key(struct jsmn_ctx const *ctx, char const *key);

#endif /* !JSMN_UTIL_H_ */
