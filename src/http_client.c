/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** One-shot HTTPS POST client over lwIP altcp_tls
*/

#include <string.h>
#include "config.h"
#include "http_client.h"
#include "http_util.h"

#ifdef ODOO_CA_CERT
    #define ODOO_CA_CERT_PTR ((u8_t const *)ODOO_CA_CERT)
    #define ODOO_CA_CERT_LEN (sizeof(ODOO_CA_CERT))
#else
    #define ODOO_CA_CERT_PTR 0
    #define ODOO_CA_CERT_LEN 0
#endif

static struct altcp_tls_config *make_tls_config(void)
{
    return altcp_tls_create_config_client(ODOO_CA_CERT_PTR,
        ODOO_CA_CERT_LEN);
}

static int parse_response(struct http_ctx *ctx, struct http_response *out)
{
    long body_off = 0;

    ctx->resp[ctx->resp_len] = '\0';
    body_off = http_body_offset(ctx->resp, ctx->resp_len);
    out->status = http_parse_status(ctx->resp, ctx->resp_len);
    if (out->status != 200 || body_off < 0)
        return -2;
    out->body = ctx->resp + body_off;
    out->body_len = ctx->resp_len - (size_t)body_off;
    return 0;
}

static int finish(struct http_ctx *ctx, struct altcp_tls_config *tls,
    int code)
{
    http_conn_close(ctx);
    if (tls != 0)
        altcp_tls_free_config(tls);
    return code;
}

int http_post_json(char const *path, char const *body,
    struct http_response *out)
{
    static char resp[HTTP_RESP_CAP];
    static char req[HTTP_REQ_CAP];
    struct http_ctx ctx = {0, req, 0, resp, HTTP_RESP_CAP, 0, {0}, 0};
    struct altcp_tls_config *tls = make_tls_config();
    int built = http_build_request(req, sizeof(req), path, body);

    if (tls == 0 || built < 0)
        return finish(&ctx, tls, -1);
    ctx.request_len = (size_t)built;
    if (http_conn_resolve(&ctx) != 0 || http_conn_open(&ctx, tls) != 0)
        return finish(&ctx, tls, -1);
    if (http_conn_wait(&ctx, 3, 30000) != 0)
        return finish(&ctx, tls, -1);
    return finish(&ctx, tls, parse_response(&ctx, out));
}
