use dashboard_core::odoo::{self, ActivityList, ParseErr};

const AUTH: &[u8] = br#"{"jsonrpc":"2.0","id":1,"result":42}"#;
const AUTH_FAIL: &[u8] = br#"{"jsonrpc":"2.0","id":1,"result":false}"#;
const COUNT: &[u8] = br#"{"jsonrpc":"2.0","id":3,"result":23}"#;
const ERROR: &[u8] =
    br#"{"jsonrpc":"2.0","id":2,"error":{"code":200,"message":"Odoo Server Error"}}"#;
const ACTS: &[u8] = br#"{"jsonrpc":"2.0","id":2,"result":[
 {"id":7,"res_name":"Jean Dupont","summary":"Rappeler pour le devis",
  "icon":"fa-phone","date_deadline":"2026-08-30"},
 {"id":8,"res_name":"Facture F0042","summary":false,
  "icon":"fa-code","date_deadline":"2026-09-01"}]}"#;

#[test]
fn auth() {
    assert_eq!(odoo::parse_auth(AUTH), Ok(42));
    assert!(odoo::parse_auth(AUTH_FAIL).is_err());
    assert!(odoo::parse_auth(b"garbage").is_err());
}

#[test]
fn count() {
    assert_eq!(odoo::parse_count(COUNT), Ok(23));
    assert!(odoo::parse_count(AUTH_FAIL).is_err());
}

#[test]
fn activities_nominal() {
    let mut list = ActivityList::default();
    odoo::parse_activities(ACTS, &mut list).unwrap();
    assert_eq!(list.count, 2);
    assert_eq!(list.total, 0);
    assert_eq!(list.items[0].name.as_str(), "Rappeler pour le devis");
    assert_eq!(list.items[0].record.as_str(), "Jean Dupont");
    assert_eq!(list.items[0].icon.as_str(), "fa-phone");
    assert_eq!(list.items[0].deadline.as_str(), "2026-08-30");
}

#[test]
fn summary_fallback() {
    let mut list = ActivityList::default();
    odoo::parse_activities(ACTS, &mut list).unwrap();
    assert_eq!(list.items[1].name.as_str(), "Facture F0042");
    assert_eq!(list.items[1].record.as_str(), "");
    assert_eq!(list.items[1].icon.as_str(), "fa-code");
    assert_eq!(list.items[1].deadline.as_str(), "2026-09-01");
}

#[test]
fn error_and_garbage() {
    let mut list = ActivityList::default();
    assert_eq!(
        odoo::parse_activities(ERROR, &mut list),
        Err(ParseErr::Odoo)
    );
    assert_eq!(
        odoo::parse_activities(b"not json at all", &mut list),
        Err(ParseErr::Malformed)
    );
}

#[test]
fn overflow_caps_at_seven() {
    let mut json = String::from("{\"result\":[");
    for i in 0..8 {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            "{{\"id\":{i},\"res_name\":\"T{i}\",\"date_deadline\":\"2026-09-01\"}}"
        ));
    }
    json.push_str("]}");
    let mut list = ActivityList::default();
    odoo::parse_activities(json.as_bytes(), &mut list).unwrap();
    assert_eq!(list.count, 7);
    assert_eq!(list.items[6].name.as_str(), "T6");
}
