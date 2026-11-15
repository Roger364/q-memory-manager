// RamScope — voir ce qui occupe la RAM sous Windows et décider de ce qui
// peut être arrêté, désactivé ou passé en démarrage manuel.
//
// Sortie : un seul .exe, aucune installation, aucune dépendance runtime.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod demo;
mod model;
mod plan;
mod platform;
mod report;
mod rules;
mod server;
mod util;

#[cfg(windows)]
mod ffi;
#[cfg(windows)]
mod win;

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn le_defaut_ne_demande_pas_uac() {
        let mode = elevation_mode(&[], false, false).unwrap();
        assert_eq!(mode, ElevationMode::ReadOnly);
        assert!(!mode.wants_elevation());
        assert!(mode.can_elevate());
    }

    #[test]
    fn deja_admin_ne_demande_pas_uac() {
        let mode = elevation_mode(&[], true, false).unwrap();
        assert_eq!(mode, ElevationMode::AlreadyElevated);
        assert!(!mode.wants_elevation());
        assert!(!mode.can_elevate());
    }

    #[test]
    fn admin_demande_l_uac_une_seule_fois() {
        let mode = elevation_mode(&args(&["--admin"]), false, false).unwrap();
        assert_eq!(mode, ElevationMode::AdminAtStart);
        assert!(mode.wants_elevation());
        assert!(mode.can_elevate());
    }

    #[test]
    fn no_admin_est_strict() {
        let mode = elevation_mode(&args(&["--no-admin"]), false, false).unwrap();
        assert_eq!(mode, ElevationMode::StrictReadOnly);
        assert!(!mode.wants_elevation());
        assert!(!mode.can_elevate());
    }

    #[test]
    fn admin_et_no_admin_sont_rejetes() {
        assert_eq!(
            elevation_mode(&args(&["--admin", "--no-admin"]), false, false),
            Err("--admin et --no-admin sont incompatibles")
        );
    }

    #[test]
    fn elevated_brise_la_boucle_et_interdit_une_nouvelle_elevation() {
        let mode = elevation_mode(&args(&["--elevated", "--admin"]), false, false).unwrap();
        assert_eq!(mode, ElevationMode::AlreadyElevated);
        assert!(!mode.wants_elevation());
        assert!(!mode.can_elevate());
    }

    #[test]
    fn demo_ne_demande_jamais_l_elevation() {
        let mode = elevation_mode(&args(&["--demo", "--admin"]), false, true).unwrap();
        assert_eq!(mode, ElevationMode::Demo);
        assert!(!mode.wants_elevation());
        assert!(!mode.can_elevate());
    }

    #[test]
    fn une_relance_elevee_reessaie_le_port_avant_de_changer() {
        assert!(should_retry_same_port(
            ElevationMode::AlreadyElevated,
            true,
            0
        ));
        assert!(should_retry_same_port(
            ElevationMode::AlreadyElevated,
            true,
            19
        ));
        assert!(!should_retry_same_port(
            ElevationMode::AlreadyElevated,
            true,
            20
        ));
        assert!(!should_retry_same_port(ElevationMode::ReadOnly, true, 0));
        assert!(!should_retry_same_port(
            ElevationMode::AlreadyElevated,
            false,
            0
        ));
    }
}

const DEFAULT_PORT: u16 = 8765;

/// Politique d'élévation décidée une fois au démarrage. Le serveur ne reçoit
/// que le droit résiduel de demander l'UAC ; il ne peut pas élever un mode
/// strict ou une démonstration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElevationMode {
    /// Défaut : lecture seule, élévation possible depuis l'interface.
    ReadOnly,
    /// `--admin` : demande l'UAC au lancement.
    AdminAtStart,
    /// `--no-admin` : lecture seule et aucune élévation possible.
    StrictReadOnly,
    /// `--demo` : aucun accès réel et aucune élévation.
    Demo,
    /// Instance déjà élevée ou relance interne `--elevated`.
    AlreadyElevated,
}

impl ElevationMode {
    fn wants_elevation(self) -> bool {
        matches!(self, Self::AdminAtStart)
    }

    fn can_elevate(self) -> bool {
        matches!(self, Self::ReadOnly | Self::AdminAtStart)
    }
}

/// Décision pure, testable sans lancer l'UAC ni toucher à Windows.
fn elevation_mode(
    args: &[String],
    already_admin: bool,
    demo: bool,
) -> Result<ElevationMode, &'static str> {
    let has = |flag: &str| args.iter().any(|a| a == flag);
    if has("--admin") && has("--no-admin") {
        return Err("--admin et --no-admin sont incompatibles");
    }
    if demo {
        return Ok(ElevationMode::Demo);
    }
    if already_admin {
        return Ok(ElevationMode::AlreadyElevated);
    }
    // `--elevated` est interne : il ne doit jamais provoquer une seconde
    // demande UAC, même si l'instance a perdu ses droits entre-temps.
    if has("--elevated") {
        return Ok(ElevationMode::AlreadyElevated);
    }
    if has("--no-admin") {
        return Ok(ElevationMode::StrictReadOnly);
    }
    if has("--admin") {
        return Ok(ElevationMode::AdminAtStart);
    }
    Ok(ElevationMode::ReadOnly)
}

