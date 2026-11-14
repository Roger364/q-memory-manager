//! Serveur HTTP local : sert l'interface et l'API sur la boucle locale.
//! L'hôte est validé, l'origine comparée exactement, et toute route `/api/`
//! exige un jeton de session.
use crate::model::*;
use crate::platform::Backend;
use crate::util;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tiny_http::{Header, Method, Request, Response, Server};

const INDEX: &str = include_str!("web/index.html");
const APPJS: &str = include_str!("web/app.js");
const CSS: &str = include_str!("web/style.css");

/// Le script est désormais entièrement externe : `script-src` n'autorise que
/// l'origine. Les styles inline restent nécessaires aux largeurs et couleurs
/// calculées dans les cartes ; `style-src` conserve donc `unsafe-inline`.
const CSP: &str = "default-src 'self'; script-src 'self'; \
style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; \
frame-ancestors 'none'; base-uri 'none'; form-action 'none'";

pub struct App {
    pub backend: Backend,
    pub token: String,
    pub cache: Mutex<Option<(Instant, Snapshot)>>,
    pub stop: AtomicBool,
    /// Hôtes exacts que cette session accepte (`127.0.0.1:<port>`,
    /// `localhost:<port>`). Tout autre `Host` est refusé, pages comprises.
    pub allowed_hosts: Vec<String>,
}

impl App {
    fn snapshot(&self, force: bool) -> Snapshot {
        if !force {
            if let Some((t, s)) = self.cache.lock().unwrap().as_ref() {
                if t.elapsed() < Duration::from_millis(2000) {
                    return s.clone();
                }
            }
        }
        let s = self.backend.snapshot();
        *self.cache.lock().unwrap() = Some((Instant::now(), s.clone()));
        s
    }
    fn invalidate(&self) {
        *self.cache.lock().unwrap() = None;
    }
}

