/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for JSON string extraction and ASCII folding
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "json_str.h"

static void test_plain_ascii(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "hello", 5);
    assert(strcmp(out, "hello") == 0);
}

static void test_unicode_escape(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "caf\\u00e9", 9);
    assert(strcmp(out, "cafe") == 0);
}

static void test_raw_utf8(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "d\xc3\xa9j\xc3\xa0 vu", 9);
    assert(strcmp(out, "deja vu") == 0);
}

static void test_escapes(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "a\\\"b\\\\c\\nd", 10);
    assert(strcmp(out, "a\"b\\c d") == 0);
}

static void test_truncation(void)
{
    char out[6];

    json_str_fold(out, sizeof(out), "abcdefgh", 8);
    assert(strcmp(out, "abcd.") == 0);
}

static void test_oe_ligature(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "c\xc5\x93ur", 5);
    assert(strcmp(out, "coeur") == 0);
}

int main(void)
{
    test_plain_ascii();
    test_unicode_escape();
    test_raw_utf8();
    test_escapes();
    test_truncation();
    test_oe_ligature();
    printf("test_json_str: OK\n");
    return 0;
}
