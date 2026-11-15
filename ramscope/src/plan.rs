//! Logique pure, indépendante de Windows.
//!
//! Ce module rassemble ce qui doit rester testable partout : validation d'une
//! liste d'actions avant toute modification système, agrégation des gains
//! d'un lot, normalisation et correspondance des règles.
//!
//! Contrainte : aucun appel système, aucune dépendance à la couche FFI
//! Windows ni au module `win`. Le module doit compiler sur toutes les cibles.

use crate::model::{Action, Gain, GainScope, Snapshot, StartType};
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

/// Nombre maximal d'actions acceptées dans un seul lot.
pub const MAX_ACTIONS: usize = 400;

/// Valide une liste d'actions dans l'ordre, sans toucher au système.
///
/// `self_pid` est le PID de RamScope : le programme ne peut pas se terminer
/// lui-même. Les garde-fous de `win.rs` restent en place en second verrou.
pub fn validate(actions: &[Action], self_pid: u32) -> Vec<Verdict> {
    if actions.len() > MAX_ACTIONS {
        return actions
            .iter()
            .map(|_| Verdict::refuse("Lot trop volumineux : 400 actions maximum."))
            .collect();
    }
    actions
        .iter()
        .enumerate()
        .map(|(i, a)| {
            if actions[..i].iter().any(|autre| autre == a) {
                Verdict::refuse("Action présente plusieurs fois dans le lot.")
            } else {
                validate_one(a, self_pid)
            }
        })
        .collect()
}

/// Valide une action utilisateur contre le snapshot qui vient d'être collecté.
///
/// Cette validation est réservée à `/api/apply`. Le chemin interne de
/// restauration ne l'appelle pas : ses actions inverses ont volontairement
/// `from: None` / `was_enabled: None`. Ici, au contraire, l'état observé est
/// obligatoire et doit correspondre, et l'action doit être exactement celle
/// que le snapshot propose à l'utilisateur. Cela empêche un client de choisir
/// un service, une cible ou un état qui n'était pas dans l'analyse courante.
pub fn validate_observed(actions: &[Action], snapshot: &Snapshot) -> Vec<Verdict> {
    actions
        .iter()
        .map(|action| validate_observed_one(action, snapshot))
        .collect()
}