fn hdr(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn send(req: Request, code: u16, ctype: &str, body: String) {
    let resp = Response::from_string(body)
        .with_status_code(code)
        .with_header(hdr("Content-Type", ctype))
        .with_header(hdr("Cache-Control", "no-store"))
        .with_header(hdr("X-Content-Type-Options", "nosniff"))
        .with_header(hdr("Referrer-Policy", "no-referrer"))
        .with_header(hdr("Content-Security-Policy", CSP))
        .with_header(hdr("X-Frame-Options", "DENY"))
        .with_header(hdr("Cross-Origin-Resource-Policy", "same-origin"));
    let _ = req.respond(resp);
}

fn json<T: serde::Serialize>(req: Request, code: u16, v: &T) {
    send(
        req,
        code,
        "application/json; charset=utf-8",
        serde_json::to_string(v).unwrap_or_else(|_| "{}".into()),
    );
}

fn err(req: Request, code: u16, msg: &str) {
    json(req, code, &serde_json::json!({ "error": msg }));
}

/// Valeur d'un en-tête, chaîne vide si absent.
fn header_value(req: &Request, name: &'static str) -> String {
    req.headers()
        .iter()
        .find(|h| h.field.equiv(name))
        .map(|h| h.value.as_str().to_string())
        .unwrap_or_default()
}

/// Vrai si l'hôte demandé fait exactement partie de ceux de la session.
/// Aucune comparaison par suffixe : `127.0.0.1:8765.attaquant.tld` est refusé.
fn host_allowed(allowed: &[String], host: &str) -> bool {
    !host.is_empty() && allowed.iter().any(|a| a.eq_ignore_ascii_case(host))
}

/// Vrai si l'origine vaut exactement `http://<hôte validé>`.
/// Une origine contenant un chemin, un autre schéma ou un `null` est refusée.
fn origin_allowed(origin: &str, host: &str) -> bool {
    let rest = match origin.strip_prefix("http://") {
        Some(r) => r,
        None => return false,
    };
    if rest.is_empty() || rest.contains('/') {
        return false;
    }
    rest.eq_ignore_ascii_case(host)
}

/// Comparaison de jeton sans sortie anticipée sur le contenu.
fn token_matches(given: &str, expected: &str) -> bool {
    let (a, b) = (given.as_bytes(), expected.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn is_loopback(host: &str) -> bool {
    host == "127.0.0.1" || host.eq_ignore_ascii_case("localhost")
}

#[derive(serde::Deserialize)]
struct ApplyReq {
    #[serde(default)]
    actions: Vec<Action>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(serde::Deserialize)]
struct RestoreReq {
    id: String,
}

fn handle(app: &App, mut req: Request) {
    let url = req.url().to_string();
    let path = url.split('?').next().unwrap_or("").to_string();
    let method = req.method().clone();

    // --- garde-fou 1 : hôte servi, y compris pour les pages ---
    let host = header_value(&req, "Host");
    if !host_allowed(&app.allowed_hosts, &host) {
        return err(req, 403, "hôte non autorisé");
    }

    // --- garde-fou 2 : jeton exigé sur toute l'API ---
    if path.starts_with("/api/") {
        let given = header_value(&req, "X-RamScope-Token");
        if !token_matches(&given, &app.token) {
            return err(req, 403, "jeton de session invalide");
        }
    }

    // --- pages statiques ---
    if method == Method::Get {
        match path.as_str() {
            "/" | "/index.html" => {
                let html = INDEX.replace("__TOKEN__", &app.token);
                return send(req, 200, "text/html; charset=utf-8", html);
            }
            "/app.js" => {
                return send(
                    req,
                    200,
                    "application/javascript; charset=utf-8",
                    APPJS.into(),
                )
            }
            "/style.css" => return send(req, 200, "text/css; charset=utf-8", CSS.into()),
            "/favicon.ico" => return send(req, 200, "image/svg+xml", favicon()),
            "/api/snapshot" => {
                let force = url.contains("force=1");
                let s = app.snapshot(force);
                return json(req, 200, &s);
            }
            "/api/rollbacks" => return json(req, 200, &util::list_rollbacks()),
            "/api/health" => {
                return json(
                    req,
                    200,
                    &serde_json::json!({ "ok": true, "demo": app.backend.is_demo(), "admin": app.backend.is_admin() }),
                )
            }
            _ => return err(req, 404, "introuvable"),
        }
    }

    if method != Method::Post {
        return err(req, 405, "méthode non autorisée");
    }

    // --- garde-fou 3 : origine exacte pour toute mutation ---
    if let Some(o) = req
        .headers()
        .iter()
        .find(|h| h.field.equiv("Origin"))
        .map(|h| h.value.as_str().to_string())
    {
        if !origin_allowed(&o, &host) {
            return err(req, 403, "origine refusée");
        }
    }

    let mut body = String::new();
    if req.as_reader().read_to_string(&mut body).is_err() {
        return err(req, 400, "corps de requête illisible");
    }

    match path.as_str() {
        "/api/apply" => {
            let parsed: Result<ApplyReq, _> = serde_json::from_str(&body);
            let r = match parsed {
                Ok(r) => r,
                Err(e) => return err(req, 400, &format!("requête invalide : {e}")),
            };
            if r.actions.is_empty() {
                return err(req, 400, "aucune action demandée");
            }
            if r.actions.len() > 400 {
                return err(req, 400, "trop d'actions d'un coup (max 400)");
            }
            if !app.backend.is_admin() {
                return err(
                    req,
                    403,
                    "droits administrateur requis : clique sur « Passer en administrateur »",
                );
            }
            let verdicts = crate::plan::validate(&r.actions, std::process::id());
            if let Some(refus) = verdicts.iter().find(|v| !v.allowed) {
                return err(req, 403, &refus.message);
            }
            let stop_now = r
                .actions
                .iter()
                .any(|a| matches!(a, Action::StopService { .. }));
            let before = if stop_now {
                Some(app.backend.available_memory())
            } else {
                None
            };
            let entries = app.backend.apply(&r.actions);
            // Attente bornée : la mémoire n'est pas rendue instantanément par
            // Windows. On ne mesure que si un arrêt immédiat a été demandé.
            let memory = before.map(|b| {
                std::thread::sleep(Duration::from_millis(700));
                let a = app.backend.available_memory();
                MemoryDelta {
                    available_before: b,
                    available_after: a,
                    gain: Gain::measured(a.saturating_sub(b), GainScope::System),
                }
            });
            let results: Vec<ActionResult> = entries
                .iter()
                .map(|e| ActionResult {
                    label: e.label.clone(),
                    ok: e.ok,
                    message: e.message.clone(),
                })
                .collect();
            // Un point de restauration ne contient que ce qui peut être annulé :
            // un Kill réussi n'a pas d'inverse et ne doit pas créer un point vide.
            let ok_entries: Vec<RollbackEntry> = entries
                .into_iter()
                .filter(|e| e.ok && e.undo.is_some())
                .collect();
            let label = r.label.unwrap_or_else(|| "Optimisation RamScope".into());
            let point = if ok_entries.is_empty() {
                None
            } else {
                util::save_rollback(&label, ok_entries)
            };
            app.invalidate();
            json(
                req,
                200,
                &ApplyResponse {
                    results,
                    rollback_id: point.map(|p| p.id),
                    memory,
                },
            )
        }
        "/api/restore" => {
            let r: RestoreReq = match serde_json::from_str(&body) {
                Ok(r) => r,
                Err(_) => return err(req, 400, "identifiant invalide"),
            };
            let point = match util::get_rollback(&r.id) {
                Some(p) => p,
                None => return err(req, 404, "point de restauration introuvable"),
            };
            if !app.backend.is_admin() {
                return err(req, 403, "droits administrateur requis");
            }
            let undos: Vec<Action> = point
                .entries
                .iter()
                .filter_map(|e| e.undo.clone())
                .collect();
            if undos.is_empty() {
                return err(req, 400, "ce point ne contient aucune action réversible");
            }
            let entries = app.backend.apply(&undos);
            let results: Vec<ActionResult> = entries
                .iter()
                .map(|e| ActionResult {
                    label: e.label.clone(),
                    ok: e.ok,
                    message: e.message.clone(),
                })
                .collect();
            let all_ok = entries.iter().all(|e| e.ok);
            if all_ok {
                util::delete_rollback(&r.id);
            }
            app.invalidate();
            json(
                req,
                200,
                &ApplyResponse {
                    results,
                    rollback_id: None,
                    memory: None,
                },
            )
        }
        "/api/elevate" => {
            if app.backend.elevate() {
                app.stop.store(true, Ordering::SeqCst);
                json(req, 200, &serde_json::json!({"ok": true}))
            } else {
                err(
                    req,
                    500,
                    "impossible de relancer en administrateur (UAC refusé ?)",
                )
            }
        }
        "/api/export" => {
            let s = app.snapshot(false);
            let dir = util::data_dir().join("rapports");
            let _ = std::fs::create_dir_all(&dir);
            let stamp = util::now_string().replace([':', ' '], "-");
            let html_path = dir.join(format!("ramscope-{stamp}.html"));
            let json_path = dir.join(format!("ramscope-{stamp}.json"));
            let ok_html = std::fs::write(&html_path, crate::report::build(&s)).is_ok();
            let ok_json = serde_json::to_vec_pretty(&s)
                .map(|b| std::fs::write(&json_path, b).is_ok())
                .unwrap_or(false);
            if ok_html || ok_json {
                json(
                    req,
                    200,
                    &serde_json::json!({
                        "html": if ok_html { html_path.to_string_lossy().to_string() } else { String::new() },
                        "json": if ok_json { json_path.to_string_lossy().to_string() } else { String::new() },
                        "dir": dir.to_string_lossy().to_string()
                    }),
                )
            } else {
                err(req, 500, "impossible d'écrire le rapport")
            }
        }
        "/api/open-dir" => {
            let dir = util::data_dir();
            #[cfg(windows)]
            {
                let _ = std::process::Command::new(crate::win::system32_path("explorer.exe"))
                    .arg(&dir)
                    .spawn();
            }
            json(
                req,
                200,
                &serde_json::json!({"dir": dir.to_string_lossy().to_string()}),
            )
        }
        "/api/quit" => {
            app.stop.store(true, Ordering::SeqCst);
            json(req, 200, &serde_json::json!({"ok": true}))
        }
        _ => err(req, 404, "introuvable"),
    }
}

fn favicon() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="14" fill="#0e1420"/><rect x="12" y="20" width="8" height="26" rx="2" fill="#4ade80"/><rect x="24" y="14" width="8" height="32" rx="2" fill="#38bdf8"/><rect x="36" y="26" width="8" height="20" rx="2" fill="#fbbf24"/><rect x="48" y="32" width="4" height="14" rx="2" fill="#f87171"/></svg>"##.into()
}

pub fn serve(host: &str, port: u16, demo: bool, open_browser: bool) -> std::io::Result<()> {
    if !is_loopback(host) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("adresse non autorisée : {host} (boucle locale uniquement)"),
        ));
    }
    let addr = format!("{host}:{port}");
    let server = Server::http(&addr)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::AddrInUse, format!("{e}")))?;
    let server = Arc::new(server);
    let local_port = server
        .server_addr()
        .to_ip()
        .map(|a| a.port())
        .unwrap_or(port);
    let url = format!("http://{host}:{local_port}");

    let mut allowed_hosts = vec![format!("{host}:{local_port}")];
    for alias in ["127.0.0.1", "localhost"] {
        let h = format!("{alias}:{local_port}");
        if !allowed_hosts.iter().any(|a| a.eq_ignore_ascii_case(&h)) {
            allowed_hosts.push(h);
        }
    }

    let app = Arc::new(App {
        backend: Backend::new(demo),
        token: util::new_id() + &util::new_id(),
        cache: Mutex::new(None),
        stop: AtomicBool::new(false),
        allowed_hosts,
    });

    println!("RamScope {} — {url}", env!("CARGO_PKG_VERSION"));
    if app.backend.is_demo() {
        println!("Mode DÉMONSTRATION (données simulées, aucun effet sur la machine).");
    }
    println!("Ctrl+C ou bouton « Quitter » pour arrêter.");

    if open_browser {
        open_window(&url);
    }

    let mut handles = Vec::new();
    for _ in 0..4 {
        let s = server.clone();
        let a = app.clone();
        handles.push(std::thread::spawn(move || {
            while let Ok(req) = s.recv() {
                handle(&a, req);
                if a.stop.load(Ordering::SeqCst) {
                    break;
                }
            }
        }));
    }

    // boucle de surveillance : arrêt propre
    loop {
        std::thread::sleep(Duration::from_millis(200));
        if app.stop.load(Ordering::SeqCst) {
            break;
        }
    }
    std::thread::sleep(Duration::from_millis(300));
    println!("RamScope arrêté.");
    std::process::exit(0);
}

