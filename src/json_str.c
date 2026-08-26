/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** JSON string token extraction with UTF-8 and escape folding to ASCII
*/

#include "json_str.h"

static const struct fold_entry FOLD_TABLE[] = {
    {0xE0, "a"}, {0xE2, "a"}, {0xE4, "a"}, {0xE7, "c"}, {0xE8, "e"},
    {0xE9, "e"}, {0xEA, "e"}, {0xEB, "e"}, {0xEE, "i"}, {0xEF, "i"},
    {0xF4, "o"}, {0xF6, "o"}, {0xF9, "u"}, {0xFB, "u"}, {0xFC, "u"},
    {0xC0, "A"}, {0xC2, "A"}, {0xC7, "C"}, {0xC8, "E"}, {0xC9, "E"},
    {0xCA, "E"}, {0xCB, "E"}, {0xCE, "I"}, {0xCF, "I"}, {0xD4, "O"},
    {0xD9, "U"}, {0xDB, "U"}, {0x153, "oe"}, {0x152, "OE"},
    {0xE6, "ae"}, {0xC6, "AE"}, {0x2019, "'"}, {0x2013, "-"},
    {0x2014, "-"}, {0x2026, "..."}, {0, 0}
};

static int hex_val(char c)
{
    if (c >= '0' && c <= '9')
        return c - '0';
    if ((c | 32) >= 'a' && (c | 32) <= 'f')
        return (c | 32) - 'a' + 10;
    return -1;
}

static size_t decode_hex4(char const *src, uint32_t *cp)
{
    uint32_t value = 0;
    int digit = 0;

    for (size_t i = 0; i < 4; i++) {
        digit = hex_val(src[i]);
        if (digit < 0)
            return 0;
        value = (value << 4) | (uint32_t)digit;
    }
    *cp = value;
    return 4;
}

static size_t decode_escape(char const *src, size_t len, uint32_t *cp)
{
    *cp = '?';
    if (len < 1)
        return 0;
    if (*src == 'u' && len >= 5 && decode_hex4(src + 1, cp) == 4)
        return 5;
    if (*src == 'u')
        return len < 5 ? len : 5;
    if (*src == 'n' || *src == 'r' || *src == 't')
        *cp = ' ';
    if (*src == '"' || *src == '\\' || *src == '/')
        *cp = (uint32_t)*src;
    return 1;
}

static size_t utf8_len(uint8_t lead)
{
    if (lead < 0x80)
        return 1;
    if ((lead & 0xE0) == 0xC0)
        return 2;
    if ((lead & 0xF0) == 0xE0)
        return 3;
    if ((lead & 0xF8) == 0xF0)
        return 4;
    return 0;
}

static size_t decode_utf8(char const *src, size_t len, uint32_t *cp)
{
    size_t need = utf8_len((uint8_t)*src);
    uint32_t value = 0;

    *cp = '?';
    if ((uint8_t)*src < 0x80) {
        *cp = (uint8_t)*src;
        return 1;
    }
    if (need == 0 || need > len)
        return 1;
    value = (uint8_t)*src & (uint32_t)(0xFF >> (need + 1));
    for (size_t i = 1; i < need; i++) {
        if (((uint8_t)src[i] & 0xC0) != 0x80)
            return 1;
        value = (value << 6) | ((uint8_t)src[i] & 0x3F);
    }
    *cp = value;
    return need;
}

static char const *fold_codepoint(uint32_t cp)
{
    static char single[2];

    if (cp >= 0x20 && cp < 0x7F) {
        single[0] = (char)cp;
        single[1] = '\0';
        return single;
    }
    for (size_t i = 0; FOLD_TABLE[i].out != 0; i++) {
        if (FOLD_TABLE[i].cp == cp)
            return FOLD_TABLE[i].out;
    }
    return cp < 0x20 ? " " : "?";
}

static void append_char(struct json_out *out, char c)
{
    if (out->pos + 1 < out->size) {
        out->dst[out->pos] = c;
        out->pos++;
        return;
    }
    if (out->size > 1)
        out->dst[out->size - 2] = '.';
}

static void append_folded(struct json_out *out, char const *s)
{
    for (size_t i = 0; s[i] != '\0'; i++)
        append_char(out, s[i]);
}

void json_str_fold(char *dst, size_t size, char const *src, size_t len)
{
    struct json_out out = {dst, size, 0};
    size_t i = 0;
    uint32_t cp = 0;

    if (size == 0)
        return;
    while (i < len) {
        if (src[i] == '\\')
            i += 1 + decode_escape(src + i + 1, len - i - 1, &cp);
        else
            i += decode_utf8(src + i, len - i, &cp);
        append_folded(&out, fold_codepoint(cp));
    }
    dst[out.pos] = '\0';
}
