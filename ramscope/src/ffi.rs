//! Déclarations Win32 brutes (FFI). Aucune dépendance externe : on lie
//! directement kernel32 / advapi32 / psapi / user32 / version / shell32.
#![allow(non_snake_case, non_camel_case_types, dead_code)]
#![allow(clippy::upper_case_acronyms)] // Alias FFI : les noms Win32 restent tels quels.

use std::ffi::c_void;

pub type HANDLE = isize;
pub type HKEY = isize;
pub type HWND = isize;
pub type BOOL = i32;
pub type DWORD = u32;
pub type WCHAR = u16;
pub type LPWSTR = *mut u16;
pub type LPCWSTR = *const u16;
pub type SIZE_T = usize;

pub const MAX_PATH: usize = 260;
pub const INVALID_HANDLE_VALUE: HANDLE = -1;

pub const PROCESS_TERMINATE: DWORD = 0x0001;
pub const PROCESS_QUERY_INFORMATION: DWORD = 0x0400;
pub const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;
pub const PROCESS_VM_READ: DWORD = 0x0010;
pub const TH32CS_SNAPPROCESS: DWORD = 0x0000_0002;

pub const SC_MANAGER_CONNECT: DWORD = 0x0001;
pub const SC_MANAGER_ENUMERATE_SERVICE: DWORD = 0x0004;
pub const SERVICE_QUERY_CONFIG: DWORD = 0x0001;
pub const SERVICE_CHANGE_CONFIG: DWORD = 0x0002;
pub const SERVICE_QUERY_STATUS: DWORD = 0x0004;
pub const SERVICE_STOP: DWORD = 0x0020;
pub const SERVICE_START: DWORD = 0x0010;
pub const SERVICE_CONTROL_STOP: DWORD = 0x0001;
pub const SERVICE_NO_CHANGE: DWORD = 0xFFFF_FFFF;
pub const SERVICE_WIN32: DWORD = 0x30;
pub const SERVICE_STATE_ALL: DWORD = 0x03;
pub const SC_ENUM_PROCESS_INFO: DWORD = 0;
pub const SERVICE_CONFIG_DESCRIPTION: DWORD = 1;
pub const SERVICE_CONFIG_DELAYED_AUTO_START_INFO: DWORD = 3;
pub const SERVICE_RUNNING: DWORD = 4;

pub const HKLM: HKEY = 0x8000_0002u32 as i32 as isize;
pub const HKCU: HKEY = 0x8000_0001u32 as i32 as isize;
pub const HKU: HKEY = 0x8000_0003u32 as i32 as isize;
pub const KEY_QUERY_VALUE: DWORD = 0x0001;
pub const KEY_SET_VALUE: DWORD = 0x0002;
pub const KEY_CREATE_SUB_KEY: DWORD = 0x0004;
pub const KEY_ENUMERATE_SUB_KEYS: DWORD = 0x0008;
pub const KEY_READ: DWORD = 0x20019;
pub const KEY_WRITE: DWORD = 0x20006;
pub const REG_SZ: DWORD = 1;
pub const REG_EXPAND_SZ: DWORD = 2;
pub const REG_BINARY: DWORD = 3;
pub const REG_DWORD: DWORD = 4;
pub const REG_MULTI_SZ: DWORD = 7;
pub const REG_OPTION_NON_VOLATILE: DWORD = 0;

