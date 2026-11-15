//! Rapport HTML autonome (export), imprimable et partageable.
use crate::model::*;
use crate::util::fmt_bytes;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn gain_text(g: &Gain) -> String {
    if g.bytes == 0 || g.nature == GainNature::Unavailable {
        "non chiffrable".to_string()
    } else {
        format!("{} ({})", fmt_bytes(g.bytes), g.nature.label())
    }
}

/// La provenance n'est affichée que lorsqu'elle existe : un conseil de repli
/// sans règle nommée ne doit pas hériter d'une fausse source.
fn provenance_text(a: &Advice) -> String {
    let source = a.source.trim();
    let reviewed = a.reviewed.trim();
    match (source.is_empty(), reviewed.is_empty()) {
        (true, true) => "—".to_string(),
        (false, true) => source.to_string(),
        (true, false) => format!("revue : {reviewed}"),
        (false, false) => format!("{source} · revue : {reviewed}"),
    }
}

fn badge(a: &Advice) -> String {
    let cls = match a.level {
        Level::Critical => "crit",
        Level::System => "sys",
        Level::Keep => "keep",
        Level::Optional => "opt",
        Level::Trim => "trim",
        Level::Junk => "junk",
        Level::Unknown => "unk",
    };
    format!("<span class=\"b {cls}\">{}</span>", esc(&a.label))
}