fn validate_observed_one(action: &Action, snapshot: &Snapshot) -> Verdict {
    match action {
        Action::Service { name, start, from } => {
            let Some(service) = snapshot.services.iter().find(|s| s.name == *name) else {
                return Verdict::refuse("Service absent du snapshot courant : relance l'analyse.");
            };
            match from {
                None => {
                    return Verdict::refuse(
                        "État constaté du service manquant : action utilisateur refusée.",
                    );
                }
                Some(attendu) if *attendu != service.start => {
                    return Verdict::refuse(
                        "L'état du service a changé depuis l'analyse : relance l'analyse.",
                    );
                }
                Some(_) => {}
            }
            if *start == service.start {
                return Verdict::refuse("Le service est déjà dans l'état demandé.");
            }
            if service.advice.action.as_ref() != Some(action) {
                return Verdict::refuse("Action non proposée par l'analyse courante.");
            }
            Verdict::allow()
        }
        Action::Startup {
            id,
            enabled,
            was_enabled,
        } => {
            let Some(item) = snapshot.startup.iter().find(|s| s.id == *id) else {
                return Verdict::refuse(
                    "Entrée de démarrage absente du snapshot courant : relance l'analyse.",
                );
            };
            match was_enabled {
                None => {
                    return Verdict::refuse(
                        "État constaté du démarrage manquant : action utilisateur refusée.",
                    );
                }
                Some(attendu) if *attendu != item.enabled => {
                    return Verdict::refuse(
                        "L'état du démarrage a changé depuis l'analyse : relance l'analyse.",
                    );
                }
                Some(_) => {}
            }
            if *enabled == item.enabled {
                return Verdict::refuse("L'entrée de démarrage est déjà dans l'état demandé.");
            }
            if item.advice.action.as_ref() != Some(action) {
                return Verdict::refuse("Action non proposée par l'analyse courante.");
            }
            Verdict::allow()
        }
        Action::Task {
            path,
            enabled,
            was_enabled,
        } => {
            let Some(item) = snapshot.tasks.iter().find(|t| t.path == *path) else {
                return Verdict::refuse("Tâche absente du snapshot courant : relance l'analyse.");
            };
            match was_enabled {
                None => {
                    return Verdict::refuse(
                        "État constaté de la tâche manquant : action utilisateur refusée.",
                    );
                }
                Some(attendu) if *attendu != item.enabled => {
                    return Verdict::refuse(
                        "L'état de la tâche a changé depuis l'analyse : relance l'analyse.",
                    );
                }
                Some(_) => {}
            }
            if *enabled == item.enabled {
                return Verdict::refuse("La tâche est déjà dans l'état demandé.");
            }
            if item.advice.action.as_ref() != Some(action) {
                return Verdict::refuse("Action non proposée par l'analyse courante.");
            }
            Verdict::allow()
        }
        Action::Kill {
            pid,
            name,
            image_path,
            created_at,
        } => {
            let Some(process) = snapshot.processes.iter().find(|p| p.pid == *pid) else {
                return Verdict::refuse(
                    "Processus absent du snapshot courant : relance l'analyse.",
                );
            };
            if process.name != *name
                || !process.path.eq_ignore_ascii_case(image_path)
                || process.created_at != *created_at
            {
                return Verdict::refuse(
                    "L'identité du processus a changé depuis l'analyse : relance l'analyse.",
                );
            }
            if process.advice.action.as_ref() != Some(action) {
                return Verdict::refuse("Terminaison non proposée par l'analyse courante.");
            }
            Verdict::allow()
        }
        Action::StopService { name } => {
            let Some(service) = snapshot.services.iter().find(|s| s.name == *name) else {
                return Verdict::refuse("Service absent du snapshot courant : relance l'analyse.");
            };
            let propose = service.advice.action.as_ref().is_some_and(|action| {
                matches!(
                    action,
                    Action::Service {
                        start: StartType::Manual | StartType::Disabled,
                        ..
                    }
                )
            });
            if !service.running || !propose {
                return Verdict::refuse("Arrêt non proposé par l'analyse courante.");
            }
            Verdict::allow()
        }
        Action::StartService { .. } => Verdict::refuse(
            "Le démarrage d'un service est réservé au chemin interne de restauration.",
        ),
    }
}

fn validate_one(action: &Action, self_pid: u32) -> Verdict {
    match action {
        Action::Service { name, start, from } => {
            if rules::is_protected_service(name) {
                Verdict::refuse("Service vital : RamScope refuse de le modifier.")
            } else if from == &Some(*start) {
                Verdict::refuse("Le service est déjà dans l'état demandé.")
            } else {
                Verdict::allow()
            }
        }
        Action::StopService { name } if rules::is_protected_service(name) => {
            Verdict::refuse("Service vital : RamScope refuse de l'arrêter.")
        }
        Action::StartService { name } if rules::is_protected_service(name) => {
            Verdict::refuse("Service vital : RamScope refuse de le démarrer.")
        }
        Action::Startup {
            enabled,
            was_enabled,
            ..
        } if was_enabled == &Some(*enabled) => {
            Verdict::refuse("L'entrée de démarrage est déjà dans l'état demandé.")
        }
        Action::Task {
            enabled,
            was_enabled,
            ..
        } if was_enabled == &Some(*enabled) => {
            Verdict::refuse("La tâche est déjà dans l'état demandé.")
        }
        Action::Kill {
            pid,
            name,
            image_path,
            created_at,
        } => {
            if rules::is_protected_process(name) || *pid <= 8 {
                Verdict::refuse("Processus vital : RamScope refuse de le terminer.")
            } else if *pid == self_pid {
                Verdict::refuse("RamScope ne peut pas se terminer lui-même")
            } else if image_path.trim().is_empty() {
                Verdict::refuse("Identité du processus incomplète : chemin manquant.")
            } else if *created_at == 0 {
                Verdict::refuse("Identité du processus incomplète : date de création manquante.")
            } else {
                Verdict::allow()
            }
        }
        _ => Verdict::allow(),
    }
}