const ELEVATED_PORT_RETRIES: u8 = 20;

/// Vrai si une relance élevée doit attendre que l'ancien serveur libère le
/// même port avant de se rabattre sur le suivant. Séparé de la boucle pour
/// être testable sans ouvrir de socket.
fn should_retry_same_port(mode: ElevationMode, addr_in_use: bool, retries: u8) -> bool {
    addr_in_use && matches!(mode, ElevationMode::AlreadyElevated) && retries < ELEVATED_PORT_RETRIES
}

fn fatal(msg: &str) {
    eprintln!("RamScope : {msg}");
    #[cfg(windows)]
    unsafe {
        let t = ffi::to_wide("RamScope");
        let m = ffi::to_wide(msg);
        ffi::MessageBoxW(0, m.as_ptr(), t.as_ptr(), 0x10);
    }
}

fn help() {
    println!(
        "RamScope {} — analyse RAM / services / démarrage pour Windows

Usage : ramscope.exe [options]

  --port N        port d'écoute (défaut {DEFAULT_PORT}, essaie les suivants si occupé)
  --host ADRESSE  adresse d'écoute : 127.0.0.1 (défaut) ou localhost
                  localhost se lie à la résolution locale ; l'alias 127.0.0.1
                  n'est pas garanti
  --no-browser    ne pas ouvrir la fenêtre automatiquement
  --admin         demander l'élévation UAC au lancement (hors --demo)
  --no-admin      interdit toute demande d'élévation (ne rétrograde pas
                  une session déjà élevée)
  --demo          données de démonstration (aucun accès à la machine)
  --help          cette aide

Sans option, RamScope ne demande pas l'UAC ; s'il n'est pas déjà élevé, il
démarre en lecture seule. L'élévation est demandée depuis l'interface après
consultation de l'inventaire. L'interface
s'ouvre dans une fenêtre d'application. Les changements de configuration sont
réversibles (point de restauration) ; la terminaison d'un processus ne l'est
pas et n'est jamais annulée par ce point.",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let val = |f: &str| -> Option<String> {
        args.iter()
            .position(|a| a == f)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };

    if has("--help") || has("-h") {
        help();
        return;
    }

    let demo = has("--demo");
    let host = val("--host").unwrap_or_else(|| "127.0.0.1".into());
    let port: u16 = val("--port")
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let open_browser = !has("--no-browser");

    #[cfg(windows)]
    let already_admin = !demo && win::is_admin();
    #[cfg(not(windows))]
    let already_admin = false;

    let mode = match elevation_mode(&args, already_admin, demo) {
        Ok(mode) => mode,
        Err(e) => {
            fatal(e);
            return;
        }
    };

    // Seul `--admin` demande l'UAC au lancement. Un refus UAC laisse le
    // processus courant en lecture seule ; le serveur pourra proposer une
    // nouvelle tentative depuis l'interface.
    #[cfg(windows)]
    {
        if mode.wants_elevation() && !win::is_admin() && win::elevate() {
            return; // l'instance élevée prend le relais
        }
    }

    let can_elevate = mode.can_elevate();

    // Un port libre dans une petite plage, pour éviter les conflits.
    // Une relance --elevated part d'un serveur lecture seule encore vivant :
    // il faut lui laisser le temps de libérer le port demandé avant de se
    // rabattre sur le suivant, sinon l'interface reste bloquée sur l'ancien
    // port alors que l'instance élevée écoute ailleurs.
    let mut elevated_port_retries = 0u8;
    let mut last_err = None;
    for p in port..port.saturating_add(12) {
        loop {
            match server::serve(&host, p, demo, can_elevate, open_browser) {
                Ok(()) => return,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::AddrInUse {
                        if should_retry_same_port(mode, true, elevated_port_retries) {
                            elevated_port_retries += 1;
                            std::thread::sleep(std::time::Duration::from_millis(250));
                            continue;
                        }
                        last_err = Some(format!("port {p} occupé"));
                        break;
                    }
                    last_err = Some(e.to_string());
                    break;
                }
            }
        }
    }

    let msg = format!(
        "RamScope n'a pas pu démarrer son interface locale.\n\n{}",
        last_err.unwrap_or_else(|| "erreur inconnue".into())
    );
    if let Ok(dir) = util::data_dir() {
        let _ = std::fs::write(dir.join("erreur.log"), &msg);
    }

    fatal(&msg);
}
