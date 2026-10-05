//! JSON tokenizer (jsmn port) and string folding to ASCII
//! (port of `src/json/jsmn_util.c` and `src/json/json_str.c`).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokKind {
    Object,
    Array,
    Str,
    Primitive,
}

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub kind: TokKind,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}

impl Token {
    pub const EMPTY: Token = Token {
        kind: TokKind::Primitive,
        start: 0,
        end: 0,
        size: 0,
    };
}

const OPEN: usize = usize::MAX;

/// # Errors
/// Returns `Err(())` if `json` is malformed (unbalanced braces/brackets,
/// an unterminated string) or if `toks` is too small to hold every token.
// Direct port of jsmn's single state-machine loop (jsmn_util.c); splitting
// it into helpers would obscure the C parity for a lint about line count,
// not correctness.
#[allow(clippy::result_unit_err, clippy::too_many_lines)]
pub fn parse(json: &[u8], toks: &mut [Token]) -> Result<usize, ()> {
    let mut count = 0usize;
    let mut toksuper: Option<usize> = None;
    let mut pos = 0usize;
    while pos < json.len() {
        let c = json[pos];
        match c {
            b'{' | b'[' => {
                if count >= toks.len() {
                    return Err(());
                }
                if let Some(s) = toksuper {
                    toks[s].size += 1;
                }
                let kind = if c == b'{' {
                    TokKind::Object
                } else {
                    TokKind::Array
                };
                toks[count] = Token {
                    kind,
                    start: pos,
                    end: OPEN,
                    size: 0,
                };
                toksuper = Some(count);
                count += 1;
            }
            b'}' | b']' => {
                let kind = if c == b'}' {
                    TokKind::Object
                } else {
                    TokKind::Array
                };
                let mut i = count;
                loop {
                    if i == 0 {
                        return Err(());
                    }
                    i -= 1;
                    let open = matches!(toks[i].kind, TokKind::Object | TokKind::Array)
                        && toks[i].end == OPEN;
                    if open {
                        if toks[i].kind != kind {
                            return Err(());
                        }
                        toks[i].end = pos + 1;
                        break;
                    }
                }
                toksuper = toks[..i].iter().rposition(|t| {
                    matches!(t.kind, TokKind::Object | TokKind::Array) && t.end == OPEN
                });
            }
            b'"' => {
                let start = pos + 1;
                let mut p = start;
                let mut closed = false;
                while p < json.len() {
                    if json[p] == b'"' {
                        closed = true;
                        break;
                    }
                    if json[p] == b'\\' {
                        p += 1;
                    }
                    p += 1;
                }
                if !closed || count >= toks.len() {
                    return Err(());
                }
                toks[count] = Token {
                    kind: TokKind::Str,
                    start,
                    end: p,
                    size: 0,
                };
                if let Some(s) = toksuper {
                    toks[s].size += 1;
                }
                count += 1;
                pos = p;
            }
            b':' => toksuper = count.checked_sub(1),
            b',' => {
                if let Some(s) = toksuper {
                    if !matches!(toks[s].kind, TokKind::Object | TokKind::Array) {
                        toksuper = toks[..count].iter().rposition(|t| {
                            matches!(t.kind, TokKind::Object | TokKind::Array) && t.end == OPEN
                        });
                    }
                }
            }
            b'\t' | b'\r' | b'\n' | b' ' => {}
            _ => {
                let start = pos;
                let mut p = pos;
                while p < json.len()
                    && !matches!(
                        json[p],
                        b'\t' | b'\r' | b'\n' | b' ' | b',' | b']' | b'}' | b':'
                    )
                {
                    p += 1;
                }
                if count >= toks.len() {
                    return Err(());
                }
                toks[count] = Token {
                    kind: TokKind::Primitive,
                    start,
                    end: p,
                    size: 0,
                };
                if let Some(s) = toksuper {
                    toks[s].size += 1;
                }
                count += 1;
                pos = p - 1;
            }
        }
        pos += 1;
    }
    if count == 0 {
        return Err(());
    }
    if toks[..count]
        .iter()
        .any(|t| matches!(t.kind, TokKind::Object | TokKind::Array) && t.end == OPEN)
    {
        return Err(());
    }
    Ok(count)
}

pub struct Ctx<'a> {
    pub json: &'a [u8],
    pub toks: &'a [Token],
}

impl Ctx<'_> {
    #[must_use]
    pub fn bytes(&self, idx: usize) -> &[u8] {
        &self.json[self.toks[idx].start..self.toks[idx].end]
    }

    #[must_use]
    pub fn tok_eq(&self, idx: usize, s: &str) -> bool {
        self.toks[idx].kind == TokKind::Str && self.bytes(idx) == s.as_bytes()
    }

    #[must_use]
    pub fn next_sibling(&self, idx: usize) -> usize {
        let end = self.toks[idx].end;
        let mut i = idx + 1;
        while i < self.toks.len() && self.toks[i].start < end {
            i += 1;
        }
        i
    }

    #[must_use]
    pub fn find_key(&self, key: &str) -> Option<usize> {
        if self.toks.is_empty() || self.toks[0].kind != TokKind::Object {
            return None;
        }
        let mut i = 1;
        while i + 1 < self.toks.len() {
            if self.tok_eq(i, key) {
                return Some(i + 1);
            }
            i = self.next_sibling(i + 1);
        }
        None
    }
}

