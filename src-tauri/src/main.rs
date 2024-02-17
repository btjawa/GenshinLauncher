// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[link(name = "unlock_fps")]

extern {
    fn calc_fps_offset(
        hUnityPlayer: MODULEENTRY32, hUserAssembly: MODULEENTRY32, process: HANDLE
    ) -> uintptr_t;
}

use winapi::{um::{minwinbase::STILL_ACTIVE, winnt::{HANDLE, SECURITY_BUILTIN_DOMAIN_RID, SID_IDENTIFIER_AUTHORITY,
DOMAIN_ALIAS_RID_ADMINS, PSID}, memoryapi::{ReadProcessMemory, WriteProcessMemory},
processthreadsapi::{CreateProcessA, GetExitCodeProcess, PROCESS_INFORMATION, STARTUPINFOA}, handleapi::CloseHandle,
tlhelp32::{CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE, PROCESSENTRY32,
TH32CS_SNAPPROCESS, Process32First, Process32Next}, securitybaseapi::{CheckTokenMembership, AllocateAndInitializeSid, FreeSid},
errhandlingapi::GetLastError, winuser::{RegisterHotKey, GetMessageW, MSG, VK_F8, UnregisterHotKey, mouse_event,
MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, WM_HOTKEY}}, shared::minwindef::{LPVOID, DWORD}, vc::vadefs::uintptr_t};

use lazy_static::lazy_static;
use std::{ffi::{CStr, CString}, ptr::null_mut, path::PathBuf, os::raw::c_char, mem, panic, process::Command,
sync::{Arc, RwLock, Mutex, atomic::{AtomicBool, Ordering}}, thread::{self, sleep}, time::Duration};
use tauri::{Manager, Window as tWindow};

mod logger;

lazy_static! {
    static ref FPS_VALUE: Arc<RwLock<i64>> = Arc::new(RwLock::new(120));
    static ref GAME_PATH: Arc<RwLock<PathBuf>> = Arc::new(RwLock::new(PathBuf::from("D:\\Genshin Impact\\Genshin Impact Game\\YuanShen.exe")));
}

fn handle_err<E: std::fmt::Display>(window: tWindow, e: E) -> String {
    log::error!("{}", e);
    window.emit("error", e.to_string()).unwrap(); e.to_string()
}

#[tauri::command]
fn is_admin() -> bool { unsafe {
    let mut sid: PSID = null_mut();
    let mut nt_authority = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 5] };
    let result = AllocateAndInitializeSid(
        &mut nt_authority as *mut _ ,
        2,
        SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_ADMINS,
        0, 0, 0, 0, 0, 0,
        &mut sid,
    );
    if result == 0 { return false; }
    let mut is_member: winapi::shared::minwindef::BOOL = 0;
    let result = CheckTokenMembership(null_mut(), sid, &mut is_member);
    FreeSid(sid);
    result != 0 && is_member != 0
} }

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

unsafe fn get_module(pid: u32, module_name: &str) -> Option<MODULEENTRY32> {
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid);
    let mut entry = MODULEENTRY32 {
        dwSize: mem::size_of::<MODULEENTRY32>() as u32,
        ..mem::zeroed()
    };
    if Module32First(snap, &mut entry) != 0 {
        loop {
            let current_module_name = CStr::from_ptr(entry.szModule.as_ptr()).to_str().unwrap_or("");
            if entry.th32ProcessID == pid && current_module_name == module_name {
                CloseHandle(snap);
                return Some(entry);
            }
            if Module32Next(snap, &mut entry) == 0 {
                break;
            }
        }
    }
    CloseHandle(snap);
    None
}

