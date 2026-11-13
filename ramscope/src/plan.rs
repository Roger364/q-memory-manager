//! Logique pure, indépendante de Windows.
//!
//! Ce module rassemble ce qui doit rester testable partout : validation d'une
//! liste d'actions avant toute modification système, agrégation des gains
//! d'un lot, normalisation et correspondance des règles.
//!
//! Contrainte : aucun appel système, aucune dépendance à la couche FFI
//! Windows ni au module `win`. Le module doit compiler sur toutes les cibles.

use crate::model::Action;
use crate::rules;

// ---------------------------------------------------------------------------
// Validation des actions
// ---------------------------------------------------------------------------

/// Résultat de la validation d'une action, avant application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub allowed: bool,
    pub message: String,
}

impl Verdict {
    fn allow() -> Verdict {
        Verdict {
            allowed: true,
            message: String::new(),
        }
    }

    fn refuse(message: &str) -> Verdict {
        Verdict {
            allowed: false,
            message: message.to_string(),
        }
    }
}

/// Valide une liste d'actions dans l'ordre, sans toucher au système.
///
/// `self_pid` est le PID de RamScope : le programme ne peut pas se terminer
/// lui-même. Les garde-fous de `win.rs` restent en place en second verrou.
pub fn validate(actions: &[Action], self_pid: u32) -> Vec<Verdict> {
    actions.iter().map(|a| validate_one(a, self_pid)).collect()
}

fn validate_one(action: &Action, self_pid: u32) -> Verdict {
    match action {
        Action::Service { name, .. } if rules::is_protected_service(name) => {
            Verdict::refuse("Service vital : RamScope refuse de le modifier.")
        }
        Action::StopService { name } if rules::is_protected_service(name) => {
            Verdict::refuse("Service vital : RamScope refuse de l'arrêter.")
        }
        Action::Kill { pid, name } if rules::is_protected_process(name) || *pid <= 8 => {
            Verdict::refuse("Processus vital : RamScope refuse de le terminer.")
        }
        Action::Kill { pid, .. } if *pid == self_pid => {
            Verdict::refuse("RamScope ne peut pas se terminer lui-même")
        }
        _ => Verdict::allow(),
    }
}

// ---------------------------------------------------------------------------
// Gains d'un lot
// ---------------------------------------------------------------------------

/// Totalise les gains d'un lot : chaque action distincte n'est comptée qu'une
/// fois, même si elle apparaît deux fois dans la liste.
///
/// `entries` associe une action à son gain immédiat et à son gain au prochain
/// démarrage. La non-additivité entre services d'un même hôte relève de
/// l'étape 3 du PLAN.md : cette fonction ne fait que dédoublonner.
pub fn total_gains(entries: &[(Action, u64, u64)]) -> (u64, u64) {
    let mut now = 0u64;
    let mut boot = 0u64;
    let mut seen: Vec<&Action> = Vec::new();
    for (action, gain_now, gain_boot) in entries {
        if seen.iter().any(|s| *s == action) {
            continue;
        }
        seen.push(action);
        now += gain_now;
        boot += gain_boot;
    }
    (now, boot)
}

// ---------------------------------------------------------------------------
// Correspondance des règles
// ---------------------------------------------------------------------------

/// Normalise une chaîne pour comparaison : minuscules uniquement.
/// Le rognage et le repliement des espaces relèvent de l'étape 4 du PLAN.md.
pub fn normalize(s: &str) -> String {
    s.to_lowercase()
}

/// Vrai si `key` apparaît dans l'un des champs.
///
/// Comportement conservé à l'étape 0, volontairement resserré à l'étape 4
/// (défaut D9) : la correspondance par sous-chaîne sur une ligne de commande
/// entière n'est pas fiable pour les clés courtes.
pub fn key_matches(key: &str, fields: &[&str]) -> bool {
    fields.iter().any(|f| f.contains(key))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StartType;

    fn verifie_cles(nom: &str, cles: Vec<&'static str>) {
        let mut vues: Vec<&str> = Vec::new();
        for c in cles {
            assert!(!c.is_empty(), "{nom} : clé vide");
            assert_eq!(c.trim(), c, "{nom} : clé « {c} » entourée d'espaces");
            assert!(!vues.contains(&c), "{nom} : clé « {c} » en double");
            vues.push(c);
        }
    }

    #[test]
    fn les_cles_des_tables_sont_uniques_et_non_vides() {
        verifie_cles("SERVICES", rules::SERVICES.iter().map(|r| r.0).collect());
        verifie_cles("PROCESSES", rules::PROCESSES.iter().map(|r| r.0).collect());
        verifie_cles("STARTUP", rules::STARTUP.iter().map(|r| r.0).collect());
        verifie_cles("TASKS", rules::TASKS.iter().map(|r| r.0).collect());
    }

    #[test]
    fn refuse_les_cibles_protegees() {
        let v = validate(
            &[
                Action::Service {
                    name: "RpcSs".into(),
                    start: StartType::Manual,
                },
                Action::StopService {
                    name: "WinDefend".into(),
                },
                Action::Kill {
                    pid: 4,
                    name: "System".into(),
                },
                Action::Kill {
                    pid: 1234,
                    name: "ramscope.exe".into(),
                },
            ],
            1234,
        );
        assert!(!v[0].allowed && v[0].message.contains("modifier"));
        assert!(!v[1].allowed && v[1].message.contains("arrêter"));
        assert!(!v[2].allowed && v[2].message.contains("terminer"));
        assert!(!v[3].allowed && v[3].message.contains("lui-même"));
    }

    #[test]
    fn accepte_une_action_ordinaire() {
        let v = validate(
            &[Action::Service {
                name: "DiagTrack".into(),
                start: StartType::Disabled,
            }],
            1234,
        );
        assert!(v[0].allowed);
        assert!(v[0].message.is_empty());
    }

    #[test]
    fn ne_compte_qu_une_fois_la_meme_action() {
        let a = Action::Service {
            name: "DiagTrack".into(),
            start: StartType::Disabled,
        };
        let b = Action::Startup {
            id: "hkcu_run|Spotify".into(),
            enabled: false,
        };
        let entries = vec![(a.clone(), 0, 10), (a, 0, 10), (b, 0, 5)];
        assert_eq!(total_gains(&entries), (0, 15));
    }

    #[test]
    fn normalise_le_nom_avant_comparaison() {
        assert_eq!(normalize("DiagTrack"), "diagtrack");
        assert!(key_matches("diagtrack", &["diagtrack"]));
        assert!(!key_matches("diagtrack", &["spooler", "x"]));
    }
}
