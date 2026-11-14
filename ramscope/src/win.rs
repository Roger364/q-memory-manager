//! Collecte et application des modifications sur Windows (FFI Win32).
use crate::ffi::*;
use crate::model::*;
use crate::plan;
use crate::rules;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUNONCE_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce";
const APPROVED_KEY: &str =
    "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved";
const SERVICES_KEY: &str = "SYSTEM\\CurrentControlSet\\Services";

// ---------------------------------------------------------------------------
// Registre : petits utilitaires
// ---------------------------------------------------------------------------

struct RegKey(HKEY);
impl RegKey {
    fn open(root: HKEY, sub: &str, access: DWORD) -> Option<RegKey> {
        let mut h: HKEY = 0;
        let w = to_wide(sub);
        let r = unsafe { RegOpenKeyExW(root, w.as_ptr(), 0, access, &mut h) };
        if r == 0 && h != 0 {
            Some(RegKey(h))
        } else {
            None
        }
    }
    fn create(root: HKEY, sub: &str, access: DWORD) -> Option<RegKey> {
        let mut h: HKEY = 0;
        let mut disp: DWORD = 0;
        let w = to_wide(sub);
        let r = unsafe {
            RegCreateKeyExW(
                root,
                w.as_ptr(),
                0,
                std::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                access,
                std::ptr::null_mut(),
                &mut h,
                &mut disp,
            )
        };
        if r == 0 && h != 0 {
            Some(RegKey(h))
        } else {
            None
        }
    }
    fn values(&self) -> Vec<(String, u32, Vec<u8>)> {
        let mut out = Vec::new();
        let mut i = 0;
        loop {
            let mut name = [0u16; 512];
            let mut nlen = name.len() as DWORD;
            let mut typ: DWORD = 0;
            let mut data = vec![0u8; 8192];
            let mut dlen = data.len() as DWORD;
            let r = unsafe {
                RegEnumValueW(
                    self.0,
                    i,
                    name.as_mut_ptr(),
                    &mut nlen,
                    std::ptr::null_mut(),
                    &mut typ,
                    data.as_mut_ptr(),
                    &mut dlen,
                )
            };
            if r != 0 {
                break;
            }
            data.truncate(dlen as usize);
            let nm = String::from_utf16_lossy(&name[..nlen as usize]);
            out.push((nm, typ, data));
            i += 1;
            if i > 4096 {
                break;
            }
        }
        out
    }
    #[allow(dead_code)]
    fn subkeys(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut i = 0;
        loop {
            let mut name = [0u16; 512];
            let mut nlen = name.len() as DWORD;
            let r = unsafe {
                RegEnumKeyExW(
                    self.0,
                    i,
                    name.as_mut_ptr(),
                    &mut nlen,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };
            if r != 0 {
                break;
            }
            out.push(String::from_utf16_lossy(&name[..nlen as usize]));
            i += 1;
            if i > 8192 {
                break;
            }
        }
        out
    }
    fn get(&self, name: &str) -> Option<(u32, Vec<u8>)> {
        let w = to_wide(name);
        let mut typ: DWORD = 0;
        let mut len: DWORD = 0;
        let r = unsafe {
            RegQueryValueExW(
                self.0,
                w.as_ptr(),
                std::ptr::null_mut(),
                &mut typ,
                std::ptr::null_mut(),
                &mut len,
            )
        };
        if r != 0 {
            return None;
        }
        let mut buf = vec![0u8; len.max(1) as usize];
        let r = unsafe {
            RegQueryValueExW(
                self.0,
                w.as_ptr(),
                std::ptr::null_mut(),
                &mut typ,
                buf.as_mut_ptr(),
                &mut len,
            )
        };
        if r != 0 {
            return None;
        }
        buf.truncate(len as usize);
        Some((typ, buf))
    }
    fn get_string(&self, name: &str) -> Option<String> {
        let (t, d) = self.get(name)?;
        if t == REG_SZ || t == REG_EXPAND_SZ {
            let u: Vec<u16> = d
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes(*c))
                .collect();
            Some(expand(&from_wide_buf(&u)))
        } else {
            None
        }
    }
    fn set_binary(&self, name: &str, data: &[u8]) -> Result<(), String> {
        let w = to_wide(name);
        let r = unsafe {
            RegSetValueExW(
                self.0,
                w.as_ptr(),
                0,
                REG_BINARY,
                data.as_ptr(),
                data.len() as DWORD,
            )
        };
        if r == 0 {
            Ok(())
        } else {
            Err(err_msg(r as u32))
        }
    }
}
impl Drop for RegKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

fn expand(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let w = to_wide(s);
    let mut buf = [0u16; 1024];
    let n = unsafe { ExpandEnvironmentStringsW(w.as_ptr(), buf.as_mut_ptr(), buf.len() as DWORD) };
    if n == 0 || n as usize > buf.len() {
        s.to_string()
    } else {
        from_wide_buf(&buf[..n as usize])
    }
}

// ---------------------------------------------------------------------------
// Métadonnées système
// ---------------------------------------------------------------------------

pub fn is_admin() -> bool {
    unsafe {
        let mut token: HANDLE = 0;
        // -1 = pseudo-handle du processus courant
        if OpenProcessToken(-1, TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elev = TOKEN_ELEVATION::default();
        let mut ret: DWORD = 0;
        let ok = GetTokenInformation(
            token,
            20, // TokenElevation
            &mut elev as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<TOKEN_ELEVATION>() as DWORD,
            &mut ret,
        );
        CloseHandle(token);
        ok != 0 && elev.TokenIsElevated != 0
    }
}

fn computer_name() -> String {
    let mut buf = [0u16; 256];
    let mut n = buf.len() as DWORD;
    if unsafe { GetComputerNameW(buf.as_mut_ptr(), &mut n) } != 0 {
        from_wide_buf(&buf[..n as usize])
    } else {
        "PC".into()
    }
}

fn user_name() -> String {
    let mut buf = [0u16; 256];
    let mut n = buf.len() as DWORD;
    if unsafe { GetUserNameW(buf.as_mut_ptr(), &mut n) } != 0 {
        from_wide_buf(&buf[..n as usize])
    } else {
        String::new()
    }
}

fn os_name() -> String {
    let mut v = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as DWORD,
        ..Default::default()
    };
    let ok = unsafe { RtlGetVersion(&mut v) } == 0;
    if !ok {
        return "Windows".into();
    }
    let build = v.dwBuildNumber;
    let name = if build >= 22000 {
        "Windows 11"
    } else {
        "Windows 10"
    };
    format!("{name} (build {build})")
}

pub fn elevate() -> bool {
    let args: Vec<String> = std::env::args().skip(1).collect();
    elevate_with_args(&args)
}

/// Reconstruit les arguments de l'instance élevée. `--elevated` est ajouté
/// une seule fois et `--no-admin` est retiré : demander l'élévation est
/// incompatible avec un démarrage en lecture seule.
fn elevated_args(args: &[String]) -> Vec<String> {
    let mut out: Vec<String> = args
        .iter()
        .filter(|a| a.as_str() != "--elevated" && a.as_str() != "--no-admin")
        .cloned()
        .collect();
    out.push("--elevated".into());
    out
}