unsafe fn spawn_game_process() -> Result<PROCESS_INFORMATION, String> {
    let game_path = GAME_PATH.read().unwrap();
    let command_line = "-popupwindow";
    if is_admin() {
        let process_path_c = CString::new(game_path.to_str().unwrap()).unwrap();
        let command_line_c = CString::new(command_line).unwrap().into_raw() as *mut c_char;
        let process_dir_c = CString::new(game_path.parent().unwrap().to_str().unwrap()).unwrap();
        let mut si: STARTUPINFOA = mem::zeroed();
        let mut pi: PROCESS_INFORMATION = mem::zeroed();
        let success = CreateProcessA(
            process_path_c.as_ptr(),
            command_line_c,
            null_mut(),
            null_mut(),
            winapi::shared::minwindef::FALSE,
            0,
            null_mut(),
            process_dir_c.as_ptr(),
            &mut si,
            &mut pi,
        );
        if success == 0 { return Err(format!("CreateProcess failed ({})", GetLastError())); }
        Ok(pi)    
    } else {
        Command::new("powershell.exe")
            .current_dir(game_path.parent().unwrap())
            .arg("Start-Process").arg("-FilePath")
            .arg(format!("{:?}", game_path)).arg("-Verb")
            .arg("RunAS").arg("-ArgumentList")
            .arg(command_line).spawn().unwrap();
        Ok(mem::zeroed())
    }
}

fn emit_rp(window: tWindow, msg: String) {
    window.emit("run-progress", msg.clone()).unwrap();
    log::info!("{}", msg);
}

#[tauri::command]
async fn unlock_fps(window: tWindow) -> Result<(), String> { unsafe {
    if is_process_alive(GAME_PATH.read().unwrap().file_name().unwrap().to_str().unwrap()) {
        emit_rp(window.clone(), "Already Running".to_string());
        return Err("Already Running".to_string());
    }
    let pi = spawn_game_process().unwrap();
    let pid = pi.dwProcessId;
    let process = pi.hProcess;
    if pid == 0 {
        emit_rp(window.clone(), "Done".to_string());
        return Err("Cannot get PID, probably not running as admin".to_string()) 
    }
    emit_rp(window.clone(), format!("PID: {}", pid));
    let h_unity_player = loop { match get_module(pid, "UnityPlayer.dll") {
        Some(module) => break module,
        None => sleep(Duration::from_millis(100)),
    } };
    let h_user_assembly = loop { match get_module(pid, "UserAssembly.dll") {
        Some(module) => break module,
        None => sleep(Duration::from_millis(100)),
    } };
    emit_rp(window.clone(), format!("UnityPlayer: {:X}", h_unity_player.modBaseAddr as uintptr_t));
    emit_rp(window.clone(), format!("UserAssembly: {:X}", h_user_assembly.modBaseAddr as uintptr_t));
    emit_rp(window.clone(), "Searching for pattern...".to_string());
    
    // 计算相对地址 (FPS)
    let mut pfps: uintptr_t = calc_fps_offset(h_unity_player, h_user_assembly, process);
    if pfps == 1 { return Err("VirtualAlloc failed".to_string()) }
    emit_rp(window.clone(), format!("FPS Offset: {:X}", pfps));
    pfps = h_unity_player.modBaseAddr as uintptr_t + pfps;

    emit_rp(window.clone(), format!("{}: Done", pid));
    let mut dw_exit_code: DWORD = STILL_ACTIVE;
    while dw_exit_code == STILL_ACTIVE {
        GetExitCodeProcess(process, &mut dw_exit_code as *mut DWORD);
        sleep(Duration::from_secs(1));
        let mut fps = 0;
        let target_fps = *FPS_VALUE.read().unwrap();
        ReadProcessMemory(process, pfps as LPVOID, &mut fps as *mut _ as LPVOID, mem::size_of::<i32>(), null_mut());
        if fps == -1 { continue; }
        if fps != target_fps {
            WriteProcessMemory(process, pfps as LPVOID, &target_fps as *const _ as LPVOID, mem::size_of_val(&target_fps), null_mut());
        }
    }
    CloseHandle(process);
    emit_rp(window, format!("{}: Closed", pid));
    Ok(())
} }

unsafe fn clicker() {
    RegisterHotKey(null_mut(), 1, 0, VK_F8 as u32);
    log::info!("Registered F8 for MouseClicker");
    let mut msg: MSG = mem::zeroed();
    static CLICKER_ENABLED: AtomicBool = AtomicBool::new(false);
    while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
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
    UnregisterHotKey(null_mut(), 1);
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
            let window = app.get_window("main").unwrap();
            let window_for_panic = Arc::new(Mutex::new(window.clone()));
            panic::set_hook(Box::new(move |e| {
                handle_err(window_for_panic.lock().unwrap().clone(), e);
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![is_admin, unlock_fps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