pub const TOKEN_QUERY: DWORD = 0x0008;
pub const SW_SHOWNORMAL: i32 = 1;
pub const SW_SHOW: i32 = 5;

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct MEMORYSTATUSEX {
    pub dwLength: DWORD,
    pub dwMemoryLoad: DWORD,
    pub ullTotalPhys: u64,
    pub ullAvailPhys: u64,
    pub ullTotalPageFile: u64,
    pub ullAvailPageFile: u64,
    pub ullTotalVirtual: u64,
    pub ullAvailVirtual: u64,
    pub ullAvailExtendedVirtual: u64,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct PERFORMANCE_INFORMATION {
    pub cb: DWORD,
    pub CommitTotal: SIZE_T,
    pub CommitLimit: SIZE_T,
    pub CommitPeak: SIZE_T,
    pub PhysicalTotal: SIZE_T,
    pub PhysicalAvailable: SIZE_T,
    pub SystemCache: SIZE_T,
    pub KernelTotal: SIZE_T,
    pub KernelPaged: SIZE_T,
    pub KernelNonpaged: SIZE_T,
    pub PageSize: SIZE_T,
    pub HandleCount: DWORD,
    pub ProcessCount: DWORD,
    pub ThreadCount: DWORD,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct PROCESS_MEMORY_COUNTERS_EX {
    pub cb: DWORD,
    pub PageFaultCount: DWORD,
    pub PeakWorkingSetSize: SIZE_T,
    pub WorkingSetSize: SIZE_T,
    pub QuotaPeakPagedPoolUsage: SIZE_T,
    pub QuotaPagedPoolUsage: SIZE_T,
    pub QuotaPeakNonPagedPoolUsage: SIZE_T,
    pub QuotaNonPagedPoolUsage: SIZE_T,
    pub PagefileUsage: SIZE_T,
    pub PeakPagefileUsage: SIZE_T,
    pub PrivateUsage: SIZE_T,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PROCESSENTRY32W {
    pub dwSize: DWORD,
    pub cntUsage: DWORD,
    pub th32ProcessID: DWORD,
    pub th32DefaultHeapID: usize,
    pub th32ModuleID: DWORD,
    pub cntThreads: DWORD,
    pub th32ParentProcessID: DWORD,
    pub pcPriClassBase: i32,
    pub dwFlags: DWORD,
    pub szExeFile: [WCHAR; MAX_PATH],
}

impl Default for PROCESSENTRY32W {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SERVICE_STATUS_PROCESS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
    pub dwProcessId: DWORD,
    pub dwServiceFlags: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ENUM_SERVICE_STATUS_PROCESSW {
    pub lpServiceName: LPWSTR,
    pub lpDisplayName: LPWSTR,
    pub ServiceStatusProcess: SERVICE_STATUS_PROCESS,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QUERY_SERVICE_CONFIGW {
    pub dwServiceType: DWORD,
    pub dwStartType: DWORD,
    pub dwErrorControl: DWORD,
    pub lpBinaryPathName: LPWSTR,
    pub lpLoadOrderGroup: LPWSTR,
    pub dwTagId: DWORD,
    pub lpDependencies: LPWSTR,
    pub lpServiceStartName: LPWSTR,
    pub lpDisplayName: LPWSTR,
}

#[repr(C)]
pub struct SERVICE_DESCRIPTIONW {
    pub lpDescription: LPWSTR,
}

#[repr(C)]
pub struct SERVICE_DELAYED_AUTO_START_INFO {
    pub fDelayedAutostart: BOOL,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct TOKEN_ELEVATION {
    pub TokenIsElevated: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OSVERSIONINFOW {
    pub dwOSVersionInfoSize: DWORD,
    pub dwMajorVersion: DWORD,
    pub dwMinorVersion: DWORD,
    pub dwBuildNumber: DWORD,
    pub dwPlatformId: DWORD,
    pub szCSDVersion: [WCHAR; 128],
}

impl Default for OSVERSIONINFOW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

pub type WNDENUMPROC = extern "system" fn(HWND, isize) -> BOOL;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

/// FOLDERID_LocalAppData : identifiant stable du dossier de données local.
pub const FOLDERID_LOCALAPPDATA: GUID = GUID {
    data1: 0xF1B3_2785,
    data2: 0x6FBA,
    data3: 0x4FCF,
    data4: [0x9D, 0x55, 0x7B, 0x8E, 0x7F, 0x15, 0x70, 0x91],
};

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> BOOL;
    pub fn GetTickCount64() -> u64;
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn GetProcessTimes(
        hProcess: HANDLE,
        lpCreationTime: *mut FILETIME,
        lpExitTime: *mut FILETIME,
        lpKernelTime: *mut FILETIME,
        lpUserTime: *mut FILETIME,
    ) -> BOOL;
    pub fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn TerminateProcess(hProcess: HANDLE, uExitCode: u32) -> BOOL;
    pub fn QueryFullProcessImageNameW(
        hProcess: HANDLE,
        dwFlags: DWORD,
        lpExeName: LPWSTR,
        lpdwSize: *mut DWORD,
    ) -> BOOL;
    pub fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
    pub fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    pub fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    pub fn GetComputerNameW(lpBuffer: LPWSTR, nSize: *mut DWORD) -> BOOL;
    pub fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: DWORD) -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
}

#[link(name = "psapi")]
extern "system" {
    pub fn GetPerformanceInfo(
        pPerformanceInformation: *mut PERFORMANCE_INFORMATION,
        cb: DWORD,
    ) -> BOOL;
    pub fn K32GetProcessMemoryInfo(
        Process: HANDLE,
        ppsmemCounters: *mut PROCESS_MEMORY_COUNTERS_EX,
        cb: DWORD,
    ) -> BOOL;
}

#[link(name = "advapi32")]
extern "system" {
    pub fn OpenSCManagerW(
        lpMachineName: LPCWSTR,
        lpDatabaseName: LPCWSTR,
        dwDesiredAccess: DWORD,
    ) -> HANDLE;
    pub fn OpenServiceW(
        hSCManager: HANDLE,
        lpServiceName: LPCWSTR,
        dwDesiredAccess: DWORD,
    ) -> HANDLE;
    pub fn CloseServiceHandle(hSCObject: HANDLE) -> BOOL;
    pub fn EnumServicesStatusExW(
        hSCManager: HANDLE,
        InfoLevel: DWORD,
        dwServiceType: DWORD,
        dwServiceState: DWORD,
        lpServices: *mut u8,
        cbBufSize: DWORD,
        pcbBytesNeeded: *mut DWORD,
        lpServicesReturned: *mut DWORD,
        lpResumeHandle: *mut DWORD,
        pszGroupName: LPCWSTR,
    ) -> BOOL;
    pub fn QueryServiceConfigW(
        hService: HANDLE,
        lpServiceConfig: *mut QUERY_SERVICE_CONFIGW,
        cbBufSize: DWORD,
        pcbBytesNeeded: *mut DWORD,
    ) -> BOOL;
    pub fn QueryServiceConfig2W(
        hService: HANDLE,
        dwInfoLevel: DWORD,
        lpBuffer: *mut u8,
        cbBufSize: DWORD,
        pcbBytesNeeded: *mut DWORD,
    ) -> BOOL;
    pub fn QueryServiceStatusEx(
        hService: HANDLE,
        InfoLevel: DWORD,
        lpBuffer: *mut u8,
        cbBufSize: DWORD,
        pcbBytesNeeded: *mut DWORD,
    ) -> BOOL;
    pub fn ChangeServiceConfigW(
        hService: HANDLE,
        dwServiceType: DWORD,
        dwStartType: DWORD,
        dwErrorControl: DWORD,
        lpBinaryPathName: LPCWSTR,
        lpLoadOrderGroup: LPCWSTR,
        lpdwTagId: *mut DWORD,
        lpDependencies: LPCWSTR,
        lpServiceStartName: LPCWSTR,
        lpPassword: LPCWSTR,
        lpDisplayName: LPCWSTR,
    ) -> BOOL;
    pub fn ControlService(
        hService: HANDLE,
        dwControl: DWORD,
        lpServiceStatus: *mut SERVICE_STATUS_PROCESS,
    ) -> BOOL;
    pub fn StartServiceW(
        hService: HANDLE,
        dwNumServiceArgs: DWORD,
        lpServiceArgVectors: *const LPCWSTR,
    ) -> BOOL;
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        ulOptions: DWORD,
        samDesired: DWORD,
        phkResult: *mut HKEY,
    ) -> i32;
    pub fn RegCreateKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        Reserved: DWORD,
        lpClass: LPWSTR,
        dwOptions: DWORD,
        samDesired: DWORD,
        lpSecurityAttributes: *mut c_void,
        phkResult: *mut HKEY,
        lpdwDisposition: *mut DWORD,
    ) -> i32;
    pub fn RegCloseKey(hKey: HKEY) -> i32;
    pub fn RegEnumValueW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpValueName: LPWSTR,
        lpcchValueName: *mut DWORD,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: *mut u8,
        lpcbData: *mut DWORD,
    ) -> i32;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: *mut u8,
        lpcbData: *mut DWORD,
    ) -> i32;
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const u8,
        cbData: DWORD,
    ) -> i32;
    pub fn RegDeleteValueW(hKey: HKEY, lpValueName: LPCWSTR) -> i32;
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: LPWSTR,
        lpcchName: *mut DWORD,
        lpReserved: *mut DWORD,
        lpClass: LPWSTR,
        lpcchClass: *mut DWORD,
        lpftLastWriteTime: *mut c_void,
    ) -> i32;
    pub fn OpenProcessToken(
        ProcessHandle: HANDLE,
        DesiredAccess: DWORD,
        TokenHandle: *mut HANDLE,
    ) -> BOOL;
    pub fn GetTokenInformation(
        TokenHandle: HANDLE,
        TokenInformationClass: DWORD,
        TokenInformation: *mut c_void,
        TokenInformationLength: DWORD,
        ReturnLength: *mut DWORD,
    ) -> BOOL;
    pub fn GetUserNameW(lpBuffer: LPWSTR, pcbBuffer: *mut DWORD) -> BOOL;
}

