/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** HTTP/1.0 request building and response splitting helpers
*/

#include <stdio.h>
#include <string.h>
#include "config.h"
#include "http_util.h"

int http_build_request(char *dst, size_t size, char const *path,
    char const *body)
{
    int written = snprintf(dst, size,
        "POST %s HTTP/1.0\r\n"
        "Host: %s\r\n"
        "Content-Type: application/json\r\n"
        "Content-Length: %u\r\n"
        "Connection: close\r\n\r\n%s",
        path, ODOO_HOST, (unsigned int)strlen(body), body);

    if (written < 0 || (size_t)written >= size)
        return -1;
    return written;
}

int http_parse_status(char const *resp, size_t len)
{
    int status = 0;

    if (len < 12 || strncmp(resp, "HTTP/1.", 7) != 0)
        return -1;
    for (size_t i = 9; i < 12; i++) {
        if (resp[i] < '0' || resp[i] > '9')
            return -1;
        status = status * 10 + (resp[i] - '0');
    }
    return status;
}

long http_body_offset(char const *resp, size_t len)
{
    for (size_t i = 0; i + 3 < len; i++) {
        if (memcmp(resp + i, "\r\n\r\n", 4) == 0)
            return (long)(i + 4);
    }
    return -1;
}
