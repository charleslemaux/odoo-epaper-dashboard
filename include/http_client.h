/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** One-shot HTTPS POST client over lwIP altcp_tls
*/

#ifndef HTTP_CLIENT_H_
    #define HTTP_CLIENT_H_

    #include <stddef.h>
    #include "lwip/altcp.h"
    #include "lwip/altcp_tls.h"
    #include "lwip/ip_addr.h"

#define HTTP_RESP_CAP 16384
#define HTTP_REQ_CAP 4096

struct http_ctx {
    struct altcp_pcb *pcb;
    char const *request;
    size_t request_len;
    char *resp;
    size_t resp_cap;
    size_t resp_len;
    ip_addr_t addr;
    int phase;
    unsigned int gen;
};

struct http_response {
    int status;
    char const *body;
    size_t body_len;
};

int http_conn_resolve(struct http_ctx *ctx);
int http_conn_open(struct http_ctx *ctx, struct altcp_tls_config *tls);
int http_conn_wait(struct http_ctx *ctx, int target, uint32_t timeout_ms);
void http_conn_close(struct http_ctx *ctx);
int http_post_json(char const *path, char const *body,
    struct http_response *out);

#endif /* !HTTP_CLIENT_H_ */