const FOLD_TABLE: &[(u32, &str)] = &[
    (0xe0, "a"),
    (0xe2, "a"),
    (0xe4, "a"),
    (0xe7, "c"),
    (0xe8, "e"),
    (0xe9, "e"),
    (0xea, "e"),
    (0xeb, "e"),
    (0xee, "i"),
    (0xef, "i"),
    (0xf4, "o"),
    (0xf6, "o"),
    (0xf9, "u"),
    (0xfb, "u"),
    (0xfc, "u"),
    (0xc0, "A"),
    (0xc2, "A"),
    (0xc7, "C"),
    (0xc8, "E"),
    (0xc9, "E"),
    (0xca, "E"),
    (0xcb, "E"),
    (0xce, "I"),
    (0xcf, "I"),
    (0xd4, "O"),
    (0xd9, "U"),
    (0xdb, "U"),
    (0x153, "oe"),
    (0x152, "OE"),
    (0xe6, "ae"),
    (0xc6, "AE"),
    (0x2019, "'"),
    (0x2013, "-"),
    (0x2014, "-"),
    (0x2026, "..."),
];

fn hex_val(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some(u32::from(c - b'0')),
        b'a'..=b'f' => Some(u32::from(c - b'a' + 10)),
        b'A'..=b'F' => Some(u32::from(c - b'A' + 10)),
        _ => None,
    }
}

fn decode_escape(src: &[u8]) -> (usize, u32) {
    if src.is_empty() {
        return (0, u32::from(b'?'));
    }
    if src[0] == b'u' {
        if src.len() >= 5 {
            let mut value = 0u32;
            for &b in &src[1..5] {
                match hex_val(b) {
                    Some(v) => value = (value << 4) | v,
                    None => return (src.len().min(5), u32::from(b'?')),
                }
            }
            return (5, value);
        }
        return (src.len().min(5), u32::from(b'?'));
    }
    match src[0] {
        b'n' | b'r' | b't' => (1, u32::from(b' ')),
        b'"' | b'\\' | b'/' => (1, u32::from(src[0])),
        _ => (1, u32::from(b'?')),
    }
}

fn utf8_len(lead: u8) -> usize {
    if lead < 0x80 {
        1
    } else if lead & 0xe0 == 0xc0 {
        2
    } else if lead & 0xf0 == 0xe0 {
        3
    } else if lead & 0xf8 == 0xf0 {
        4
    } else {
        0
    }
}

fn decode_utf8(src: &[u8]) -> (usize, u32) {
    let lead = src[0];
    if lead < 0x80 {
        return (1, u32::from(lead));
    }
    let need = utf8_len(lead);
    if need == 0 || need > src.len() {
        return (1, u32::from(b'?'));
    }
    let mut value = u32::from(lead) & (0xff >> (need + 1));
    for &b in &src[1..need] {
        if b & 0xc0 != 0x80 {
            return (1, u32::from(b'?'));
        }
        value = (value << 6) | (u32::from(b) & 0x3f);
    }
    (need, value)
}

struct Out<'a> {
    dst: &'a mut [u8],
    pos: usize,
}

impl Out<'_> {
    fn push(&mut self, b: u8) {
        if self.pos + 1 < self.dst.len() {
            self.dst[self.pos] = b;
            self.pos += 1;
        } else if self.dst.len() > 1 {
            self.dst[self.dst.len() - 2] = b'.';
        }
    }

    fn push_cp(&mut self, cp: u32) {
        if (0x20..0x7f).contains(&cp) {
            // cp is range-checked to ASCII just above; the truncation is
            // provably safe (C-parity arithmetic, as in json_str.c).
            #[allow(clippy::cast_possible_truncation)]
            self.push(cp as u8);
            return;
        }
        if let Some((_, s)) = FOLD_TABLE.iter().find(|(c, _)| *c == cp) {
            for &b in s.as_bytes() {
                self.push(b);
            }
            return;
        }
        self.push(if cp < 0x20 { b' ' } else { b'?' });
    }
}

pub fn fold_into(dst: &mut [u8], src: &[u8]) -> usize {
    if dst.is_empty() {
        return 0;
    }
    let mut out = Out { dst, pos: 0 };
    let mut i = 0usize;
    while i < src.len() {
        let (consumed, cp) = if src[i] == b'\\' {
            let (c, cp) = decode_escape(&src[i + 1..]);
            (c + 1, cp)
        } else {
            decode_utf8(&src[i..])
        };
        i += consumed.max(1);
        out.push_cp(cp);
    }
    out.pos
}
