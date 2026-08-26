/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Connection lifecycle and lwIP callbacks for the HTTPS client
*/

#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "lwip/dns.h"
#include "lwip/pbuf.h"
#include "mbedtls/ssl.h"
#include "config.h"
#include "http_client.h"

static unsigned int dns_gen;

static void cb_dns(char const *name, ip_addr_t const *addr, void *arg)
{
    struct http_ctx *ctx = arg;

    (void)name;
    if (ctx->gen != dns_gen)
        return;
    if (addr == 0) {
        ctx->phase = -1;
        return;
    }
    ctx->addr = *addr;
    ctx->phase = 1;
}

static err_t cb_connected(void *arg, struct altcp_pcb *pcb, err_t err)
{
    struct http_ctx *ctx = arg;

    if (err != ERR_OK) {
        ctx->phase = -2;
        return ERR_OK;
    }
    altcp_write(pcb, ctx->request, (u16_t)ctx->request_len, 0);
    altcp_output(pcb);
    ctx->phase = 2;
    return ERR_OK;
}

static err_t cb_recv(void *arg, struct altcp_pcb *pcb, struct pbuf *p,
    err_t err)
{
    struct http_ctx *ctx = arg;
    u16_t room = (u16_t)(ctx->resp_cap - ctx->resp_len - 1);
    u16_t copied = 0;

    (void)err;
    if (p == 0) {
        ctx->phase = 3;
        return ERR_OK;
    }
    copied = pbuf_copy_partial(p, ctx->resp + ctx->resp_len, room, 0);
    ctx->resp_len += copied;
    if (copied < p->tot_len)
        ctx->phase = -3;
    altcp_recved(pcb, p->tot_len);
    pbuf_free(p);
    return ERR_OK;
}

static void cb_err(void *arg, err_t err)
{
    struct http_ctx *ctx = arg;

    (void)err;
    ctx->pcb = 0;
    if (ctx->phase != 3)
        ctx->phase = -4;
}

int http_conn_resolve(struct http_ctx *ctx)
{
    err_t err;

    dns_gen = ctx->gen;
    err = dns_gethostbyname(ODOO_HOST, &ctx->addr, cb_dns, ctx);
    if (err == ERR_OK) {
        ctx->phase = 1;
        return 0;
    }
    if (err != ERR_INPROGRESS)
        return -1;
    return http_conn_wait(ctx, 1, 10000);
}

int http_conn_open(struct http_ctx *ctx, struct altcp_tls_config *tls)
{
    ctx->pcb = altcp_tls_new(tls, IPADDR_TYPE_V4);
    if (ctx->pcb == 0)
        return -1;
    mbedtls_ssl_set_hostname(altcp_tls_context(ctx->pcb), ODOO_HOST);
    altcp_arg(ctx->pcb, ctx);
    altcp_err(ctx->pcb, cb_err);
    altcp_recv(ctx->pcb, cb_recv);
    if (altcp_connect(ctx->pcb, &ctx->addr, ODOO_PORT, cb_connected)
        != ERR_OK)
        return -1;
    return 0;
}

int http_conn_wait(struct http_ctx *ctx, int target, uint32_t timeout_ms)
{
    absolute_time_t deadline = make_timeout_time_ms(timeout_ms);

    while (ctx->phase >= 0 && ctx->phase < target) {
        if (absolute_time_diff_us(get_absolute_time(), deadline) < 0)
            return -1;
        cyw43_arch_poll();
        watchdog_update();
        sleep_ms(5);
    }
    return ctx->phase >= target ? 0 : -1;
}

void http_conn_close(struct http_ctx *ctx)
{
    if (ctx->pcb == 0)
        return;
    altcp_arg(ctx->pcb, 0);
    altcp_recv(ctx->pcb, 0);
    altcp_err(ctx->pcb, 0);
    if (altcp_close(ctx->pcb) != ERR_OK)
        altcp_abort(ctx->pcb);
    ctx->pcb = 0;
}
