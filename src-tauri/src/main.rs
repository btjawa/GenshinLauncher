// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio::sync::OnceCell;
use winapi::{
    shared::{basetsd::SIZE_T, minwindef::{DWORD, HMODULE, LPCVOID, LPVOID}},
    um::{
        errhandlingapi::GetLastError,
        handleapi::CloseHandle,
        memoryapi::{ReadProcessMemory, VirtualAlloc, VirtualFree, WriteProcessMemory},
        minwinbase::STILL_ACTIVE,
        processthreadsapi::{CreateProcessA, GetExitCodeProcess, PROCESS_INFORMATION, STARTUPINFOA},
        psapi::{EnumProcessModules, GetModuleBaseNameA, GetModuleInformation, MODULEINFO},
        tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPPROCESS},
        winbase::{FormatMessageA, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS},
        winnt::{HANDLE, IMAGE_DOS_HEADER, IMAGE_NT_HEADERS, LANG_NEUTRAL, MAKELANGID, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, SUBLANG_DEFAULT},
        winuser::{mouse_event, GetMessageW, RegisterHotKey, UnregisterHotKey, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MSG, VK_F8, WM_HOTKEY},
    },
};

use lazy_static::lazy_static;
use std::{ffi::{CStr, CString}, ptr, path::PathBuf, os::raw::c_char,
mem, panic, sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}}, thread::{self, sleep}, time::Duration};
use tauri::{Manager, Window as tWindow, Wry};

mod logger;

lazy_static! {
    static ref FPS_VALUE: Arc<RwLock<i64>> = Arc::new(RwLock::new(120));
    static ref GAME_PATH: Arc<RwLock<PathBuf>> = Arc::new(RwLock::new(PathBuf::from("D:\\Genshin Impact\\Genshin Impact Game\\YuanShen.exe")));
    static ref APP_WINDOW: Arc<OnceCell<tWindow<Wry>>> = Arc::new(OnceCell::new());
}

fn handle_err<E: std::fmt::Display>(e: E) -> String {
    let err_msg = e.to_string();
    log::error!("{}", err_msg);
    APP_WINDOW.get().unwrap().emit("error", &err_msg).unwrap();
    e.to_string()
}

unsafe fn get_last_error() -> String {
    let code = GetLastError();
    let mut buf: LPVOID = ptr::null_mut();
    let size = FormatMessageA(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        ptr::null(),
        code,
        MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD,
        &mut buf as *mut LPVOID as *mut i8,
        0,
        ptr::null_mut(),
    );
    let message = if size == 0 {
        "Unknown error".into()
    } else {
        let message = CStr::from_ptr(buf as *const i8).to_string_lossy().into_owned();
        LocalFree(buf);
        message
    };
    format!("{}, {}", message, code)
}

unsafe fn is_process_alive(process_name: &str) -> bool {
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    let mut entry = PROCESSENTRY32 {
        dwSize: mem::size_of::<PROCESSENTRY32>() as u32,
        ..mem::zeroed()
    };
    if Process32First(snap, &mut entry) != 0 {
        loop {
            let current_process_name = CStr::from_ptr(entry.szExeFile.as_ptr()).to_str().unwrap_or("");
            if current_process_name == process_name {
                CloseHandle(snap);
                return true;
            }
            if Process32Next(snap, &mut entry) == 0 {
                break;
            }
        }
    }
    CloseHandle(snap);     
    false
}

