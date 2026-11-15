//! Utilitaires transverses : horodatage, identifiants, stockage des rollbacks.
use crate::model::{RollbackEntry, RollbackPoint};
use std::path::PathBuf;

#[cfg(windows)]
pub fn now_string() -> String {
    #[repr(C)]
    #[allow(non_snake_case)]
    struct SystemTimeW {
        wYear: u16,
        wMonth: u16,
        wDayOfWeek: u16,
        wDay: u16,
        wHour: u16,
        wMinute: u16,
        wSecond: u16,
        wMilliseconds: u16,
    }
    extern "system" {
        fn GetLocalTime(lpSystemTime: *mut SystemTimeW);
    }
    unsafe {
        let mut t: SystemTimeW = std::mem::zeroed();
        GetLocalTime(&mut t);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond
        )
    }
}

#[cfg(not(windows))]
pub fn now_string() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = civil(secs);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
}

/// Conversion jours -> date grégorienne (algorithme de Howard Hinnant).
#[cfg(not(windows))]
fn civil(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (
        y,
        m,
        d,
        (rem / 3600) as u32,
        ((rem % 3600) / 60) as u32,
        (rem % 60) as u32,
    )
}

/// Jeton de session imprévisible (32 octets, 64 caractères hexadécimaux).
///
/// Le jeton n'authentifie pas un processus local : il protège l'API contre
/// une page ou une origine qui n'a pas reçu la page de cette session. Voir
/// `SAFETY.md` (décision D9-1).
#[cfg(windows)]
pub fn session_token() -> Result<String, String> {
    use crate::ffi::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG};

    let mut bytes = [0u8; 32];
    let status = unsafe {
        BCryptGenRandom(
            0,
            bytes.as_mut_ptr(),
            bytes.len() as u32,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    };
    if status != 0 {
        return Err(format!(
            "génération du jeton de session impossible (BCryptGenRandom 0x{status:08X})"
        ));
    }
    Ok(hex_lower(&bytes))
}

#[cfg(not(windows))]
pub fn session_token() -> Result<String, String> {
    use std::io::Read;

    let mut bytes = [0u8; 32];
    let mut source = std::fs::File::open("/dev/urandom")
        .map_err(|e| format!("source d'aléa indisponible : {e}"))?;
    source
        .read_exact(&mut bytes)
        .map_err(|e| format!("lecture d'aléa impossible : {e}"))?;
    Ok(hex_lower(&bytes))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

pub fn new_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id() as u128;
    let mut x = nanos ^ (pid << 64) ^ 0x9E3779B97F4A7C15u128;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    format!("{:016x}", x as u64)
}

pub fn data_dir() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        crate::win::data_dir()
    }
    #[cfg(not(windows))]
    {
        let p = PathBuf::from("ramscope-data");
        std::fs::create_dir_all(p.join("rollbacks"))
            .map_err(|e| format!("impossible de créer le dossier de données : {e}"))?;
        Ok(p)
    }
}

pub fn rollback_dir() -> Result<PathBuf, String> {
    let d = data_dir()?.join("rollbacks");
    std::fs::create_dir_all(&d)
        .map_err(|e| format!("impossible de créer le dossier de restauration : {e}"))?;
    Ok(d)
}

pub fn save_rollback(label: &str, entries: Vec<RollbackEntry>) -> Option<RollbackPoint> {
    if entries.is_empty() {
        return None;
    }
    let id = new_id();
    let point = RollbackPoint {
        id: id.clone(),
        created_at: now_string(),
        label: label.to_string(),
        entries,
    };
    let path = rollback_dir().ok()?.join(format!("{id}.json"));
    match serde_json::to_vec_pretty(&point) {
        Ok(bytes) => {
            if std::fs::write(&path, bytes).is_ok() {
                Some(point)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn list_rollbacks() -> Vec<RollbackPoint> {
    let mut out = Vec::new();
    let Ok(dir) = rollback_dir() else {
        return out;
    };
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            if e.path().extension().map(|x| x == "json").unwrap_or(false) {
                if let Ok(s) = std::fs::read_to_string(e.path()) {
                    if let Ok(p) = serde_json::from_str::<RollbackPoint>(&s) {
                        out.push(p);
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    out
}

pub fn get_rollback(id: &str) -> Option<RollbackPoint> {
    if !id.chars().all(|c| c.is_ascii_hexdigit()) || id.is_empty() {
        return None;
    }
    let dir = rollback_dir().ok()?;
    let s = std::fs::read_to_string(dir.join(format!("{id}.json"))).ok()?;
    serde_json::from_str(&s).ok()
}

pub fn delete_rollback(id: &str) {
    if id.chars().all(|c| c.is_ascii_hexdigit()) && !id.is_empty() {
        if let Ok(dir) = rollback_dir() {
            let _ = std::fs::remove_file(dir.join(format!("{id}.json")));
        }
    }
}

pub fn fmt_bytes(b: u64) -> String {
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = MB * 1024.0;
    if b as f64 >= GB {
        format!("{:.2} Go", b as f64 / GB)
    } else {
        format!("{:.0} Mo", b as f64 / MB)
    }
}

/// Nom de fichier exécutable extrait d'une ligne de commande.
pub fn extract_exe(cmd: &str) -> String {
    let c = cmd.trim();
    if c.is_empty() {
        return String::new();
    }
    let raw = if let Some(rest) = c.strip_prefix('"') {
        rest.split('"').next().unwrap_or("").to_string()
    } else {
        // Pas de guillemets : on coupe au premier argument connu
        let lower = c.to_lowercase();
        match lower.find(".exe") {
            Some(i) => c[..i + 4].to_string(),
            None => c.split_whitespace().next().unwrap_or("").to_string(),
        }
    };
    raw.rsplit(['\\', '/']).next().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_jeton_de_session_est_long_hexadecimal_et_imprevisible() {
        let a = session_token().expect("le CSPRNG doit être disponible pendant les tests");
        let b = session_token().expect("le CSPRNG doit être disponible pendant les tests");
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(a.chars().all(|c| !c.is_ascii_uppercase()));
        assert_ne!(
            a, b,
            "deux jetons successifs ne doivent pas être identiques"
        );
    }
}