/// Échappement compatible avec la ligne de commande Windows utilisée par
/// ShellExecuteW : les espaces et guillemets ne doivent pas changer le sens
/// des options transmises.
fn quote_windows_arg(arg: &str) -> String {
    if !arg.is_empty() && !arg.chars().any(|c| c.is_whitespace() || c == '"') {
        return arg.to_string();
    }
    let mut out = String::from("\"");
    let mut slashes = 0usize;
    for c in arg.chars() {
        match c {
            '\\' => slashes += 1,
            '"' => {
                out.push_str(&"\\".repeat(slashes * 2 + 1));
                out.push('"');
                slashes = 0;
            }
            _ => {
                if slashes > 0 {
                    out.push_str(&"\\".repeat(slashes));
                    slashes = 0;
                }
                out.push(c);
            }
        }
    }
    if slashes > 0 {
        out.push_str(&"\\".repeat(slashes * 2));
    }
    out.push('"');
    out
}

fn elevated_params(args: &[String]) -> String {
    elevated_args(args)
        .iter()
        .map(|a| quote_windows_arg(a))
        .collect::<Vec<_>>()
        .join(" ")
}

fn elevate_with_args(args: &[String]) -> bool {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let params = to_wide(&elevated_params(args));
    let verb = to_wide("runas");
    let file = to_wide(&exe.to_string_lossy());
    let h = unsafe {
        ShellExecuteW(
            0,
            verb.as_ptr(),
            file.as_ptr(),
            params.as_ptr(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    h > 32
}

// ---------------------------------------------------------------------------
// Mémoire
// ---------------------------------------------------------------------------

/// Mémoire physique disponible, sans construire de snapshot complet.
pub fn memory_available() -> u64 {
    let mut m = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as DWORD,
        ..Default::default()
    };
    unsafe {
        GlobalMemoryStatusEx(&mut m);
    }
    m.ullAvailPhys
}

fn memory_info(procs: &[ProcessInfo]) -> MemoryInfo {
    let mut m = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as DWORD,
        ..Default::default()
    };
    unsafe {
        GlobalMemoryStatusEx(&mut m);
    }
    let mut pi = PERFORMANCE_INFORMATION {
        cb: std::mem::size_of::<PERFORMANCE_INFORMATION>() as DWORD,
        ..Default::default()
    };
    let has_pi = unsafe { GetPerformanceInfo(&mut pi, pi.cb) } != 0;
    let page = if pi.PageSize > 0 {
        pi.PageSize as u64
    } else {
        4096
    };

    let compressed = procs
        .iter()
        .filter(|p| p.name.eq_ignore_ascii_case("memory compression"))
        .map(|p| p.working_set)
        .sum();

    MemoryInfo {
        total: m.ullTotalPhys,
        available: m.ullAvailPhys,
        used: m.ullTotalPhys.saturating_sub(m.ullAvailPhys),
        load: m.dwMemoryLoad,
        commit_used: if has_pi {
            pi.CommitTotal as u64 * page
        } else {
            0
        },
        commit_limit: if has_pi {
            pi.CommitLimit as u64 * page
        } else {
            m.ullTotalPageFile
        },
        kernel: if has_pi {
            pi.KernelTotal as u64 * page
        } else {
            0
        },
        cache: if has_pi {
            pi.SystemCache as u64 * page
        } else {
            0
        },
        compressed,
        sum_private: procs.iter().map(|p| p.private).sum(),
        sum_working_set: procs.iter().map(|p| p.working_set).sum(),
    }
}

// ---------------------------------------------------------------------------
// Processus
// ---------------------------------------------------------------------------

struct RawProc {
    pid: u32,
    ppid: u32,
    name: String,
    threads: u32,
}

fn raw_processes() -> Vec<RawProc> {
    let mut out = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE || snap == 0 {
            return out;
        }
        let mut e = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as DWORD,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut e) != 0 {
            loop {
                out.push(RawProc {
                    pid: e.th32ProcessID,
                    ppid: e.th32ParentProcessID,
                    name: from_wide_buf(&e.szExeFile),
                    threads: e.cntThreads,
                });
                if Process32NextW(snap, &mut e) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    out
}

fn process_path(h: HANDLE) -> String {
    let mut buf = [0u16; 1024];
    let mut n = buf.len() as DWORD;
    if unsafe { QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut n) } != 0 {
        from_wide_buf(&buf[..n as usize])
    } else {
        String::new()
    }
}

fn process_created(h: HANDLE) -> u64 {
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    if unsafe { GetProcessTimes(h, &mut creation, &mut exit, &mut kernel, &mut user) } != 0 {
        ((creation.dwHighDateTime as u64) << 32) | creation.dwLowDateTime as u64
    } else {
        0
    }
}

fn process_mem(h: HANDLE) -> (u64, u64) {
    let mut c = PROCESS_MEMORY_COUNTERS_EX {
        cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as DWORD,
        ..Default::default()
    };
    if unsafe { K32GetProcessMemoryInfo(h, &mut c, c.cb) } != 0 {
        (c.WorkingSetSize as u64, c.PrivateUsage as u64)
    } else {
        (0, 0)
    }
}

extern "system" fn enum_win_cb(hwnd: HWND, lparam: isize) -> BOOL {
    unsafe {
        let set = &mut *(lparam as *mut HashSet<u32>);
        if IsWindowVisible(hwnd) != 0 && GetWindowTextLengthW(hwnd) > 0 {
            let mut pid: DWORD = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid != 0 {
                set.insert(pid);
            }
        }
    }
    1
}

fn pids_with_windows() -> HashSet<u32> {
    let mut set: HashSet<u32> = HashSet::new();
    unsafe {
        EnumWindows(enum_win_cb, &mut set as *mut _ as isize);
    }
    set
}

/// Lit la description et l'éditeur depuis les ressources de version du fichier.
fn version_info(path: &str) -> (String, String) {
    if path.is_empty() {
        return (String::new(), String::new());
    }
    let w = to_wide(path);
    let mut h: DWORD = 0;
    let size = unsafe { GetFileVersionInfoSizeW(w.as_ptr(), &mut h) };
    if size == 0 {
        return (String::new(), String::new());
    }
    let mut buf = vec![0u8; size as usize];
    if unsafe {
        GetFileVersionInfoW(
            w.as_ptr(),
            0,
            size,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
        )
    } == 0
    {
        return (String::new(), String::new());
    }

    let mut blocks: Vec<String> = Vec::new();
    // Table de traduction (langue + page de code)
    let sub = to_wide("\\VarFileInfo\\Translation");
    let mut p: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut len: u32 = 0;
    if unsafe { VerQueryValueW(buf.as_ptr() as *const _, sub.as_ptr(), &mut p, &mut len) } != 0
        && !p.is_null()
        && len >= 4
    {
        let n = (len / 4) as usize;
        let tr = unsafe { std::slice::from_raw_parts(p as *const u32, n) };
        for t in tr {
            blocks.push(format!("{:04x}{:04x}", t & 0xFFFF, (t >> 16) & 0xFFFF));
        }
    }
    blocks.push("040904b0".into());
    blocks.push("040c04b0".into());
    blocks.push("000004b0".into());

    let query = |field: &str| -> String {
        for b in &blocks {
            let sub = to_wide(&format!("\\StringFileInfo\\{b}\\{field}"));
            let mut p: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut len: u32 = 0;
            if unsafe { VerQueryValueW(buf.as_ptr() as *const _, sub.as_ptr(), &mut p, &mut len) }
                != 0
                && !p.is_null()
                && len > 0
            {
                let s = from_wide(p as *const u16);
                if !s.trim().is_empty() {
                    return s.trim().to_string();
                }
            }
        }
        String::new()
    };

    (query("FileDescription"), query("CompanyName"))
}

// ---------------------------------------------------------------------------
// Services
// ---------------------------------------------------------------------------

struct RawService {
    name: String,
    display: String,
    state: u32,
    pid: u32,
}

fn raw_services() -> (Vec<RawService>, Option<String>) {
    let mut out = Vec::new();
    unsafe {
        let scm = OpenSCManagerW(
            std::ptr::null(),
            std::ptr::null(),
            SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE,
        );
        if scm == 0 {
            return (
                out,
                Some(format!(
                    "Services inaccessibles : {}",
                    err_msg(last_error())
                )),
            );
        }
        let mut needed: DWORD = 0;
        let mut count: DWORD = 0;
        let mut resume: DWORD = 0;
        let mut buf = vec![0u8; 256 * 1024];
        loop {
            let ok = EnumServicesStatusExW(
                scm,
                SC_ENUM_PROCESS_INFO,
                SERVICE_WIN32,
                SERVICE_STATE_ALL,
                buf.as_mut_ptr(),
                buf.len() as DWORD,
                &mut needed,
                &mut count,
                &mut resume,
                std::ptr::null(),
            );
            if ok != 0 {
                let items = std::slice::from_raw_parts(
                    buf.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW,
                    count as usize,
                );
                for it in items {
                    out.push(RawService {
                        name: from_wide(it.lpServiceName),
                        display: from_wide(it.lpDisplayName),
                        state: it.ServiceStatusProcess.dwCurrentState,
                        pid: it.ServiceStatusProcess.dwProcessId,
                    });
                }
                break;
            }
            let e = last_error();
            if e == 234 && needed as usize > buf.len() {
                buf.resize(needed as usize + 16384, 0);
                resume = 0;
                continue;
            }
            CloseServiceHandle(scm);
            return (
                out,
                Some(format!(
                    "Énumération des services incomplète : {}",
                    err_msg(e)
                )),
            );
        }
        CloseServiceHandle(scm);
    }
    (out, None)
}

fn service_config(name: &str) -> Option<(StartType, String, String, String)> {
    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm == 0 {
            return None;
        }
        let w = to_wide(name);
        let svc = OpenServiceW(scm, w.as_ptr(), SERVICE_QUERY_CONFIG | SERVICE_QUERY_STATUS);
        if svc == 0 {
            CloseServiceHandle(scm);
            return None;
        }
        let mut needed: DWORD = 0;
        let mut buf = vec![0u8; 8192];
        let mut start = StartType::Manual;
        let mut path = String::new();
        let mut account = String::new();
        let mut display = String::new();
        if QueryServiceConfigW(
            svc,
            buf.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW,
            buf.len() as DWORD,
            &mut needed,
        ) != 0
        {
            let cfg = &*(buf.as_ptr() as *const QUERY_SERVICE_CONFIGW);
            start = match cfg.dwStartType {
                0 | 1 => StartType::Auto,
                2 => StartType::Auto,
                3 => StartType::Manual,
                4 => StartType::Disabled,
                _ => StartType::Manual,
            };
            path = from_wide(cfg.lpBinaryPathName);
            account = from_wide(cfg.lpServiceStartName);
            display = from_wide(cfg.lpDisplayName);
            if cfg.dwStartType == 2 {
                // démarrage automatique différé ?
                let mut dbuf = [0u8; 8];
                let mut dneeded: DWORD = 0;
                if QueryServiceConfig2W(
                    svc,
                    SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
                    dbuf.as_mut_ptr(),
                    dbuf.len() as DWORD,
                    &mut dneeded,
                ) != 0
                {
                    let info = &*(dbuf.as_ptr() as *const SERVICE_DELAYED_AUTO_START_INFO);
                    if info.fDelayedAutostart != 0 {
                        start = StartType::Delayed;
                    }
                }
            }
        }
        let mut description = String::new();
        let mut dbuf = vec![0u8; 8192];
        let mut dneeded: DWORD = 0;
        if QueryServiceConfig2W(
            svc,
            SERVICE_CONFIG_DESCRIPTION,
            dbuf.as_mut_ptr(),
            dbuf.len() as DWORD,
            &mut dneeded,
        ) != 0
        {
            let d = &*(dbuf.as_ptr() as *const SERVICE_DESCRIPTIONW);
            description = from_wide(d.lpDescription);
        }
        CloseServiceHandle(svc);
        CloseServiceHandle(scm);
        let _ = display;
        Some((start, path, account, description))
    }
}

