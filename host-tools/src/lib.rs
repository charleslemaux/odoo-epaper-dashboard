//! Host-side sample-frame rendering shared by the preview bin and parity test.

use dashboard_core::dashboard::{self, DashboardData};
use dashboard_core::gfx::Frame;
use dashboard_core::odoo::Activity;
use dashboard_core::refresh::Snapshot;
use dashboard_core::time_fmt::Date;

fn act(name: &str, record: &str, deadline: &str, icon: &str) -> Activity {
    let mut a = Activity::default();
    let _ = a.name.push_str(name);
    let _ = a.record.push_str(record);
    let _ = a.deadline.push_str(deadline);
    let _ = a.icon.push_str(icon);
    a
}

pub fn sample_snapshot() -> Snapshot {
    let mut snap = Snapshot::default();
    snap.list.count = 6;
    snap.list.total = 11;
    snap.list.items[0] = act(
        "Rappeler pour le devis",
        "Jean Dupont",
        "2026-08-25",
        "fa-phone",
    );
    snap.list.items[1] = act(
        "Relire et valider la proposition commerciale complete",
        "Societe Martin & Fils",
        "2026-08-27",
        "fa-code",
    );
    snap.list.items[2] = act("Facture F0042", "", "2026-08-19", "fa-cube");
    snap.list.items[3] = act(
        "Envoyer le contrat signe",
        "Dossier 2318",
        "2026-09-02",
        "fa-microchip",
    );
    snap.list.items[4] = act("Point hebdo equipe", "", "2026-08-28", "fa-lightbulb-o");
    snap.list.items[5] = act(
        "Preparer la demo",
        "Projet Alpha",
        "2026-09-05",
        "fa-calendar-check-o",
    );
    snap
}

pub fn render_sample(fb: &mut Frame) {
    let snap = sample_snapshot();
    let d = DashboardData {
        snap: &snap,
        today: Date {
            year: 2026,
            month: 8,
            day: 27,
        },
        banner_date: "jeu 27/08",
        updated_hhmm: "03:15",
        offline_since: "",
    };
    dashboard::render(fb, &d);
}