unsafe fn pattern_scan(module: *mut u8, signature: &str) -> usize {
    fn pattern_to_byte(pattern: &str) -> Vec<i32> {
        let mut bytes = Vec::new();
        let mut chars = pattern.chars().peekable();
        while let Some(&ch) = chars.peek() {
            if ch == '?' {
                chars.next();
                if chars.peek() == Some(&'?') {
                    chars.next();
                }
                bytes.push(-1);
            } else if !ch.is_whitespace() {
                let mut hex_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || ch == '?' {
                        break;
                    }
                    hex_str.push(ch);
                    chars.next();
                }
                if let Ok(byte) = i32::from_str_radix(&hex_str, 16) {
                    bytes.push(byte);
                }
            } else { chars.next(); }
        } return bytes;
    }
    let dos_header = module as *const IMAGE_DOS_HEADER;
    let nt_headers = (module.add((*dos_header).e_lfanew as usize)) as *const IMAGE_NT_HEADERS;
    let pattern_bytes = pattern_to_byte(signature);
    let scan_bytes = std::slice::from_raw_parts(module, (*nt_headers).OptionalHeader.SizeOfImage as usize);
    let s = pattern_bytes.len();
    let d = pattern_bytes.as_slice();
    for i in 0..(scan_bytes.len() - s) {
        let mut found = true;
        for j in 0..s {
            if scan_bytes[i + j] != d[j] as u8 && d[j] != -1 {
                found = false;
                break;
            }
        }
        if found { return module.add(i) as usize }
    } return 0;
}

unsafe fn get_module(handle: HANDLE, module_name: &str) -> Option<MODULEENTRY32> {
    let mut modules: Vec<HMODULE> = vec![ptr::null_mut(); 1024];
    let mut cb_needed = 0;
    if EnumProcessModules(handle, modules.as_mut_ptr(), (modules.len() * mem::size_of::<HMODULE>()) as u32, &mut cb_needed) == 0 {
        return None;
    }
    modules.resize((cb_needed / mem::size_of::<HMODULE>() as u32) as usize, ptr::null_mut());
    for &module in modules.iter() {
        if module.is_null() {
            continue;
        }
        let mut sz_module_name = vec![0u8; 256];
        if GetModuleBaseNameA(handle, module, sz_module_name.as_mut_ptr() as *mut i8, sz_module_name.len() as u32) == 0 {
            continue;
        }
        let module_name_cstr = CString::new(module_name).expect("CString::new failed");
        if sz_module_name.starts_with(module_name_cstr.to_bytes()) {
            let mut mod_info: MODULEINFO = std::mem::zeroed();
            if GetModuleInformation(handle, module, &mut mod_info, mem::size_of::<MODULEINFO>() as u32) == 0 {
                continue;
            }
            let entry = MODULEENTRY32 {
                dwSize: mem::size_of::<MODULEENTRY32>() as u32,
                th32ModuleID: 0,
                th32ProcessID: 0,
                GlblcntUsage: 0,
                ProccntUsage: 0,
                modBaseAddr: mod_info.lpBaseOfDll as *mut u8,
                modBaseSize: mod_info.SizeOfImage,
                hModule: module,
                szModule: [0; 256],
                szExePath: [0; 260],
            };
            return Some(entry);
        }
    }
    None
}

unsafe fn spawn_game_process() -> Result<PROCESS_INFORMATION, String> {
    let game_path = GAME_PATH.read().unwrap();
    let command_line = "";
    let process_path_c = CString::new(game_path.to_str().unwrap()).unwrap();
    let command_line_c = CString::new(command_line).unwrap().into_raw() as *mut c_char;
    let process_dir_c = CString::new(game_path.parent().unwrap().to_str().unwrap()).unwrap();
    let mut si: STARTUPINFOA = mem::zeroed();
    let mut pi: PROCESS_INFORMATION = mem::zeroed();
    let success = CreateProcessA(
        process_path_c.as_ptr(),
        command_line_c,
        ptr::null_mut(), ptr::null_mut(),
        winapi::shared::minwindef::FALSE,
        0, ptr::null_mut(),
        process_dir_c.as_ptr(),
        &mut si, &mut pi,
    );
    if success == 0 { return Err(format!("CreateProcess failed ({})", GetLastError())); }
    Ok(pi)    
}

fn emit_rp(msg: String) {
    APP_WINDOW.get().unwrap().emit("run-progress", msg.clone()).unwrap();
    log::info!("{}", msg);
}

unsafe fn read_unaligned_i32(ptr: *const u8) -> i32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(std::slice::from_raw_parts(ptr, 4));
    i32::from_le_bytes(bytes)
}