// ---------------------------------------------------------------------------
// Gains d'un lot
// ---------------------------------------------------------------------------

/// Contexte d'un service hébergé dans un processus partagé (`svchost.exe`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostGain {
    /// PID du processus hôte.
    pub pid: u32,
    /// Nombre total de services qui partagent cet hôte.
    pub services: u32,
    /// Working set du processus hôte.
    pub host_ram: u64,
}

/// Entrée d'agrégation : action, gains typés et, pour un service, son hôte.
#[derive(Clone, Debug)]
pub struct GainEntry {
    pub action: Action,
    pub gain: Gain,
    pub gain_next_boot: Gain,
    pub host: Option<HostGain>,
}

/// Totalise les gains d'un lot : chaque action distincte n'est comptée qu'une
/// fois, même si elle apparaît deux fois dans la liste.
///
/// Règle d'hôte : un service hébergé ne compte que si **tous** les services de
/// son hôte sont dans le lot. Le gain vaut alors la RAM de l'hôte, une seule
/// fois. Jamais la somme des parts `host_ram / shared` : ces parts ne sont pas
/// additives et un hôte maintenu en vie ne libère rien.
pub fn total_gains(entries: &[GainEntry]) -> (Gain, Gain) {
    let mut now = Gain::unavailable();
    let mut boot = Gain::unavailable();
    let mut seen: Vec<Action> = Vec::new();
    // (pid, services attendus, RAM de l'hôte, noms des services du lot)
    let mut hosts: Vec<(u32, u32, u64, Vec<String>)> = Vec::new();

    for e in entries {
        if seen.contains(&e.action) {
            continue;
        }
        seen.push(e.action.clone());

        if let (Action::Service { name, .. }, Some(h)) = (&e.action, &e.host) {
            if h.services > 1 {
                if let Some((_, expected, ram, names)) =
                    hosts.iter_mut().find(|(pid, ..)| *pid == h.pid)
                {
                    *expected = (*expected).max(h.services);
                    *ram = (*ram).max(h.host_ram);
                    if !names.contains(name) {
                        names.push(name.clone());
                    }
                } else {
                    hosts.push((h.pid, h.services, h.host_ram, vec![name.clone()]));
                }
                continue;
            }
        }

        now.merge(&e.gain);
        boot.merge(&e.gain_next_boot);
    }

    for (_, expected, host_ram, names) in hosts {
        if expected > 1 && names.len() as u32 >= expected {
            boot.merge(&Gain::estimated(host_ram, GainScope::Host));
        }
    }

    (now, boot)
}

// ---------------------------------------------------------------------------
// Correspondance des règles
// ---------------------------------------------------------------------------