fn service_dll(name: &str) -> String {
    let key = format!("{SERVICES_KEY}\\{name}\\Parameters");
    RegKey::open(HKLM, &key, KEY_READ)
        .and_then(|k| k.get_string("ServiceDll"))
        .unwrap_or_default()
}

fn service_state_label(s: u32) -> &'static str {
    match s {
        1 => "Arrêté",
        2 => "Démarrage",
        3 => "Arrêt en cours",
        4 => "En cours",
        5 => "Reprise",
        6 => "Pause",
        7 => "En pause",
        _ => "Inconnu",
    }
}

// ---------------------------------------------------------------------------
// Démarrage
// ---------------------------------------------------------------------------

fn startup_folder_user() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    Path::new(&appdata).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")
}
fn startup_folder_common() -> PathBuf {
    let pd = std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".into());
    Path::new(&pd).join("Microsoft\\Windows\\Start Menu\\Programs\\StartUp")
}

/// Extrait approximativement la cible d'un raccourci .lnk (sans COM).
fn lnk_target(path: &Path) -> String {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let mut best = String::new();
    // ASCII
    let ascii: Vec<u8> = data
        .iter()
        .map(|&b| if (32..127).contains(&b) { b } else { 0 })
        .collect();
    let s = String::from_utf8_lossy(&ascii);
    for m in s.split('\0') {
        if m.len() > 5
            && m.contains(":\\")
            && m.to_lowercase().ends_with(".exe")
            && m.len() > best.len()
        {
            best = m.to_string();
        }
    }
    if best.is_empty() {
        let u: Vec<u16> = data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        let wide = String::from_utf16_lossy(&u);
        for m in wide.split('\0') {
            let m = m.trim_matches(|c: char| c.is_control());
            if m.len() > 5
                && m.contains(":\\")
                && m.to_lowercase().ends_with(".exe")
                && m.len() > best.len()
            {
                best = m.to_string();
            }
        }
    }
    best
}