/// Ouvre une fenêtre « application » (sans barre d'adresse) si Edge/Chrome est
/// disponible, sinon le navigateur par défaut.
fn open_window(url: &str) {
    #[cfg(windows)]
    {
        let candidates = [
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ];
        for c in candidates {
            if std::path::Path::new(c).exists() {
                let arg = format!("--app={url}");
                if std::process::Command::new(c)
                    .args([
                        arg.as_str(),
                        "--window-size=1440,940",
                        "--disable-features=Translate",
                    ])
                    .spawn()
                    .is_ok()
                {
                    return;
                }
            }
        }
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hotes() -> Vec<String> {
        vec!["127.0.0.1:8765".to_string(), "localhost:8765".to_string()]
    }

    #[test]
    fn l_hote_doit_correspondre_exactement() {
        let h = hotes();
        assert!(host_allowed(&h, "127.0.0.1:8765"));
        assert!(host_allowed(&h, "LOCALHOST:8765"));
        assert!(!host_allowed(&h, "attaquant.tld:8765"));
        assert!(!host_allowed(&h, "127.0.0.1:8765.attaquant.tld"));
        assert!(!host_allowed(&h, "127.0.0.1:8766"));
        assert!(!host_allowed(&h, ""));
    }

    #[test]
    fn l_origine_ne_se_compare_pas_par_suffixe() {
        assert!(origin_allowed("http://127.0.0.1:8765", "127.0.0.1:8765"));
        assert!(!origin_allowed(
            "http://attaquant.tld/?x=127.0.0.1:8765",
            "127.0.0.1:8765"
        ));
        assert!(!origin_allowed(
            "http://127.0.0.1:8765.attaquant.tld",
            "127.0.0.1:8765"
        ));
        assert!(!origin_allowed("https://127.0.0.1:8765", "127.0.0.1:8765"));
        assert!(!origin_allowed("null", "127.0.0.1:8765"));
    }

    #[test]
    fn le_jeton_se_compare_sans_raccourci() {
        assert!(token_matches("abc123", "abc123"));
        assert!(!token_matches("abc124", "abc123"));
        assert!(!token_matches("abc", "abc123"));
        assert!(!token_matches("", "abc123"));
    }

    #[test]
    fn seule_la_boucle_locale_est_acceptee() {
        assert!(is_loopback("127.0.0.1"));
        assert!(is_loopback("localhost"));
        assert!(!is_loopback("0.0.0.0"));
        assert!(!is_loopback("192.168.1.10"));
    }

    #[test]
    fn l_interface_ne_convertit_pas_le_gain_differe_en_immediat() {
        let js = include_str!("web/app.js");
        assert!(
            !js.contains("now += e.boot"),
            "conversion différé → immédiat réintroduite"
        );
        assert!(
            !js.contains("if (stopNow) boot = 0"),
            "remise à zéro du différé réintroduite"
        );
        assert!(js.contains("Tout de suite :"));
        assert!(js.contains("Au prochain démarrage :"));
    }

    #[test]
    fn l_interface_interdit_les_formulations_qui_promettent_l_absence_de_risque() {
        let js = include_str!("web/app.js").to_lowercase();
        let html = include_str!("web/index.html").to_lowercase();
        for interdit in ["sans risque", "aucun risque réel", "gain net"] {
            assert!(
                !js.contains(interdit),
                "formulation bannie dans app.js : {interdit}"
            );
            assert!(
                !html.contains(interdit),
                "formulation bannie dans index.html : {interdit}"
            );
        }
        assert!(js.contains("sélectionner les éléments réversibles à gain différé"));
        assert!(js.contains("irréversible"));
        assert!(js.contains("source détectée"));
        assert!(js.contains("effet tout de suite"));
        assert!(js.contains("effet au prochain démarrage"));
        assert!(js.contains("réversibilité"));
        assert!(js.contains("justification"));
    }

    #[test]
    fn une_terminaison_ne_peut_pas_etre_selectionnee_dans_un_lot() {
        let js = include_str!("web/app.js");
        assert!(js.contains("function batchable"));
        assert!(js.contains("item.advice.action.kind !== 'kill'"));
        assert!(js.contains("ne peut pas être appliquée dans un lot"));
    }

    #[test]
    fn le_fond_de_modale_ne_se_ferme_pas_sur_un_clic_herite() {
        let js = include_str!("web/app.js");
        assert!(
            js.contains("addEventListener('pointerdown'"),
            "le fond doit mémoriser le début de l’appui"
        );
        assert!(
            js.contains("if (maskPressed && e.target === $('#mask')) closeModal();"),
            "un clic hérité du bouton d’ouverture ne doit pas fermer la modale"
        );
        assert!(
            !js.contains(
                "$('#mask').onclick = e => { if (e.target === $('#mask')) closeModal(); };"
            ),
            "ancienne fermeture sur clic retargeté réintroduite"
        );
    }

    #[test]
    fn la_csp_n_autorise_pas_le_script_inline() {
        assert!(!CSP.contains("script-src 'self' 'unsafe-inline'"));
        assert!(CSP.contains("script-src 'self';"));
        let html = include_str!("web/index.html");
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("__URL__"));
        assert!(html.contains("name=\"ramscope-token\""));
        let js = include_str!("web/app.js");
        assert!(js.contains("meta[name=\"ramscope-token\"]"));
        assert!(!js.contains("onclick=\""));
        assert!(js.contains("id=\"btnElevate\""));
        assert!(js.contains("api('/api/elevate'"));
    }

    #[test]
    fn refuse_une_adresse_non_locale_au_demarrage() {
        // Le contrôle a lieu avant toute écoute : rien n'est ouvert.
        let e = serve("0.0.0.0", 0, true, false).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput);
    }
}
