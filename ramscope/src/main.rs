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

const DEFAULT_PORT: u16 = 8765;

fn help() {
    println!(
        "RamScope {} — analyse RAM / services / démarrage pour Windows

Usage : ramscope.exe [options]

  --port N        port d'écoute (défaut {DEFAULT_PORT}, essaie les suivants si occupé)
  --host ADRESSE  adresse d'écoute : 127.0.0.1 (défaut) ou localhost
  --no-browser    ne pas ouvrir la fenêtre automatiquement
  --no-admin      rester en lecture seule (ne pas demander les droits admin)
  --demo          données de démonstration (aucun accès à la machine)
  --help          cette aide

L'interface s'ouvre dans une fenêtre d'application. Toutes les modifications
sont réversibles : chaque lot appliqué crée un point de restauration.",
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

    // Élévation UAC au lancement (sauf demande explicite de lecture seule).
    #[cfg(windows)]
    {
        if !demo && !has("--no-admin") && !has("--elevated") && !win::is_admin() {
            if win::elevate() {
                return; // l'instance élevée prend le relais
            }
        }
    }

    // Un port libre dans une petite plage, pour éviter les conflits.
    let mut last_err = None;
    for p in port..port.saturating_add(12) {
        match server::serve(&host, p, demo, open_browser) {
            Ok(()) => return,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    last_err = Some(format!("port {p} occupé"));
                    continue;
                }
                last_err = Some(e.to_string());
                break;
            }
        }
    }

    let msg = format!(
        "RamScope n'a pas pu démarrer son interface locale.\n\n{}",
        last_err.unwrap_or_else(|| "erreur inconnue".into())
    );
    eprintln!("{msg}");
    let _ = std::fs::write(util::data_dir().join("erreur.log"), &msg);

    #[cfg(windows)]
    unsafe {
        let t = ffi::to_wide("RamScope");
        let m = ffi::to_wide(&msg);
        ffi::MessageBoxW(0, m.as_ptr(), t.as_ptr(), 0x10);
    }
}
