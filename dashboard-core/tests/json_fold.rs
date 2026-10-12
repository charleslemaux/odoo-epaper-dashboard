use dashboard_core::json::{self, Token};

fn fold(src: &[u8], cap: usize) -> String {
    let mut buf = vec![0u8; cap];
    let len = json::fold_into(&mut buf, src);
    String::from_utf8(buf[..len].to_vec()).unwrap()
}

#[test]
fn plain_ascii() {
    assert_eq!(fold(b"hello", 16), "hello");
}

#[test]
fn unicode_escape() {
    assert_eq!(fold(br"caf\u00e9", 16), "cafe");
}

#[test]
fn raw_utf8() {
    assert_eq!(fold(b"d\xc3\xa9j\xc3\xa0 vu", 16), "deja vu");
}

#[test]
fn escapes() {
    assert_eq!(fold(br#"a\"b\\c\nd"#, 16), "a\"b\\c d");
}

#[test]
fn truncation() {
    assert_eq!(fold(b"abcdefgh", 6), "abcd.");
}

#[test]
fn oe_ligature() {
    assert_eq!(fold(b"c\xc5\x93ur", 16), "coeur");
}

#[test]
fn tokenizer_object_shape() {
    let src = br#"{"a":1,"b":[true,{"c":"x"}]}"#;
    let mut toks = [Token::EMPTY; 32];
    let count = json::parse(src, &mut toks).unwrap();
    let ctx = json::Ctx {
        json: src,
        toks: &toks[..count],
    };
    assert!(matches!(toks[0].kind, json::TokKind::Object));
    assert_eq!(toks[0].size, 2);
    let b = ctx.find_key("b").unwrap();
    assert!(matches!(toks[b].kind, json::TokKind::Array));
    assert_eq!(toks[b].size, 2);
    assert_eq!(ctx.next_sibling(b), count);
    assert!(ctx.find_key("missing").is_none());
}

#[test]
fn tokenizer_rejects_unclosed() {
    let mut toks = [Token::EMPTY; 8];
    assert!(json::parse(br#"{"a":1"#, &mut toks).is_err());
}
