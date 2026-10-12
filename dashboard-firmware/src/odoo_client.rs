//! Odoo sync orchestration: authenticate, fetch activities, re-auth
//! (port of `src/odoo/odoo_client.c`).
//!
//! Request bodies and the response buffer are kept in module-level statics
//! (same `&'static mut` function-accessor trick as `net_http.rs`'s
//! `static_buf!`) rather than as locals of this async fn: locals that are
//! live across an `.await` point get embedded into the generated state
//! machine of whatever awaits this call (ultimately `main`'s), and the
//! response buffer alone is 16 KB.
//!
//! Only ever called sequentially from the single main task's superloop
//! (never concurrently with itself), same invariant as `net_http::post_json`.

use core::ptr::addr_of_mut;

use embassy_net::Stack;
use embassy_rp::peripherals::TRNG;
use embassy_rp::trng::Trng;

use dashboard_core::odoo::{
    build_activities, build_auth, build_count, parse_activities, parse_auth, parse_count,
    ActivityList, Config, ParseErr, REQ_CAP,
};

use crate::config::{ODOO_ACTIVITY_DOMAIN, ODOO_API_KEY, ODOO_DB, ODOO_LOGIN};
use crate::net_http::post_json;

const RESP_BUF_LEN: usize = 16384;

fn body_buf() -> &'static mut [u8; REQ_CAP] {
    static mut BUF: [u8; REQ_CAP] = [0; REQ_CAP];
    unsafe { &mut *addr_of_mut!(BUF) }
}

fn resp_buf() -> &'static mut [u8; RESP_BUF_LEN] {
    static mut BUF: [u8; RESP_BUF_LEN] = [0; RESP_BUF_LEN];
    unsafe { &mut *addr_of_mut!(BUF) }
}

fn config() -> Config<'static> {
    Config {
        db: ODOO_DB,
        login: ODOO_LOGIN,
        api_key: ODOO_API_KEY,
        domain: ODOO_ACTIVITY_DOMAIN,
    }
}

/// Port of `do_auth`: build the JSON-RPC `authenticate` call, POST it, parse
/// the returned uid. Logs `odoo: authenticated uid={}` on success, matching
/// the C firmware's `printf("odoo: authenticated uid=%d\n", *uid)`.
async fn do_auth(
    stack: Stack<'static>,
    rng: &mut Trng<'static, TRNG>,
    cfg: &Config<'_>,
    uid: &mut i32,
) -> bool {
    let buf = body_buf();
    let Some(len) = build_auth(buf, cfg) else {
        return false;
    };
    let Ok(body) = core::str::from_utf8(&buf[..len]) else {
        return false;
    };
    let resp = resp_buf();
    let Ok(resp) = post_json(stack, rng, "/jsonrpc", body, resp).await else {
        return false;
    };
    match parse_auth(resp.body) {
        Ok(new_uid) => {
            *uid = new_uid;
            log::info!("odoo: authenticated uid={}", new_uid);
            true
        }
        Err(_) => false,
    }
}

/// Port of `do_fetch`: build the `search_read` call, POST it, parse the
/// activity list. Network/build failures map to `ParseErr::Malformed`
/// (never `Odoo`) so only a genuine Odoo-side error response triggers the
/// re-auth retry in [`sync`].
async fn do_fetch(
    stack: Stack<'static>,
    rng: &mut Trng<'static, TRNG>,
    cfg: &Config<'_>,
    uid: i32,
    list: &mut ActivityList,
) -> Result<(), ParseErr> {
    let buf = body_buf();
    let len = build_activities(buf, cfg, uid).ok_or(ParseErr::Malformed)?;
    let body = core::str::from_utf8(&buf[..len]).map_err(|_| ParseErr::Malformed)?;
    let resp = resp_buf();
    let resp = post_json(stack, rng, "/jsonrpc", body, resp)
        .await
        .map_err(|_| ParseErr::Malformed)?;
    parse_activities(resp.body, list)
}

/// Port of `do_count`: build the `search_count` call, POST it, store the
/// total into `list.total`.
async fn do_count(
    stack: Stack<'static>,
    rng: &mut Trng<'static, TRNG>,
    cfg: &Config<'_>,
    uid: i32,
    list: &mut ActivityList,
) -> bool {
    let buf = body_buf();
    let Some(len) = build_count(buf, cfg, uid) else {
        return false;
    };
    let Ok(body) = core::str::from_utf8(&buf[..len]) else {
        return false;
    };
    let resp = resp_buf();
    let Ok(resp) = post_json(stack, rng, "/jsonrpc", body, resp).await else {
        return false;
    };
    match parse_count(resp.body) {
        Ok(total) => {
            list.total = total;
            true
        }
        Err(_) => false,
    }
}

/// Port of `odoo_client_sync`: authenticate when `*uid <= 0`, fetch
/// activities (re-authenticating once and retrying on `ParseErr::Odoo`),
/// then fetch the total count, clamped to never read below the number of
/// activities actually fetched. Returns `false` on any failure.
pub async fn sync(
    stack: Stack<'static>,
    rng: &mut Trng<'static, TRNG>,
    uid: &mut i32,
    list: &mut ActivityList,
) -> bool {
    let cfg = config();

    if *uid <= 0 && !do_auth(stack, rng, &cfg, uid).await {
        return false;
    }

    let mut ret = do_fetch(stack, rng, &cfg, *uid, list).await;
    if ret == Err(ParseErr::Odoo) && do_auth(stack, rng, &cfg, uid).await {
        ret = do_fetch(stack, rng, &cfg, *uid, list).await;
    }
    if ret.is_err() {
        return false;
    }

    if !do_count(stack, rng, &cfg, *uid, list).await {
        return false;
    }
    if list.total < list.count {
        list.total = list.count;
    }
    true
}
