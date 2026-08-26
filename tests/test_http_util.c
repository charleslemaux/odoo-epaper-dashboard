/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for HTTP request building and response splitting
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "http_util.h"

static void test_build_request(void)
{
    char req[512];
    int len = http_build_request(req, sizeof(req), "/jsonrpc", "{}");

    assert(len > 0);
    assert(strncmp(req, "POST /jsonrpc HTTP/1.0\r\n", 24) == 0);
    assert(strstr(req, "Host: odoo.test.lan\r\n") != 0);
    assert(strstr(req, "Content-Type: application/json\r\n") != 0);
    assert(strstr(req, "Content-Length: 2\r\n") != 0);
    assert(strstr(req, "Connection: close\r\n\r\n{}") != 0);
}

static void test_build_truncation(void)
{
    char req[16];

    assert(http_build_request(req, sizeof(req), "/jsonrpc", "{}") == -1);
}

static void test_parse_status(void)
{
    static const char OK[] = "HTTP/1.0 200 OK\r\n\r\nbody";
    static const char NOTFOUND[] = "HTTP/1.1 404 Not Found\r\n\r\n";

    assert(http_parse_status(OK, strlen(OK)) == 200);
    assert(http_parse_status(NOTFOUND, strlen(NOTFOUND)) == 404);
    assert(http_parse_status("garbage", 7) == -1);
}

static void test_body_offset(void)
{
    static const char RESP[] = "HTTP/1.0 200 OK\r\nA: b\r\n\r\n{\"x\":1}";

    assert(http_body_offset(RESP, strlen(RESP)) == 25);
    assert(strcmp(RESP + 25, "{\"x\":1}") == 0);
    assert(http_body_offset("no separator", 12) == -1);
}

int main(void)
{
    test_build_request();
    test_build_truncation();
    test_parse_status();
    test_body_offset();
    printf("test_http_util: OK\n");
    return 0;
}