#[tauri::command]
async fn unlock_fps() -> Result<(), String> { unsafe {
    if is_process_alive(GAME_PATH.read().unwrap().file_name().unwrap().to_str().unwrap()) {
        emit_rp("Already Running".into());
        return Err("Already Running".into());
    }
    let pi = spawn_game_process().unwrap();
    let pid = pi.dwProcessId;
    let process = pi.hProcess;
    emit_rp(format!("PID: {}", pid));
    let h_unity_player = loop { match get_module(process, "UnityPlayer.dll") {
        Some(module) => break module,
        None => sleep(Duration::from_millis(100)),
    } };
    emit_rp(format!("UnityPlayer: {:X}", h_unity_player.modBaseAddr as usize));
    
    // 计算相对地址 (FPS)
    let up: LPVOID = VirtualAlloc(ptr::null_mut(), h_unity_player.modBaseSize as SIZE_T, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if up.is_null() {
        emit_rp(format!("VirtualAlloc UP failed ({})", get_last_error()));
    }
    if ReadProcessMemory(process, h_unity_player.modBaseAddr as LPCVOID, up, h_unity_player.modBaseSize as SIZE_T, ptr::null_mut()) == 0 {
        emit_rp(format!("ReadProcessMemory unity failed ({})", get_last_error()));
    }
    emit_rp("Searching for pattern...".into());
    let address: usize = pattern_scan(up as *mut u8, "7F 0E E8 ?? ?? ?? ?? 66 0F 6E C8");
    if address == 0 {
        emit_rp("Outdate Pattern".into());
    }
    let mut rip: usize = address;
    rip += 3;
    rip += read_unaligned_i32(rip as *const u8) as usize + 6;
    rip += read_unaligned_i32(rip as *const u8) as usize + 4;
    let pfps: usize = rip - up as usize + h_unity_player.modBaseAddr as usize;
    VirtualFree(up, 0, MEM_RELEASE);
    emit_rp(format!("FPS Offset: {:X}", pfps));

    emit_rp(format!("{}: Done", pid));
    let mut dw_exit_code: DWORD = STILL_ACTIVE;
    while dw_exit_code == STILL_ACTIVE {
        GetExitCodeProcess(process, &mut dw_exit_code as *mut DWORD);
        sleep(Duration::from_secs(1));
        let mut fps = 0;
        let target_fps = *FPS_VALUE.read().unwrap();
        ReadProcessMemory(process, pfps as LPVOID, &mut fps as *mut _ as LPVOID, mem::size_of::<i32>(), ptr::null_mut());
        if fps == -1 { continue; }
        if fps != target_fps {
            WriteProcessMemory(process, pfps as LPVOID, &target_fps as *const _ as LPVOID, mem::size_of_val(&target_fps), ptr::null_mut());
        }
    }
    CloseHandle(process);
    emit_rp(format!("{}: Closed", pid));
    Ok(())
} }

unsafe fn clicker() {
    RegisterHotKey(ptr::null_mut(), 1, 0, VK_F8 as u32);
    log::info!("Registered F8 for MouseClicker");
    let mut msg: MSG = mem::zeroed();
    static CLICKER_ENABLED: AtomicBool = AtomicBool::new(false);
    while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
        if msg.message == WM_HOTKEY {
            let e = CLICKER_ENABLED.load(Ordering::SeqCst);
            CLICKER_ENABLED.store(!e, Ordering::SeqCst);
            if CLICKER_ENABLED.load(Ordering::SeqCst) {
                thread::spawn(|| {
                    while CLICKER_ENABLED.load(Ordering::SeqCst) {
                        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                        sleep(Duration::from_millis(100));
                    }
                });
            }
        }
    }
    UnregisterHotKey(ptr::null_mut(), 1);
}

fn main() {
    logger::init_logger().unwrap();
    thread::spawn(|| { unsafe { clicker() } });
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            app.emit_all("single-instance", (argv, cwd)).unwrap();
        }))
        .setup(|app| {
            APP_WINDOW.set(app.get_window("main").unwrap()).unwrap();
            panic::set_hook(Box::new(move |e| { handle_err(e); }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![unlock_fps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
