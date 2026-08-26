/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** HTTP/1.0 request building and response splitting helpers
*/

#ifndef HTTP_UTIL_H_
    #define HTTP_UTIL_H_

    #include <stddef.h>

int http_build_request(char *dst, size_t size, char const *path,
    char const *body);
int http_parse_status(char const *resp, size_t len);
long http_body_offset(char const *resp, size_t len);

#endif /* !HTTP_UTIL_H_ */