fn approved_state(root: HKEY, subkey: &str, value: &str) -> Option<bool> {
    let key = format!("{APPROVED_KEY}\\{subkey}");
    let k = RegKey::open(root, &key, KEY_READ)?;
    let (_, data) = k.get(value)?;
    if data.is_empty() {
        return None;
    }
    Some(data[0] & 1 == 0)
}

fn set_approved(root: HKEY, subkey: &str, value: &str, enabled: bool) -> Result<(), String> {
    let key = format!("{APPROVED_KEY}\\{subkey}");
    let k = RegKey::create(root, &key, KEY_WRITE | KEY_READ).ok_or_else(|| {
        format!(
            "clé StartupApproved inaccessible ({})",
            err_msg(last_error())
        )
    })?;
    let mut data = [0u8; 12];
    data[0] = if enabled { 2 } else { 3 };
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ft = (secs + 11_644_473_600) * 10_000_000;
    data[4..12].copy_from_slice(&ft.to_le_bytes());
    k.set_binary(value, &data)
}

#[allow(dead_code)]
struct RawStartup {
    id: String,
    name: String,
    command: String,
    location: String,
    root: HKEY,
    approved_sub: String,
    enabled: bool,
}

fn raw_startup() -> Vec<RawStartup> {
    let mut out = Vec::new();
    let sources: [(HKEY, &str, &str, &str); 4] = [
        (HKCU, RUN_KEY, "Run", "Registre (utilisateur)"),
        (HKLM, RUN_KEY, "Run", "Registre (machine)"),
        (
            HKLM,
            "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run",
            "Run",
            "Registre (machine 32 bits)",
        ),
        (HKCU, RUNONCE_KEY, "RunOnce", "Registre (une fois)"),
    ];
    for (root, key, sub, loc) in sources {
        if let Some(k) = RegKey::open(root, key, KEY_READ) {
            let hive = if root == HKCU { "hkcu" } else { "hklm" };
            let wow = if key.contains("WOW6432Node") {
                "_wow"
            } else {
                ""
            };
            for (name, typ, data) in k.values() {
                if name.is_empty() || (typ != REG_SZ && typ != REG_EXPAND_SZ) {
                    continue;
                }
                let u: Vec<u16> = data
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| u16::from_le_bytes(*c))
                    .collect();
                let cmd = expand(&from_wide_buf(&u));
                let enabled = approved_state(root, sub, &name).unwrap_or(true);
                out.push(RawStartup {
                    id: format!("{hive}{wow}_run|{name}"),
                    name: name.clone(),
                    command: cmd,
                    location: loc.to_string(),
                    root,
                    approved_sub: sub.to_string(),
                    enabled,
                });
            }
        }
    }
    // Dossiers Démarrage
    for (folder, loc, hive, root) in [
        (
            startup_folder_user(),
            "Dossier Démarrage (utilisateur)",
            "hkcu",
            HKCU,
        ),
        (
            startup_folder_common(),
            "Dossier Démarrage (tous les utilisateurs)",
            "hklm",
            HKLM,
        ),
    ] {
        if let Ok(rd) = std::fs::read_dir(&folder) {
            for e in rd.flatten() {
                let p = e.path();
                let fname = e.file_name().to_string_lossy().to_string();
                if fname.eq_ignore_ascii_case("desktop.ini") {
                    continue;
                }
                let cmd = if p.extension().map(|x| x == "lnk").unwrap_or(false) {
                    let t = lnk_target(&p);
                    if t.is_empty() {
                        format!("(raccourci) {}", p.display())
                    } else {
                        t
                    }
                } else {
                    p.to_string_lossy().to_string()
                };
                let enabled = approved_state(root, "StartupFolder", &fname).unwrap_or(true);
                out.push(RawStartup {
                    id: format!("{hive}_folder|{fname}"),
                    name: fname,
                    command: cmd,
                    location: loc.to_string(),
                    root,
                    approved_sub: "StartupFolder".to_string(),
                    enabled,
                });
            }
        }
    }
    out
}

fn parse_startup_id(id: &str) -> Option<(HKEY, String, String)> {
    let (hive, name) = id.split_once('|')?;
    let (root, sub) = match hive {
        "hkcu_run" | "hkcu_runonce" => (HKCU, "Run"),
        "hklm_run" => (HKLM, "Run"),
        "hklm_wow_run" => (HKLM, "Run"),
        "hkcu_folder" => (HKCU, "StartupFolder"),
        "hklm_folder" => (HKLM, "StartupFolder"),
        _ => return None,
    };
    Some((root, sub.to_string(), name.to_string()))
}

// ---------------------------------------------------------------------------
// Tâches planifiées (lecture directe des XML, indépendante de la langue)
// ---------------------------------------------------------------------------

/// Chemin absolu d'un outil système. Ne jamais laisser Windows résoudre
/// `schtasks.exe` ou `explorer.exe` depuis le répertoire courant, le PATH ou
/// une variable d'environnement héritée.
pub(crate) fn system32_path(name: &str) -> Result<PathBuf, String> {
    let mut buf = vec![0u16; 32_768];
    let len = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as DWORD) };
    if len == 0 {
        return Err(format!(
            "répertoire système introuvable : {}",
            err_msg(last_error())
        ));
    }
    if len as usize >= buf.len() {
        return Err("répertoire système trop long".into());
    }
    buf.truncate(len as usize);
    Ok(PathBuf::from(from_wide_buf(&buf)).join(name))
}

fn tasks_dir() -> Result<PathBuf, String> {
    system32_path("Tasks")
}

fn walk_tasks(dir: &Path, base: &Path, out: &mut Vec<PathBuf>, denied: &mut u32) {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => {
            *denied += 1;
            return;
        }
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk_tasks(&p, base, out, denied);
        } else {
            let _ = base;
            out.push(p);
        }
    }
}

fn trigger_label(n: &str) -> &'static str {
    match n {
        "LogonTrigger" => "À l'ouverture de session",
        "BootTrigger" => "Au démarrage du PC",
        "CalendarTrigger" => "Planifié (date/heure)",
        "TimeTrigger" => "À une heure fixe",
        "EventTrigger" => "Sur un événement",
        "IdleTrigger" => "Quand le PC est inactif",
        "RegistrationTrigger" => "À l'enregistrement",
        "SessionStateChangeTrigger" => "Au changement de session",
        "WnfStateChangeTrigger" => "Sur notification système",
        "MaintenanceTrigger" => "Maintenance automatique",
        _ => "Autre déclencheur",
    }
}

