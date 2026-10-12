use dashboard_core::odoo::{self, Config};

const CFG: Config = Config {
    db: "testdb",
    login: "tester@test.lan",
    api_key: "test-key",
    domain: "[[\"user_id\",\"=\",%d]]",
};

fn s(buf: &[u8], len: usize) -> &str {
    std::str::from_utf8(&buf[..len]).unwrap()
}

#[test]
fn auth_body() {
    let mut buf = [0u8; 512];
    let len = odoo::build_auth(&mut buf, &CFG).unwrap();
    let body = s(&buf, len);
    assert!(body.contains("\"service\":\"common\""));
    assert!(body.contains("\"authenticate\""));
    assert!(body.contains("\"testdb\""));
    assert!(body.contains("\"tester@test.lan\""));
    assert!(body.contains("\"test-key\""));
}

#[test]
fn activities_body() {
    let mut buf = [0u8; 2048];
    let len = odoo::build_activities(&mut buf, &CFG, 42).unwrap();
    let body = s(&buf, len);
    assert!(body.contains("\"execute_kw\""));
    assert!(body.contains("\"mail.activity\""));
    assert!(body.contains("\"search_read\""));
    assert!(body.contains("[[\"user_id\",\"=\",42]]"));
    assert!(body.contains("\"res_name\""));
    assert!(body.contains("\"summary\""));
    assert!(body.contains("\"icon\""));
    assert!(body.contains("\"limit\":7"));
    assert!(body.contains("\"order\":\"date_deadline asc\""));
}

#[test]
fn count_body() {
    let mut buf = [0u8; 2048];
    let len = odoo::build_count(&mut buf, &CFG, 42).unwrap();
    let body = s(&buf, len);
    assert!(body.contains("\"search_count\""));
    assert!(body.contains("[[\"user_id\",\"=\",42]]"));
    assert!(body.contains("\"id\":3"));
    assert!(!body.contains("\"limit\""));
}

#[test]
fn truncation() {
    let mut buf = [0u8; 32];
    assert!(odoo::build_auth(&mut buf, &CFG).is_none());
    assert!(odoo::build_activities(&mut buf, &CFG, 1).is_none());
    assert!(odoo::build_count(&mut buf, &CFG, 1).is_none());
}