pub fn build(s: &Snapshot) -> String {
    let mut h = String::new();
    h.push_str(&format!(
        r#"<!doctype html><html lang="fr"><head><meta charset="utf-8">
<title>RamScope — rapport de {}</title><style>
:root{{--ink:#12161c;--mut:#5b6673;--line:#e2e6ec}}
*{{box-sizing:border-box}}
body{{font:14px/1.5 -apple-system,Segoe UI,Roboto,Helvetica,Arial,sans-serif;color:var(--ink);margin:0;padding:32px;background:#f7f8fa}}
h1{{font-size:24px;margin:0 0 4px}} h2{{font-size:17px;margin:32px 0 10px;padding-bottom:6px;border-bottom:2px solid var(--line)}}
.sub{{color:var(--mut);margin-bottom:20px}}
.cards{{display:flex;gap:12px;flex-wrap:wrap;margin:16px 0}}
.card{{background:#fff;border:1px solid var(--line);border-radius:10px;padding:14px 18px;min-width:150px}}
.card .v{{font-size:22px;font-weight:700}} .card .k{{color:var(--mut);font-size:12px;text-transform:uppercase;letter-spacing:.04em}}
table{{width:100%;border-collapse:collapse;background:#fff;border:1px solid var(--line);border-radius:8px;overflow:hidden;margin-bottom:8px}}
th{{text-align:left;font-size:11px;text-transform:uppercase;letter-spacing:.04em;color:var(--mut);padding:8px 10px;background:#f0f2f6;border-bottom:1px solid var(--line)}}
td{{padding:8px 10px;border-bottom:1px solid #eef1f5;vertical-align:top}}
tr:last-child td{{border-bottom:none}}
.num{{text-align:right;white-space:nowrap;font-variant-numeric:tabular-nums}}
.b{{display:inline-block;padding:2px 8px;border-radius:20px;font-size:11px;font-weight:600;white-space:nowrap}}
.crit{{background:#fde8e8;color:#a11}} .sys{{background:#eceff4;color:#4a5568}} .keep{{background:#e6f6ec;color:#1a7f45}}
.opt{{background:#fff4e0;color:#96650a}} .trim{{background:#ffe9d6;color:#a04a00}} .junk{{background:#f3e8ff;color:#6b21a8}} .unk{{background:#eef1f5;color:#5b6673}}
.why{{color:var(--mut);font-size:12.5px}}
.note{{background:#fff8e1;border:1px solid #f0dca0;border-radius:8px;padding:10px 14px;margin:10px 0;font-size:13px}}
footer{{margin-top:36px;color:var(--mut);font-size:12px}}
code{{background:#f0f2f6;padding:1px 5px;border-radius:4px;font-size:12px;word-break:break-all}}
@media print{{body{{background:#fff;padding:0}} table{{page-break-inside:auto}} tr{{page-break-inside:avoid}}}}
</style></head><body>
<h1>RamScope — rapport mémoire &amp; démarrage</h1>
<div class="sub">{} · {} · {} · analysé le {}{}</div>"#,
        esc(&s.meta.computer),
        esc(&s.meta.computer),
        esc(&s.meta.os),
        esc(&s.meta.user),
        esc(&s.meta.collected_at),
        if s.meta.demo { " · <b>MODE DÉMONSTRATION</b>" } else { "" }
    ));

    h.push_str(&format!(
        r#"<div class="cards">
<div class="card"><div class="k">RAM totale</div><div class="v">{}</div></div>
<div class="card"><div class="k">Utilisée</div><div class="v">{} ({}%)</div></div>
<div class="card"><div class="k">Disponible</div><div class="v">{}</div></div>
<div class="card"><div class="k">Gain immédiat</div><div class="v">{}</div></div>
<div class="card"><div class="k">Gain au prochain démarrage</div><div class="v">{}</div></div>
</div>"#,
        fmt_bytes(s.memory.total),
        fmt_bytes(s.memory.used),
        s.memory.load,
        fmt_bytes(s.memory.available),
        gain_text(&s.plan_gain),
        gain_text(&s.plan_gain_next_boot)
    ));

    for n in &s.meta.notes {
        h.push_str(&format!("<div class=\"note\">{}</div>", esc(n)));
    }

    // --- plan ---
    h.push_str("<h2>Plan d'optimisation recommandé</h2><table><tr><th>Élément</th><th>Verdict</th><th>Action conseillée</th><th class=\"num\">Tout de suite</th><th class=\"num\">Au prochain démarrage</th><th>Pourquoi</th><th>Provenance</th></tr>");
    let mut rows = 0;
    for svc in &s.services {
        if let Some(a) = &svc.advice.action {
            if svc.advice.level.preselected() {
                rows += 1;
                h.push_str(&format!(
                    "<tr><td><b>{}</b><br><code>{}</code></td><td>{}</td><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"why\">{}</td><td class=\"why\">{}</td></tr>",
                    esc(&svc.display), esc(&svc.name), badge(&svc.advice), esc(&a.describe()),
                    gain_text(&svc.advice.gain), gain_text(&svc.advice.gain_next_boot), esc(&svc.advice.reason),
                    esc(&provenance_text(&svc.advice))
                ));
            }
        }
    }
    for st in &s.startup {
        if let Some(a) = &st.advice.action {
            if st.advice.level.preselected() {
                rows += 1;
                h.push_str(&format!(
                    "<tr><td><b>{}</b><br><code>{}</code></td><td>{}</td><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"why\">{}</td><td class=\"why\">{}</td></tr>",
                    esc(&st.name), esc(&st.command), badge(&st.advice), esc(&a.describe()),
                    gain_text(&st.advice.gain), gain_text(&st.advice.gain_next_boot), esc(&st.advice.reason),
                    esc(&provenance_text(&st.advice))
                ));
            }
        }
    }
    for t in &s.tasks {
        if let Some(a) = &t.advice.action {
            if t.advice.level.preselected() {
                rows += 1;
                h.push_str(&format!(
                    "<tr><td><b>{}</b><br><code>{}</code></td><td>{}</td><td>{}</td><td class=\"num\">—</td><td class=\"num\">—</td><td class=\"why\">{}</td><td class=\"why\">{}</td></tr>",
                    esc(&t.name), esc(&t.path), badge(&t.advice), esc(&a.describe()), esc(&t.advice.reason),
                    esc(&provenance_text(&t.advice))
                ));
            }
        }
    }
    if rows == 0 {
        h.push_str("<tr><td colspan=\"7\">Rien à optimiser : configuration déjà saine.</td></tr>");
    }
    h.push_str("</table>");

    // --- processus ---
    h.push_str("<h2>Processus en mémoire</h2><table><tr><th>Processus</th><th class=\"num\">PID</th><th class=\"num\">RAM privée</th><th class=\"num\">RAM totale</th><th>Verdict</th><th>Pourquoi</th><th>Provenance</th></tr>");
    for p in &s.processes {
        h.push_str(&format!(
            "<tr><td><b>{}</b><br><span class=\"why\">{}</span></td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td>{}</td><td class=\"why\">{}</td><td class=\"why\">{}</td></tr>",
            esc(&p.name), esc(&p.description), p.pid, fmt_bytes(p.private), fmt_bytes(p.working_set),
            badge(&p.advice), esc(&p.advice.reason), esc(&provenance_text(&p.advice))
        ));
    }
    h.push_str("</table>");

    // --- services ---
    h.push_str("<h2>Services</h2><table><tr><th>Service</th><th>État</th><th>Démarrage</th><th class=\"num\">RAM</th><th>Verdict</th><th>Pourquoi</th><th>Provenance</th></tr>");
    for v in &s.services {
        h.push_str(&format!(
            "<tr><td><b>{}</b><br><code>{}</code></td><td>{}</td><td>{}</td><td class=\"num\">{}</td><td>{}</td><td class=\"why\">{}</td><td class=\"why\">{}</td></tr>",
            esc(&v.display), esc(&v.name), esc(&v.state), esc(v.start.label()),
            fmt_bytes(v.ram), badge(&v.advice), esc(&v.advice.reason), esc(&provenance_text(&v.advice))
        ));
    }
    h.push_str("</table>");

    // --- démarrage ---
    h.push_str("<h2>Démarrage automatique</h2><table><tr><th>Entrée</th><th>Emplacement</th><th>État</th><th>Verdict</th><th>Pourquoi</th><th>Provenance</th></tr>");
    for v in &s.startup {
        h.push_str(&format!(
            "<tr><td><b>{}</b><br><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td class=\"why\">{}</td><td class=\"why\">{}</td></tr>",
            esc(&v.name), esc(&v.command), esc(&v.location),
            if v.enabled { "Activé" } else { "Désactivé" },
            badge(&v.advice), esc(&v.advice.reason), esc(&provenance_text(&v.advice))
        ));
    }
    h.push_str("</table>");

    // --- tâches ---
    h.push_str("<h2>Tâches planifiées</h2><table><tr><th>Tâche</th><th>Déclencheurs</th><th>État</th><th>Verdict</th><th>Pourquoi</th><th>Provenance</th></tr>");
    for v in &s.tasks {
        h.push_str(&format!(
            "<tr><td><b>{}</b><br><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td class=\"why\">{}</td><td class=\"why\">{}</td></tr>",
            esc(&v.name), esc(&v.path), esc(&v.triggers.join(", ")),
            if v.enabled { "Activée" } else { "Désactivée" },
            badge(&v.advice), esc(&v.advice.reason), esc(&provenance_text(&v.advice))
        ));
    }
    h.push_str("</table>");

    h.push_str(&format!(
        "<footer>Généré par RamScope {} · {} processus, {} services, {} entrées de démarrage, {} tâches.<br>
Les gains sont des estimations : la RAM d'un service svchost est partagée entre plusieurs services, et un service passé en manuel peut redémarrer à la demande. Un hôte partagé ne compte que si tous ses services sont inclus dans le lot.</footer></body></html>",
        esc(&s.meta.version), s.counts.processes, s.counts.services, s.startup.len(), s.tasks.len()
    ));
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separe_les_gains_et_affiche_leur_nature() {
        let s = crate::demo::DemoState::default().snapshot();
        let h = build(&s);
        assert!(h.contains("Tout de suite"));
        assert!(h.contains("Au prochain démarrage"));
        assert!(h.contains("estimé") || h.contains("non chiffrable"));
        let interdit = format!("Rien à optimiser {} risque", "sans");
        assert!(!h.contains(&interdit));
        assert!(!h.contains("Gain</th>"));
    }

    #[test]
    fn le_rapport_expose_la_provenance_des_regles() {
        let s = crate::demo::DemoState::default().snapshot();
        let h = build(&s);
        assert!(
            h.contains("Table RamScope héritée"),
            "source absente du rapport"
        );
        assert!(h.contains("non datée"), "date de revue absente du rapport");
        assert!(h.contains("Provenance"), "colonne de provenance absente");
    }
}