fn task_from_xml(xml: &str, path: &str) -> Option<TaskItem> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    let root = doc.root_element();
    let child_text = |node: roxmltree::Node, name: &str| -> String {
        node.children()
            .find(|c| c.has_tag_name(name))
            .and_then(|c| c.text())
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let reg = root.children().find(|c| c.has_tag_name("RegistrationInfo"));
    let (author, description) = match reg {
        Some(r) => (child_text(r, "Author"), child_text(r, "Description")),
        None => (String::new(), String::new()),
    };
    let enabled = root
        .children()
        .find(|c| c.has_tag_name("Settings"))
        .and_then(|s| s.children().find(|c| c.has_tag_name("Enabled")))
        .and_then(|c| c.text())
        .map(|t| !t.trim().eq_ignore_ascii_case("false"))
        .unwrap_or(true);
    let mut triggers = Vec::new();
    if let Some(t) = root.children().find(|c| c.has_tag_name("Triggers")) {
        for c in t.children().filter(|c| c.is_element()) {
            triggers.push(trigger_label(c.tag_name().name()).to_string());
        }
    }
    let mut command = String::new();
    if let Some(a) = root.children().find(|c| c.has_tag_name("Actions")) {
        for exec in a.children().filter(|c| c.is_element()) {
            let cmd = child_text(exec, "Command");
            let args = child_text(exec, "Arguments");
            let data = child_text(exec, "Data");
            let full = [cmd, args, data]
                .iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            if !full.is_empty() {
                if command.is_empty() {
                    command = full;
                } else {
                    command.push_str(" ; ");
                    command.push_str(&full);
                }
            }
        }
    }
    let name = path.rsplit('\\').next().unwrap_or(path).to_string();
    let advice = rules::task_advice(path, &command, enabled, &triggers);
    Some(TaskItem {
        path: path.to_string(),
        name,
        enabled,
        triggers,
        command,
        author,
        description,
        advice,
    })
}

/// Repli sans administrateur : `schtasks /query /xml` expose les tâches que
/// l'annuaire `System32\\Tasks` refuse de lire, sous forme de XML encadrés par
/// des commentaires `<!-- \\Chemin -->`. L'analyse reste indépendante de la langue.
fn tasks_from_schtasks_xml() -> Vec<TaskItem> {
    let Ok(schtasks) = system32_path("schtasks.exe") else {
        return Vec::new();
    };
    let output = match std::process::Command::new(schtasks)
        .args(["/query", "/xml"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);
    tasks_from_schtasks_output(&text)
}

/// Transforme la sortie de `schtasks /query /xml` en tâches. Séparé de
/// l'exécution pour pouvoir le tester sans lancer un processus ni toucher
/// aux tâches réelles de la machine.
fn tasks_from_schtasks_output(text: &str) -> Vec<TaskItem> {
    let mut out = Vec::new();
    for block in text.split("<!--").skip(1) {
        let Some((comment, rest)) = block.split_once("-->") else {
            continue;
        };
        let path = comment.trim();
        if !path.starts_with('\\') {
            continue;
        }
        let Some(start) = rest.find("<Task") else {
            continue;
        };
        let Some(end) = rest.rfind("</Task>") else {
            continue;
        };
        if let Some(item) = task_from_xml(&rest[start..end + 7], path) {
            out.push(item);
        }
    }
    out
}

fn task_key(path: &str) -> String {
    path.trim().replace('/', "\\").to_lowercase()
}

fn needs_task_fallback(out_len: usize, unreadable: u32, denied: u32) -> bool {
    out_len == 0 || unreadable > 0 || denied > 0
}

fn merge_tasks(primary: Vec<TaskItem>, fallback: Vec<TaskItem>) -> Vec<TaskItem> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(primary.len() + fallback.len());
    for task in primary.into_iter().chain(fallback) {
        if seen.insert(task_key(&task.path)) {
            out.push(task);
        }
    }
    out
}

