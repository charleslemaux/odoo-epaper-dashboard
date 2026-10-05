//! Odoo activity model, JSON-RPC request builders and response parsers
//! (port of src/odoo/*.c).

use crate::bufwrite::SliceWriter;
use crate::json::{self, Ctx, TokKind, Token};
use core::fmt::Write;
use heapless::String;

pub const MAX_ACTIVITIES: usize = 7;
pub const FETCH_LIMIT: usize = MAX_ACTIVITIES;
pub const REQ_CAP: usize = 2048;
pub const MAX_JSON_TOKENS: usize = 512;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Activity {
    pub name: String<64>,
    pub record: String<32>,
    pub deadline: String<11>,
    pub icon: String<24>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct ActivityList {
    pub count: u32,
    pub total: u32,
    pub items: [Activity; MAX_ACTIVITIES],
}

pub struct Config<'a> {
    pub db: &'a str,
    pub login: &'a str,
    pub api_key: &'a str,
    pub domain: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseErr {
    Malformed,
    Odoo,
}

fn write_domain(w: &mut SliceWriter, tmpl: &str, uid: i32) -> Option<()> {
    match tmpl.split_once("%d") {
        Some((pre, post)) => write!(w, "{pre}{uid}{post}").ok(),
        None => w.write_str(tmpl).ok(),
    }
}

pub fn build_auth(dst: &mut [u8], cfg: &Config) -> Option<usize> {
    let mut w = SliceWriter::new(dst);
    write!(
        w,
        "{{\"jsonrpc\":\"2.0\",\"method\":\"call\",\"params\":{{\"service\":\"common\",\
         \"method\":\"authenticate\",\"args\":[\"{}\",\"{}\",\"{}\",{{}}]}},\"id\":1}}",
        cfg.db, cfg.login, cfg.api_key
    )
    .ok()?;
    Some(w.len())
}

pub fn build_activities(dst: &mut [u8], cfg: &Config, uid: i32) -> Option<usize> {
    let mut w = SliceWriter::new(dst);
    write!(
        w,
        "{{\"jsonrpc\":\"2.0\",\"method\":\"call\",\"params\":{{\"service\":\"object\",\
         \"method\":\"execute_kw\",\"args\":[\"{}\",{},\"{}\",\"mail.activity\",\
         \"search_read\",[",
        cfg.db, uid, cfg.api_key
    )
    .ok()?;
    write_domain(&mut w, cfg.domain, uid)?;
    write!(
        w,
        "],{{\"fields\":[\"res_name\",\"summary\",\"icon\",\"date_deadline\"],\
         \"limit\":{FETCH_LIMIT},\"order\":\"date_deadline asc\"}}]}},\"id\":2}}"
    )
    .ok()?;
    Some(w.len())
}

pub fn build_count(dst: &mut [u8], cfg: &Config, uid: i32) -> Option<usize> {
    let mut w = SliceWriter::new(dst);
    write!(
        w,
        "{{\"jsonrpc\":\"2.0\",\"method\":\"call\",\"params\":{{\"service\":\"object\",\
         \"method\":\"execute_kw\",\"args\":[\"{}\",{},\"{}\",\"mail.activity\",\
         \"search_count\",[",
        cfg.db, uid, cfg.api_key
    )
    .ok()?;
    write_domain(&mut w, cfg.domain, uid)?;
    write!(w, "]]}},\"id\":3}}").ok()?;
    Some(w.len())
}

fn fold_to<const N: usize>(src: &[u8]) -> String<N> {
    let mut buf = [0u8; N];
    let len = json::fold_into(&mut buf, src);
    let mut s = String::new();
    if let Ok(text) = core::str::from_utf8(&buf[..len]) {
        let _ = s.push_str(text);
    }
    s
}

fn parse_result_int(json_bytes: &[u8]) -> Result<i64, ParseErr> {
    let mut toks = [Token::EMPTY; 64];
    let count = json::parse(json_bytes, &mut toks).map_err(|()| ParseErr::Malformed)?;
    let ctx = Ctx {
        json: json_bytes,
        toks: &toks[..count],
    };
    let val = ctx.find_key("result").ok_or(ParseErr::Malformed)?;
    if ctx.toks[val].kind != TokKind::Primitive {
        return Err(ParseErr::Malformed);
    }
    let text = core::str::from_utf8(ctx.bytes(val)).map_err(|_| ParseErr::Malformed)?;
    if !text.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Err(ParseErr::Malformed);
    }
    text.parse().map_err(|_| ParseErr::Malformed)
}

pub fn parse_auth(json_bytes: &[u8]) -> Result<i32, ParseErr> {
    Ok(parse_result_int(json_bytes)? as i32)
}

pub fn parse_count(json_bytes: &[u8]) -> Result<u32, ParseErr> {
    Ok(parse_result_int(json_bytes)? as u32)
}

fn set_field(ctx: &Ctx, key: usize, act: &mut Activity) {
    let val = key + 1;
    if val >= ctx.toks.len() || ctx.toks[val].kind != TokKind::Str {
        return;
    }
    if ctx.tok_eq(key, "summary") {
        act.name = fold_to(ctx.bytes(val));
    } else if ctx.tok_eq(key, "res_name") {
        act.record = fold_to(ctx.bytes(val));
    } else if ctx.tok_eq(key, "date_deadline") {
        act.deadline = fold_to(ctx.bytes(val));
    } else if ctx.tok_eq(key, "icon") {
        act.icon = fold_to(ctx.bytes(val));
    }
}

fn parse_activity(ctx: &Ctx, idx: usize, act: &mut Activity) {
    let mut child = idx + 1;
    for _ in 0..ctx.toks[idx].size {
        set_field(ctx, child, act);
        child = ctx.next_sibling(child + 1);
    }
    if act.name.is_empty() {
        act.name = String::new();
        let _ = act.name.push_str(act.record.as_str());
        act.record.clear();
    }
}

pub fn parse_activities(json_bytes: &[u8], list: &mut ActivityList) -> Result<(), ParseErr> {
    let mut toks = [Token::EMPTY; MAX_JSON_TOKENS];
    *list = ActivityList::default();
    let count = json::parse(json_bytes, &mut toks).map_err(|()| ParseErr::Malformed)?;
    let ctx = Ctx {
        json: json_bytes,
        toks: &toks[..count],
    };
    if ctx.find_key("error").is_some() {
        return Err(ParseErr::Odoo);
    }
    let val = ctx.find_key("result").ok_or(ParseErr::Malformed)?;
    if ctx.toks[val].kind != TokKind::Array {
        return Err(ParseErr::Malformed);
    }
    let total = ctx.toks[val].size;
    let mut child = val + 1;
    for n in 0..total {
        if n < MAX_ACTIVITIES {
            parse_activity(&ctx, child, &mut list.items[n]);
        }
        child = ctx.next_sibling(child);
    }
    list.count = total.min(MAX_ACTIVITIES) as u32;
    Ok(())
}
