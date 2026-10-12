use dashboard_core::activity_icon::icon_for;
use dashboard_core::icon_data::Icon;

#[test]
fn known_classes() {
    assert_eq!(icon_for("fa-check"), Icon::Check);
    assert_eq!(icon_for("fa-envelope"), Icon::Envelope);
    assert_eq!(icon_for("fa-phone"), Icon::Phone);
    assert_eq!(icon_for("fa-users"), Icon::Users);
    assert_eq!(icon_for("fa-code"), Icon::Code);
    assert_eq!(icon_for("fa-cube"), Icon::Cube);
    assert_eq!(icon_for("fa-microchip"), Icon::Microchip);
    assert_eq!(icon_for("fa-lightbulb-o"), Icon::Lightbulb);
    assert_eq!(icon_for("fa-calendar-check-o"), Icon::CalendarCheck);
    assert_eq!(icon_for("fa-upload"), Icon::Upload);
    assert_eq!(icon_for("fa-pencil-square-o"), Icon::PenSquare);
}

#[test]
fn unknown_falls_back() {
    assert_eq!(icon_for("fa-rocket"), Icon::Clock);
    assert_eq!(icon_for(""), Icon::Clock);
    assert_eq!(icon_for("fa-che"), Icon::Clock);
}