fn collect_tasks(notes: &mut Vec<String>) -> Vec<TaskItem> {
    let base = match tasks_dir() {
        Ok(base) => base,
        Err(e) => {
            notes.push(format!("tâches planifiées indisponibles : {e}"));
            return Vec::new();
        }
    };
    let mut files = Vec::new();
    let mut denied = 0u32;
    walk_tasks(&base, &base, &mut files, &mut denied);
    let mut out = Vec::new();
    let mut unreadable = 0u32;
    for f in files {
        let rel = f.strip_prefix(&base).unwrap_or(&f);
        let path = format!("\\{}", rel.to_string_lossy().replace('/', "\\"));
        let xml = match std::fs::read_to_string(&f) {
            Ok(s) => s,
            Err(_) => {
                unreadable += 1;
                continue;
            }
        };
        if let Some(item) = task_from_xml(&xml, &path) {
            out.push(item);
        }
    }
    let mut fallback_used = false;
    if needs_task_fallback(out.len(), unreadable, denied) {
        let fallback = tasks_from_schtasks_xml();
        fallback_used = !fallback.is_empty();
        if fallback_used {
            out = merge_tasks(out, fallback);
        }
    }
    if (unreadable > 0 || denied > 0) && !fallback_used {
        notes.push(format!(
            "{} tâche(s) planifiée(s) non lisibles sans droits administrateur : passe en mode admin pour la liste complète.",
            unreadable + denied
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// Collecte complète
// ---------------------------------------------------------------------------

pub fn collect() -> Snapshot {
    let t0 = std::time::Instant::now();
    let mut notes: Vec<String> = Vec::new();
    let admin = is_admin();
    if !admin {
        notes.push(
            "Mode lecture seule : lance RamScope en administrateur pour appliquer les changements."
                .into(),
        );
    }

    // --- processus ---
    let raw = raw_processes();
    let windowed = pids_with_windows();
    let mut vcache: HashMap<String, (String, String)> = HashMap::new();
    let mut procs: Vec<ProcessInfo> = Vec::new();
    for r in &raw {
        let mut path = String::new();
        let mut created = 0u64;
        let mut ws = 0u64;
        let mut priv_ = 0u64;
        unsafe {
            let h = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
                0,
                r.pid,
            );
            let h = if h == 0 {
                OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, r.pid)
            } else {
                h
            };
            if h != 0 {
                path = process_path(h);
                created = process_created(h);
                let m = process_mem(h);
                ws = m.0;
                priv_ = m.1;
                CloseHandle(h);
            }
        }
        let (desc, company) = if path.is_empty() {
            (String::new(), String::new())
        } else if let Some(v) = vcache.get(&path) {
            v.clone()
        } else {
            let v = version_info(&path);
            vcache.insert(path.clone(), v.clone());
            v
        };
        procs.push(ProcessInfo {
            pid: r.pid,
            ppid: r.ppid,
            name: r.name.clone(),
            description: desc,
            company,
            path,
            created_at: created,
            working_set: ws,
            private: priv_,
            threads: r.threads,
            has_window: windowed.contains(&r.pid),
            services: Vec::new(),
            category: String::new(),
            advice: Advice::new(Level::Unknown, ""),
        });
    }

    // --- services ---
    let (raw_svc, svc_note) = raw_services();
    if let Some(n) = svc_note {
        notes.push(n);
    }
    let ram_by_pid: HashMap<u32, u64> = procs.iter().map(|p| (p.pid, p.working_set)).collect();
    let mut host_count: HashMap<u32, u32> = HashMap::new();
    for s in &raw_svc {
        if s.pid > 0 && s.state == SERVICE_RUNNING {
            *host_count.entry(s.pid).or_insert(0) += 1;
        }
    }
    let mut services: Vec<ServiceInfo> = Vec::new();
    for s in raw_svc {
        let (start, path, account, description) = service_config(&s.name).unwrap_or((
            StartType::Manual,
            String::new(),
            String::new(),
            String::new(),
        ));
        let running = s.state == SERVICE_RUNNING;
        let shared = if s.pid > 0 {
            *host_count.get(&s.pid).unwrap_or(&1)
        } else {
            1
        };
        let host_ram = if s.pid > 0 {
            *ram_by_pid.get(&s.pid).unwrap_or(&0)
        } else {
            0
        };
        let ram = if running && shared > 0 {
            host_ram / shared as u64
        } else {
            0
        };
        let dll = if path.to_lowercase().contains("svchost") {
            service_dll(&s.name)
        } else {
            String::new()
        };
        let protected = rules::is_protected_service(&s.name);
        let advice = rules::service_advice(&s.name, &s.display, &path, start, running, ram);
        services.push(ServiceInfo {
            name: s.name.clone(),
            display: if s.display.is_empty() {
                s.name.clone()
            } else {
                s.display.clone()
            },
            description,
            running,
            state: service_state_label(s.state).to_string(),
            start,
            pid: s.pid,
            path,
            dll,
            account,
            host_ram,
            host_shared: shared,
            ram,
            protected,
            advice,
        });
        if s.pid > 0 {
            if let Some(p) = procs.iter_mut().find(|p| p.pid == s.pid) {
                p.services.push(s.name.clone());
            }
        }
    }

    // --- conseils processus (après association des services) ---
    for p in procs.iter_mut() {
        let mut a = rules::process_advice(
            &p.name,
            &p.path,
            &p.description,
            &p.company,
            p.private,
            p.has_window,
            &p.services,
        );
        if let Some(Action::Kill {
            pid,
            image_path,
            created_at,
            ..
        }) = &mut a.action
        {
            *pid = p.pid;
            *image_path = p.path.clone();
            *created_at = p.created_at;
        }
        if rules::is_protected_process(&p.name) {
            a.action = None;
            a.gain = Gain::unavailable();
        }
        p.advice = a;
        p.category = if !p.services.is_empty() {
            "Hôte de services".into()
        } else if p.path.to_lowercase().contains("\\windows\\") {
            "Système".into()
        } else if p.has_window {
            "Application".into()
        } else {
            "Arrière-plan".into()
        };
    }

    // --- démarrage ---
    let exe_ram: HashMap<String, u64> = {
        let mut m: HashMap<String, u64> = HashMap::new();
        for p in &procs {
            *m.entry(p.name.to_lowercase()).or_insert(0) += p.private;
        }
        m
    };
    let mut startup: Vec<StartupItem> = Vec::new();
    for r in raw_startup() {
        let exe = crate::util::extract_exe(&r.command);
        let ram = *exe_ram.get(&exe.to_lowercase()).unwrap_or(&0);
        let mut advice = rules::startup_advice(&r.name, &r.command, r.enabled, ram);
        if let Some(Action::Startup { id, .. }) = &mut advice.action {
            *id = r.id.clone();
        }
        startup.push(StartupItem {
            id: r.id,
            name: r.name,
            command: r.command,
            location: r.location,
            enabled: r.enabled,
            ram,
            advice,
        });
    }

    // --- tâches ---
    let tasks = collect_tasks(&mut notes);

    let memory = memory_info(&procs);
    procs.sort_by_key(|p| std::cmp::Reverse(p.private));
    services.sort_by(|a, b| {
        a.advice
            .level
            .rank()
            .cmp(&b.advice.level.rank())
            .then(b.ram.cmp(&a.ram))
    });
    startup.sort_by(|a, b| {
        a.advice
            .level
            .rank()
            .cmp(&b.advice.level.rank())
            .then(b.ram.cmp(&a.ram))
    });

    let mut counts = Counts {
        processes: procs.len(),
        services: services.len(),
        services_auto: services
            .iter()
            .filter(|s| matches!(s.start, StartType::Auto | StartType::Delayed))
            .count(),
        services_running: services.iter().filter(|s| s.running).count(),
        startup_enabled: startup.iter().filter(|s| s.enabled).count(),
        tasks_enabled: tasks.iter().filter(|t| t.enabled).count(),
        actionable: 0,
    };
    let mut planned: Vec<plan::GainEntry> = Vec::new();
    for s in services.iter().filter(|s| s.advice.level.preselected()) {
        if let Some(action) = &s.advice.action {
            counts.actionable += 1;
            let host = if s.host_shared > 1 && s.pid > 0 {
                Some(plan::HostGain {
                    pid: s.pid,
                    services: s.host_shared,
                    host_ram: s.host_ram,
                })
            } else {
                None
            };
            planned.push(plan::GainEntry {
                action: action.clone(),
                gain: s.advice.gain,
                gain_next_boot: s.advice.gain_next_boot,
                host,
            });
        }
    }
    for s in startup.iter().filter(|s| s.advice.level.preselected()) {
        if let Some(action) = &s.advice.action {
            counts.actionable += 1;
            planned.push(plan::GainEntry {
                action: action.clone(),
                gain: s.advice.gain,
                gain_next_boot: s.advice.gain_next_boot,
                host: None,
            });
        }
    }
    let (plan_gain, plan_boot) = plan::total_gains(&planned);
    counts.actionable += tasks
        .iter()
        .filter(|t| t.advice.action.is_some() && t.advice.level.preselected())
        .count();

    Snapshot {
        meta: Meta {
            computer: computer_name(),
            user: user_name(),
            os: os_name(),
            uptime_secs: unsafe { GetTickCount64() } / 1000,
            is_admin: admin,
            demo: false,
            collected_at: crate::util::now_string(),
            duration_ms: t0.elapsed().as_millis() as u64,
            version: env!("CARGO_PKG_VERSION").to_string(),
            notes,
        },
        memory,
        processes: procs,
        services,
        startup,
        tasks,
        plan_gain,
        plan_gain_next_boot: plan_boot,
        counts,
    }
}

// ---------------------------------------------------------------------------
// Application des actions
// ---------------------------------------------------------------------------

fn apply_service(name: &str, target: StartType) -> Result<(), String> {
    if rules::is_protected_service(name) {
        return Err("service vital : modification refusée par RamScope".into());
    }
    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm == 0 {
            return Err(format!("SCManager : {}", err_msg(last_error())));
        }
        let w = to_wide(name);
        let svc = OpenServiceW(scm, w.as_ptr(), SERVICE_CHANGE_CONFIG);
        if svc == 0 {
            let e = last_error();
            CloseServiceHandle(scm);
            return Err(err_msg(e));
        }
        let ok = ChangeServiceConfigW(
            svc,
            SERVICE_NO_CHANGE,
            target.code(),
            SERVICE_NO_CHANGE,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        let e = last_error();
        // Démarrage différé : drapeau séparé (clé DelayedAutostart)
        if ok != 0 {
            if let Some(k) = RegKey::create(HKLM, &format!("{SERVICES_KEY}\\{name}"), KEY_SET_VALUE)
            {
                let v: u32 = if target == StartType::Delayed { 1 } else { 0 };
                let wv = to_wide("DelayedAutostart");
                let _ = RegSetValueExW(k.0, wv.as_ptr(), 0, REG_DWORD, v.to_le_bytes().as_ptr(), 4);
            }
        }
        CloseServiceHandle(svc);
        CloseServiceHandle(scm);
        if ok != 0 {
            Ok(())
        } else {
            Err(err_msg(e))
        }
    }
}

fn control_service(name: &str, stop: bool) -> Result<(), String> {
    if stop && rules::is_protected_service(name) {
        return Err("service vital : arrêt refusé par RamScope".into());
    }
    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm == 0 {
            return Err(format!("SCManager : {}", err_msg(last_error())));
        }
        let w = to_wide(name);
        let access = if stop {
            SERVICE_STOP | SERVICE_QUERY_STATUS
        } else {
            SERVICE_START | SERVICE_QUERY_STATUS
        };
        let svc = OpenServiceW(scm, w.as_ptr(), access);
        if svc == 0 {
            let e = last_error();
            CloseServiceHandle(scm);
            return Err(err_msg(e));
        }
        let ok = if stop {
            let mut st = SERVICE_STATUS_PROCESS::default();
            ControlService(svc, SERVICE_CONTROL_STOP, &mut st)
        } else {
            StartServiceW(svc, 0, std::ptr::null())
        };
        let e = last_error();
        CloseServiceHandle(svc);
        CloseServiceHandle(scm);
        if ok != 0 {
            Ok(())
        } else {
            Err(err_msg(e))
        }
    }
}

fn service_is_running(name: &str) -> bool {
    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm == 0 {
            return false;
        }
        let w = to_wide(name);
        let svc = OpenServiceW(scm, w.as_ptr(), SERVICE_QUERY_STATUS);
        if svc == 0 {
            CloseServiceHandle(scm);
            return false;
        }
        let mut st = SERVICE_STATUS_PROCESS::default();
        let mut needed: DWORD = 0;
        let ok = QueryServiceStatusEx(
            svc,
            0,
            &mut st as *mut _ as *mut u8,
            std::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
            &mut needed,
        );
        CloseServiceHandle(svc);
        CloseServiceHandle(scm);
        ok != 0 && st.dwCurrentState == SERVICE_RUNNING
    }
}

