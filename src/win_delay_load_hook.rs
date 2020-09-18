//! Rust port of [win_delay_load_hook.cc][].
//!
//! When the addon tries to load the "node.exe" DLL module, this module gives it the pointer to the
//! .exe we are running in instead. Typically, that will be the same value. But if the node executable
//! was renamed, you would not otherwise get the correct DLL.
//!
//! [win_delay_load_hook.cc]: https://github.com/nodejs/node-gyp/blob/e18a61afc1669d4897e6c5c8a6694f4995a0f4d6/src/win_delay_load_hook.cc

use winapi::shared::minwindef::{BOOL, DWORD, FARPROC, HMODULE, LPVOID};
use winapi::shared::ntdef::LPCSTR;
use winapi::um::libloaderapi::GetModuleHandleA;
use std::ffi::CStr;
use std::ptr::null_mut;

// Structures hand-copied from
// https://docs.microsoft.com/en-us/cpp/build/reference/structure-and-constant-definitions

#[repr(C)]
#[allow(non_snake_case)]
struct DelayLoadProc {
    fImportByName: BOOL,
    // Technically this is `union{LPCSTR; DWORD;}` but we don't access it anyways.
    szProcName: LPCSTR,
}

#[repr(C)]
#[allow(non_snake_case)]
struct DelayLoadInfo {
    /// size of structure
    cb: DWORD,
    /// raw form of data (everything is there)
    /// Officially a pointer to ImgDelayDescr but we don't access it.
    pidd: LPVOID,
    /// points to address of function to load
    ppfn: *mut FARPROC,
    /// name of dll
    szDll: LPCSTR,
    /// name or ordinal of procedure
    dlp: DelayLoadProc,
    /// the hInstance of the library we have loaded
    hmodCur: HMODULE,
    /// the actual function that will be called
    pfnCur: FARPROC,
    /// error received (if an error notification)
    dwLastError: DWORD,
}

#[allow(non_snake_case)]
type PfnDliHook = unsafe extern "C" fn(dliNotify: usize, pdli: *const DelayLoadInfo) -> FARPROC;

const HOST_BINARIES: &[&[u8]] = &[b"node.exe", b"electron.exe"];

unsafe extern "C" fn load_exe_hook(event: usize, info: *const DelayLoadInfo) -> FARPROC {

    if event != 0x01 /* dliNotePreLoadLibrary */ {
        return null_mut();
    }

    let dll_name = CStr::from_ptr((*info).szDll);
    if !HOST_BINARIES.iter().any(|&host_name| host_name == dll_name.to_bytes()) {
        return null_mut();
    }

    let exe_handle = GetModuleHandleA(null_mut());

    // PfnDliHook sometimes has to return a FARPROC, sometimes an HMODULE, but only one
    // of them could be specified in the header, so we have to cast our HMODULE to that.
    exe_handle as FARPROC
}

#[no_mangle]
static mut __pfnDliNotifyHook2: *mut PfnDliHook = load_exe_hook as *mut PfnDliHook;