/// Normalise une chaîne pour comparaison : minuscules uniquement.
pub fn normalize(s: &str) -> String {
    s.to_lowercase()
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
        verifie_cles("SERVICES", rules::SERVICES.iter().map(|r| r.key).collect());
        verifie_cles(
            "PROCESSES",
            rules::PROCESSES.iter().map(|r| r.key).collect(),
        );
        verifie_cles("STARTUP", rules::STARTUP.iter().map(|r| r.key).collect());
        verifie_cles("TASKS", rules::TASKS.iter().map(|r| r.key).collect());
    }

    #[test]
    fn refuse_les_cibles_protegees() {
        let v = validate(
            &[
                Action::Service {
                    name: "RpcSs".into(),
                    start: StartType::Manual,
                    from: None,
                },
                Action::StopService {
                    name: "WinDefend".into(),
                },
                Action::Kill {
                    pid: 4,
                    name: "System".into(),
                    image_path: "C:\\Windows\\System32\\ntoskrnl.exe".into(),
                    created_at: 1,
                },
                Action::Kill {
                    pid: 1234,
                    name: "ramscope.exe".into(),
                    image_path: "C:\\ramscope.exe".into(),
                    created_at: 1,
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
                from: None,
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
            from: None,
        };
        let b = Action::Startup {
            id: "hkcu_run|Spotify".into(),
            enabled: false,
            was_enabled: None,
        };
        let entries = vec![
            GainEntry {
                action: a.clone(),
                gain: Gain::unavailable(),
                gain_next_boot: Gain::estimated(10, GainScope::Private),
                host: None,
            },
            GainEntry {
                action: a,
                gain: Gain::unavailable(),
                gain_next_boot: Gain::estimated(10, GainScope::Private),
                host: None,
            },
            GainEntry {
                action: b,
                gain: Gain::unavailable(),
                gain_next_boot: Gain::estimated(5, GainScope::Private),
                host: None,
            },
        ];
        let (now, boot) = total_gains(&entries);
        assert_eq!(now.bytes, 0);
        assert_eq!(boot.bytes, 15);
    }

    fn service_gain(name: &str, host_pid: u32, shared: u32, host_ram: u64, part: u64) -> GainEntry {
        GainEntry {
            action: Action::Service {
                name: name.into(),
                start: StartType::Manual,
                from: Some(StartType::Auto),
            },
            gain: Gain::unavailable(),
            gain_next_boot: Gain::estimated(part, GainScope::HostPart),
            host: Some(HostGain {
                pid: host_pid,
                services: shared,
                host_ram,
            }),
        }
    }

    #[test]
    fn un_hote_ne_compte_que_si_tous_ses_services_sont_dans_le_lot() {
        const MO: u64 = 1024 * 1024;
        let host = [
            service_gain("a", 980, 4, 400 * MO, 100 * MO),
            service_gain("b", 980, 4, 400 * MO, 100 * MO),
            service_gain("c", 980, 4, 400 * MO, 100 * MO),
            service_gain("d", 980, 4, 400 * MO, 100 * MO),
        ];
        assert_eq!(total_gains(&host[..1]).1.bytes, 0);
        assert_eq!(total_gains(&host[..3]).1.bytes, 0);
        assert_eq!(total_gains(&host).1.bytes, 400 * MO);
        assert_eq!(total_gains(&host).0.bytes, 0);
    }

    #[test]
    fn deux_hotes_complets_s_additionnent() {
        const MO: u64 = 1024 * 1024;
        let entries = vec![
            service_gain("a", 980, 2, 100 * MO, 50 * MO),
            service_gain("b", 980, 2, 100 * MO, 50 * MO),
            service_gain("c", 1040, 2, 200 * MO, 100 * MO),
            service_gain("d", 1040, 2, 200 * MO, 100 * MO),
        ];
        assert_eq!(total_gains(&entries).1.bytes, 300 * MO);
    }

    #[test]
    fn un_hote_incomplet_n_ajoute_rien_meme_avec_un_gain_immediat() {
        const MO: u64 = 1024 * 1024;
        let mut incomplete = service_gain("a", 980, 4, 400 * MO, 100 * MO);
        incomplete.gain = Gain::estimated(90 * MO, GainScope::HostPart);
        let entries = vec![incomplete];
        let (now, boot) = total_gains(&entries);
        assert_eq!(now.bytes, 0);
        assert_eq!(boot.bytes, 0);
    }

    #[test]
    fn normalise_le_nom_avant_comparaison() {
        assert_eq!(normalize("DiagTrack"), "diagtrack");
    }

    #[test]
    fn refuse_une_identite_de_processus_incomplete() {
        let v = validate(
            &[
                Action::Kill {
                    pid: 4242,
                    name: "app.exe".into(),
                    image_path: String::new(),
                    created_at: 42,
                },
                Action::Kill {
                    pid: 4242,
                    name: "app.exe".into(),
                    image_path: "C:\\app.exe".into(),
                    created_at: 0,
                },
            ],
            1,
        );
        assert!(!v[0].allowed && v[0].message.contains("chemin"));
        assert!(!v[1].allowed && v[1].message.contains("création"));
    }

    #[test]
    fn refuse_les_doublons_dans_un_lot() {
        let a = Action::Service {
            name: "DiagTrack".into(),
            start: StartType::Disabled,
            from: None,
        };
        let v = validate(&[a.clone(), a], 1);
        assert!(v[0].allowed);
        assert!(!v[1].allowed && v[1].message.contains("plusieurs fois"));
    }

    #[test]
    fn refuse_un_lot_de_plus_de_400_actions() {
        let a = Action::Service {
            name: "DiagTrack".into(),
            start: StartType::Disabled,
            from: None,
        };
        let actions = vec![a; MAX_ACTIONS + 1];
        let v = validate(&actions, 1);
        assert_eq!(v.len(), MAX_ACTIONS + 1);
        assert!(v.iter().all(|x| !x.allowed));
        assert!(v[0].message.contains("400"));
    }

    #[test]
    fn refuse_une_action_deja_appliquee() {
        let v = validate(
            &[
                Action::Service {
                    name: "DiagTrack".into(),
                    start: StartType::Manual,
                    from: Some(StartType::Manual),
                },
                Action::Startup {
                    id: "hkcu_run|Spotify".into(),
                    enabled: false,
                    was_enabled: Some(false),
                },
            ],
            1,
        );
        assert!(!v[0].allowed && v[0].message.contains("déjà"));
        assert!(!v[1].allowed && v[1].message.contains("déjà"));
    }

    #[test]
    fn accepte_un_lot_vide() {
        assert!(validate(&[], 1).is_empty());
    }

    fn snapshot_demo() -> Snapshot {
        crate::demo::DemoState::default().snapshot()
    }

    fn action_service_proposee(snapshot: &Snapshot) -> Action {
        snapshot
            .services
            .iter()
            .find_map(|service| match &service.advice.action {
                Some(action @ Action::Service { .. }) => Some(action.clone()),
                _ => None,
            })
            .expect("la démo doit proposer au moins un service")
    }

    #[test]
    fn refuse_un_from_null_sur_une_action_utilisateur() {
        let snapshot = snapshot_demo();
        let mut action = action_service_proposee(&snapshot);
        if let Action::Service { from, .. } = &mut action {
            *from = None;
        }
        let verdict = validate_observed(&[action], &snapshot);
        assert!(!verdict[0].allowed);
        assert!(verdict[0].message.contains("manquant"));
    }

    #[test]
    fn refuse_un_etat_divergent_du_snapshot() {
        let snapshot = snapshot_demo();
        let mut action = action_service_proposee(&snapshot);
        let name = match &action {
            Action::Service { name, .. } => name.clone(),
            _ => unreachable!(),
        };
        let actuel = snapshot
            .services
            .iter()
            .find(|service| service.name == name)
            .expect("le service proposé doit exister")
            .start;
        if let Action::Service { from, .. } = &mut action {
            *from = Some(if actuel == StartType::Auto {
                StartType::Manual
            } else {
                StartType::Auto
            });
        }
        let verdict = validate_observed(&[action], &snapshot);
        assert!(!verdict[0].allowed);
        assert!(verdict[0].message.contains("changé"));
    }

    #[test]
    fn accepte_les_actions_et_l_etat_du_snapshot() {
        let snapshot = snapshot_demo();
        let service = action_service_proposee(&snapshot);
        let startup = snapshot
            .startup
            .iter()
            .find_map(|item| match &item.advice.action {
                Some(action @ Action::Startup { .. }) => Some(action.clone()),
                _ => None,
            })
            .expect("la démo doit proposer une entrée de démarrage");
        let task = snapshot
            .tasks
            .iter()
            .find_map(|item| match &item.advice.action {
                Some(action @ Action::Task { .. }) => Some(action.clone()),
                _ => None,
            })
            .expect("la démo doit proposer une tâche");
        let verdicts = validate_observed(&[service, startup, task], &snapshot);
        assert!(verdicts.iter().all(|v| v.allowed), "{verdicts:?}");
    }

    #[test]
    fn refuse_une_action_non_proposee_meme_avec_le_bon_etat() {
        let snapshot = snapshot_demo();
        let mut action = action_service_proposee(&snapshot);
        let name = match &action {
            Action::Service { name, .. } => name.clone(),
            _ => unreachable!(),
        };
        let actuel = snapshot
            .services
            .iter()
            .find(|service| service.name == name)
            .expect("le service proposé doit exister")
            .start;
        let original = match &action {
            Action::Service { start, .. } => *start,
            _ => unreachable!(),
        };
        let nouveau = [
            StartType::Auto,
            StartType::Delayed,
            StartType::Manual,
            StartType::Disabled,
        ]
        .into_iter()
        .find(|candidat| *candidat != actuel && *candidat != original)
        .expect("un autre état de démarrage doit exister");
        if let Action::Service { start, .. } = &mut action {
            *start = nouveau;
        }
        let verdict = validate_observed(&[action], &snapshot);
        assert!(!verdict[0].allowed);
        assert!(verdict[0].message.contains("proposée"));
    }

    #[test]
    fn refuse_le_demarrage_d_un_service_sur_la_route_utilisateur() {
        let snapshot = snapshot_demo();
        let verdict = validate_observed(
            &[Action::StartService {
                name: "DiagTrack".into(),
            }],
            &snapshot,
        );
        assert!(!verdict[0].allowed);
        assert!(verdict[0].message.contains("restauration"));
    }

    #[test]
    fn accepte_un_arret_immediat_propose_et_refuse_un_arret_arbitraire() {
        let snapshot = snapshot_demo();
        let propose = snapshot
            .services
            .iter()
            .find(|service| {
                service.running
                    && service.advice.action.as_ref().is_some_and(|action| {
                        matches!(
                            action,
                            Action::Service {
                                start: StartType::Manual | StartType::Disabled,
                                ..
                            }
                        )
                    })
            })
            .expect("la démo doit proposer un arrêt immédiat");
        let allowed = validate_observed(
            &[Action::StopService {
                name: propose.name.clone(),
            }],
            &snapshot,
        );
        assert!(allowed[0].allowed, "{:?}", allowed[0]);

        let arbitraire = snapshot
            .services
            .iter()
            .find(|service| service.running && service.advice.action.is_none())
            .expect("la démo doit contenir un service sans action proposée");
        let refused = validate_observed(
            &[Action::StopService {
                name: arbitraire.name.clone(),
            }],
            &snapshot,
        );
        assert!(!refused[0].allowed);
        assert!(refused[0].message.contains("proposé"));
    }

    #[test]
    fn accepte_une_terminaison_proposee_par_le_snapshot() {
        let snapshot = snapshot_demo();
        let action = snapshot
            .processes
            .iter()
            .find_map(|process| match &process.advice.action {
                Some(action @ Action::Kill { .. }) => Some(action.clone()),
                _ => None,
            })
            .expect("la démo doit proposer une terminaison");
        let verdict = validate_observed(&[action], &snapshot);
        assert!(verdict[0].allowed, "{:?}", verdict[0]);
    }
}