fn apply_startup(id: &str, enabled: bool) -> Result<(), String> {
    let (root, sub, name) = parse_startup_id(id).ok_or("identifiant de démarrage invalide")?;
    set_approved(root, &sub, &name, enabled)
}

fn current_startup_state(id: &str) -> Option<bool> {
    let (root, sub, name) = parse_startup_id(id)?;
    Some(approved_state(root, &sub, &name).unwrap_or(true))
}

fn current_service_start(name: &str) -> Option<StartType> {
    service_config(name).map(|(s, ..)| s)
}

fn run_schtasks(path: &str, enable: bool) -> Result<(), String> {
    let flag = if enable { "/enable" } else { "/disable" };
    let schtasks =
        system32_path("schtasks.exe").map_err(|e| format!("schtasks introuvable : {e}"))?;
    let out = std::process::Command::new(schtasks)
        .args(["/change", "/tn", path, flag])
        .output()
        .map_err(|e| format!("schtasks introuvable : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let msg2 = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Err(if msg.is_empty() {
            if msg2.is_empty() {
                "schtasks a refusé la modification (droits admin ?)".into()
            } else {
                msg2
            }
        } else {
            msg
        })
    }
}

fn current_task_enabled(path: &str) -> Option<bool> {
    let f = tasks_dir().ok()?.join(path.trim_start_matches('\\'));
    let xml = std::fs::read_to_string(f).ok()?;
    let doc = roxmltree::Document::parse(&xml).ok()?;
    Some(
        doc.root_element()
            .children()
            .find(|c| c.has_tag_name("Settings"))
            .and_then(|s| s.children().find(|c| c.has_tag_name("Enabled")))
            .and_then(|c| c.text())
            .map(|t| !t.trim().eq_ignore_ascii_case("false"))
            .unwrap_or(true),
    )
}

fn kill_process(pid: u32, name: &str, image_path: &str, created_at: u64) -> Result<(), String> {
    if rules::is_protected_process(name) || pid <= 8 {
        return Err("processus système protégé : refusé".into());
    }
    if pid == unsafe { GetCurrentProcessId() } {
        return Err("RamScope ne peut pas se terminer lui-même".into());
    }
    unsafe {
        let h = OpenProcess(
            PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            pid,
        );
        if h == 0 {
            return Err(err_msg(last_error()));
        }
        let path = process_path(h);
        let cree = process_created(h);
        if cree == 0 || path.is_empty() {
            CloseHandle(h);
            return Err("identité du processus illisible : action refusée".into());
        }
        if cree != created_at || !path.eq_ignore_ascii_case(image_path) {
            CloseHandle(h);
            return Err(
                "l'identité du processus a changé depuis l'analyse : action refusée".into(),
            );
        }
        let ok = TerminateProcess(h, 0);
        let e = last_error();
        CloseHandle(h);
        if ok != 0 {
            Ok(())
        } else {
            Err(err_msg(e))
        }
    }
}

/// Applique une liste d'actions et construit les entrées de rollback.
pub fn apply(actions: &[Action]) -> Vec<RollbackEntry> {
    let verdicts = plan::validate(actions, unsafe { GetCurrentProcessId() });
    let mut entries = Vec::new();
    for (a, verdict) in actions.iter().zip(verdicts) {
        // garde-fous serveur, indépendants de l'interface
        if !verdict.allowed {
            entries.push(RollbackEntry {
                label: a.describe(),
                done: a.clone(),
                undo: None,
                ok: false,
                message: verdict.message,
            });
            continue;
        }

        let undo = match a {
            Action::Service { name, .. } => current_service_start(name).map(|s| Action::Service {
                name: name.clone(),
                start: s,
                from: None,
            }),
            Action::Startup { id, .. } => current_startup_state(id).map(|e| Action::Startup {
                id: id.clone(),
                enabled: e,
                was_enabled: None,
            }),
            Action::Task { path, .. } => current_task_enabled(path).map(|e| Action::Task {
                path: path.clone(),
                enabled: e,
                was_enabled: None,
            }),
            Action::StopService { name } => {
                if service_is_running(name) {
                    Some(Action::StartService { name: name.clone() })
                } else {
                    None
                }
            }
            Action::StartService { name } => {
                if service_is_running(name) {
                    None
                } else {
                    Some(Action::StopService { name: name.clone() })
                }
            }
            Action::Kill { .. } => None,
        };

        let res = match a {
            Action::Service { name, start, from } => {
                if let Some(attendu) = from {
                    match current_service_start(name) {
                        Some(courant) if courant != *attendu => Err(format!(
                            "l'état du service « {name} » a changé depuis l'analyse (attendu {}, trouvé {})",
                            attendu.label(),
                            courant.label()
                        )),
                        None => Err(format!("service « {name} » introuvable")),
                        _ => apply_service(name, *start),
                    }
                } else {
                    apply_service(name, *start)
                }
            }
            Action::Startup {
                id,
                enabled,
                was_enabled,
            } => {
                if let Some(attendu) = was_enabled {
                    match current_startup_state(id) {
                        Some(courant) if courant != *attendu => {
                            Err("l'état du démarrage a changé depuis l'analyse".to_string())
                        }
                        None => Err("entrée de démarrage introuvable".to_string()),
                        _ => apply_startup(id, *enabled),
                    }
                } else {
                    apply_startup(id, *enabled)
                }
            }
            Action::Task {
                path,
                enabled,
                was_enabled,
            } => {
                if let Some(attendu) = was_enabled {
                    match current_task_enabled(path) {
                        Some(courant) if courant != *attendu => {
                            Err("l'état de la tâche a changé depuis l'analyse".to_string())
                        }
                        None => Err("tâche introuvable".to_string()),
                        _ => run_schtasks(path, *enabled),
                    }
                } else {
                    run_schtasks(path, *enabled)
                }
            }
            Action::Kill {
                pid,
                name,
                image_path,
                created_at,
            } => kill_process(*pid, name, image_path, *created_at),
            Action::StopService { name } => control_service(name, true),
            Action::StartService { name } => control_service(name, false),
        };

        entries.push(RollbackEntry {
            label: a.describe(),
            done: a.clone(),
            undo: undo.clone(),
            ok: res.is_ok(),
            message: match &res {
                Ok(()) => "OK".into(),
                Err(e) => e.clone(),
            },
        });
    }
    entries
}