#[link(name = "user32")]
extern "system" {
    pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: isize) -> BOOL;
    pub fn IsWindowVisible(hWnd: HWND) -> BOOL;
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
    pub fn GetWindowTextLengthW(hWnd: HWND) -> i32;
    pub fn MessageBoxW(hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: u32) -> i32;
}

#[link(name = "version")]
extern "system" {
    pub fn GetFileVersionInfoSizeW(lptstrFilename: LPCWSTR, lpdwHandle: *mut DWORD) -> DWORD;
    pub fn GetFileVersionInfoW(
        lptstrFilename: LPCWSTR,
        dwHandle: DWORD,
        dwLen: DWORD,
        lpData: *mut c_void,
    ) -> BOOL;
    pub fn VerQueryValueW(
        pBlock: *const c_void,
        lpSubBlock: LPCWSTR,
        lplpBuffer: *mut *mut c_void,
        puLen: *mut u32,
    ) -> BOOL;
}

#[link(name = "shell32")]
extern "system" {
    pub fn SHGetKnownFolderPath(
        rfid: *const GUID,
        dwFlags: DWORD,
        hToken: HANDLE,
        ppszPath: *mut LPWSTR,
    ) -> i32;
    pub fn ShellExecuteW(
        hwnd: HWND,
        lpOperation: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: i32,
    ) -> HANDLE;
}

#[link(name = "ntdll")]
extern "system" {
    pub fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> i32;
}

#[link(name = "ole32")]
extern "system" {
    pub fn CoTaskMemFree(pv: *mut c_void);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn from_wide(p: *const u16) -> String {
    if p.is_null() {
        return String::new();
    }
    unsafe {
        let mut len = 0usize;
        while *p.add(len) != 0 && len < 32768 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
    }
}

pub fn from_wide_buf(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

pub fn last_error() -> u32 {
    unsafe { GetLastError() }
}

/// Message d'erreur Windows lisible, sans dépendance.
pub fn err_msg(code: u32) -> String {
    match code {
        5 => "accès refusé (droits administrateur requis)".into(),
        2 => "élément introuvable".into(),
        3 => "chemin introuvable".into(),
        87 => "paramètre incorrect".into(),
        1053 => "le service n'a pas répondu à temps".into(),
        1058 => "service désactivé ou sans périphérique activé".into(),
        1060 => "le service n'existe pas".into(),
        1072 => "service marqué pour suppression (redémarrage requis)".into(),
        1077 => "aucun démarrage du service n'a été tenté".into(),
        0 => "opération refusée par le système".into(),
        c => format!("erreur Windows {c}"),
    }
}