pub fn data_dir() -> Result<PathBuf, String> {
    let mut raw: LPWSTR = std::ptr::null_mut();
    let hr = unsafe { SHGetKnownFolderPath(&FOLDERID_LOCALAPPDATA, 0, 0, &mut raw) };
    if hr < 0 || raw.is_null() {
        return Err(format!(
            "dossier LocalAppData introuvable (HRESULT 0x{hr:08X})"
        ));
    }
    let base = from_wide(raw);
    unsafe { CoTaskMemFree(raw as *mut std::ffi::c_void) };
    let d = PathBuf::from(base).join("RamScope");
    std::fs::create_dir_all(d.join("rollbacks"))
        .map_err(|e| format!("impossible de créer le dossier de données : {e}"))?;
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_une_tache_xml() {
        let xml = r#"<Task version="1.2">
          <RegistrationInfo><Author>PC\Ro</Author><Description>test</Description></RegistrationInfo>
          <Triggers><LogonTrigger/></Triggers>
          <Settings><Enabled>false</Enabled></Settings>
          <Actions><Exec><Command>cmd.exe</Command><Arguments>/c echo ok</Arguments></Exec></Actions>
        </Task>"#;
        let t = task_from_xml(xml, "\\Demo\\Tache").expect("XML valide");
        assert_eq!(t.path, "\\Demo\\Tache");
        assert_eq!(t.name, "Tache");
        assert!(!t.enabled);
        assert_eq!(t.triggers, vec!["À l'ouverture de session".to_string()]);
        assert_eq!(t.command, "cmd.exe /c echo ok");
        assert_eq!(t.author, "PC\\Ro");
    }

    fn tache_test(path: &str) -> TaskItem {
        let xml = r#"<Task version="1.2">
          <Settings><Enabled>true</Enabled></Settings>
          <Actions><Exec><Command>cmd.exe</Command></Exec></Actions>
        </Task>"#;
        task_from_xml(xml, path).expect("XML valide")
    }

    use std::ffi::OsString;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        old: Option<OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let old = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn chemins_des_outils_systeme_sont_absolus() {
        let path = system32_path("schtasks.exe").expect("répertoire système disponible");
        assert!(path.is_absolute());
        assert!(path
            .to_string_lossy()
            .to_lowercase()
            .ends_with("system32\\schtasks.exe"));
    }

    #[test]
    fn systemroot_forge_ne_change_pas_le_chemin_systeme() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let normal = system32_path("schtasks.exe").expect("répertoire système disponible");
        let _guard = EnvGuard::set("SystemRoot", r"C:\Temp\X");
        let forged = system32_path("schtasks.exe").expect("répertoire système disponible");
        assert_eq!(normal, forged);
        assert!(!forged.starts_with(r"C:\Temp\X"));
    }

    #[test]
    fn localappdata_forge_ne_change_pas_le_dossier_donnees() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let normal = data_dir().expect("dossier de données disponible");
        let _guard = EnvGuard::set("LOCALAPPDATA", r"C:\Temp\X");
        let forged = data_dir().expect("dossier de données disponible");
        assert_eq!(normal, forged);
        assert!(!forged.starts_with(r"C:\Temp\X"));
    }

    #[test]
    fn elevation_conserve_les_options_et_retire_no_admin() {
        let args: Vec<String> = [
            "--port",
            "8917",
            "--host",
            "localhost",
            "--no-browser",
            "--no-admin",
            "--elevated",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let params = elevated_params(&args);
        assert!(params.contains("--port 8917"));
        assert!(params.contains("--host localhost"));
        assert!(params.contains("--no-browser"));
        assert!(!params.contains("--no-admin"));
        assert_eq!(params.matches("--elevated").count(), 1);
    }

    #[test]
    fn elevation_quote_les_arguments_avec_espaces() {
        let arg = r"C:\Program Files\RamScope";
        assert_eq!(quote_windows_arg(arg), r#""C:\Program Files\RamScope""#);
        let params = elevated_params(&[arg.to_string(), "--port".into(), "8917".into()]);
        assert!(params.starts_with(r#""C:\Program Files\RamScope""#));
        assert!(params.ends_with("--elevated"));
    }

    #[test]
    fn le_repli_couvre_une_acl_partielle() {
        assert!(needs_task_fallback(0, 0, 0));
        assert!(!needs_task_fallback(1, 0, 0));
        assert!(needs_task_fallback(1, 1, 0));
        assert!(needs_task_fallback(1, 0, 1));
    }

    #[test]
    fn fusionne_les_taches_par_chemin_sans_doublon() {
        let direct = vec![tache_test("\\A"), tache_test("\\B")];
        let fallback = vec![tache_test("\\b"), tache_test("\\C")];
        let merged = merge_tasks(direct, fallback);
        let paths: Vec<&str> = merged.iter().map(|t| t.path.as_str()).collect();
        assert_eq!(paths, vec!["\\A", "\\B", "\\C"]);
    }

    #[test]
    fn decode_la_sortie_de_schtasks() {
        let xml = r#"
<!-- \Demo\Active -->
<Task version="1.2">
  <Settings><Enabled>true</Enabled></Settings>
  <Actions><Exec><Command>cmd.exe</Command><Arguments>/c echo actif</Arguments></Exec></Actions>
</Task>
<!-- \Demo\Inactive -->
<Task version="1.2">
  <Settings><Enabled>false</Enabled></Settings>
  <Actions><Exec><Command>cmd.exe</Command></Exec></Actions>
</Task>"#;
        let tasks = tasks_from_schtasks_output(xml);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].path, "\\Demo\\Active");
        assert!(tasks[0].enabled);
        assert_eq!(tasks[0].command, "cmd.exe /c echo actif");
        assert_eq!(tasks[1].path, "\\Demo\\Inactive");
        assert!(!tasks[1].enabled);
    }
}
