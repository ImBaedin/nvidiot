//! Running process enumeration
//!
//! Lists all running processes with visible windows.

use crate::nvapi::types::RunningProcess;
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use windows::{
    core::{BOOL, PWSTR},
    Win32::Foundation::{HWND, LPARAM},
    Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    },
    Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT},
    Win32::System::ProcessStatus::GetModuleBaseNameW,
};

#[cfg(target_os = "windows")]
use crate::nvapi::applications::find_application;
#[cfg(target_os = "windows")]
use crate::nvapi::settings::get_shadowplay_status;

/// Data collected during window enumeration
#[cfg(target_os = "windows")]
struct ProcessInfo {
    process_id: u32,
    process_name: String,
    window_title: String,
    executable_path: Option<String>,
}

/// Callback data for EnumWindows
#[cfg(target_os = "windows")]
struct EnumData {
    processes: HashMap<u32, ProcessInfo>,
}

/// Window enumeration callback
#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumData);

    // Skip invisible windows
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1); // Continue enumeration
    }

    // Get window title
    let mut title_buffer = [0u16; 512];
    let title_len = GetWindowTextW(hwnd, &mut title_buffer);
    if title_len == 0 {
        return BOOL(1); // Skip windows without titles
    }
    let window_title = String::from_utf16_lossy(&title_buffer[..title_len as usize]);

    // Skip empty titles
    if window_title.trim().is_empty() {
        return BOOL(1);
    }

    // Get process ID
    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    if process_id == 0 {
        return BOOL(1);
    }

    // Skip if we already have this process
    if data.processes.contains_key(&process_id) {
        return BOOL(1);
    }

    // Get process info
    if let Some((process_name, executable_path)) = get_process_info(process_id) {
        // Skip system processes
        if is_system_process(&process_name) {
            return BOOL(1);
        }

        data.processes.insert(process_id, ProcessInfo {
            process_id,
            process_name,
            window_title,
            executable_path,
        });
    }

    BOOL(1) // Continue enumeration
}

#[cfg(target_os = "windows")]
fn get_process_info(pid: u32) -> Option<(String, Option<String>)> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        // Get full path first (more reliable)
        let mut path_buffer = [0u16; 1024];
        let mut path_len = path_buffer.len() as u32;
        let (process_name, executable_path) = if QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), PWSTR(path_buffer.as_mut_ptr()), &mut path_len).is_ok() {
            let full_path = String::from_utf16_lossy(&path_buffer[..path_len as usize]);
            // Extract process name from path
            let name = full_path.rsplit('\\').next()
                .unwrap_or(&full_path)
                .to_string();
            (name, Some(full_path))
        } else {
            // Fallback to GetModuleBaseNameW
            let mut name_buffer = [0u16; 260];
            let name_len = GetModuleBaseNameW(handle, None, &mut name_buffer);
            if name_len > 0 {
                (String::from_utf16_lossy(&name_buffer[..name_len as usize]), None)
            } else {
                return None;
            }
        };

        Some((process_name, executable_path))
    }
}

#[cfg(target_os = "windows")]
fn is_system_process(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    matches!(name_lower.as_str(),
        "explorer.exe" |
        "searchhost.exe" |
        "shellexperiencehost.exe" |
        "startmenuexperiencehost.exe" |
        "textinputhost.exe" |
        "applicationframehost.exe" |
        "systemsettings.exe" |
        "runtimebroker.exe" |
        "dwm.exe" |
        "csrss.exe" |
        "winlogon.exe" |
        "services.exe" |
        "lsass.exe" |
        "svchost.exe"
    )
}

/// Get all running processes with visible windows
#[cfg(target_os = "windows")]
pub fn get_running_processes() -> Vec<RunningProcess> {
    let mut data = EnumData {
        processes: HashMap::new(),
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut data as *mut _ as isize),
        );
    }

    // Convert to RunningProcess and check DRS status
    data.processes
        .into_values()
        .map(|info| {
            let (has_drs_profile, profile_name, is_blacklisted) = match find_application(&info.process_name) {
                Ok((profile_handle, _app)) => {
                    let profile_name = get_profile_name_from_handle(profile_handle);
                    let is_blacklisted = get_shadowplay_status(profile_handle).ok();
                    (true, profile_name, is_blacklisted)
                }
                Err(_) => (false, None, None),
            };

            RunningProcess {
                process_name: info.process_name,
                window_title: info.window_title,
                process_id: info.process_id,
                executable_path: info.executable_path,
                has_drs_profile,
                profile_name,
                is_blacklisted,
            }
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
pub fn get_running_processes() -> Vec<RunningProcess> {
    Vec::new()
}

#[cfg(target_os = "windows")]
fn get_profile_name_from_handle(profile_handle: crate::nvapi::ffi::NvDRSProfileHandle) -> Option<String> {
    use crate::nvapi::ffi::{get_nvapi, wchar_to_string, NvdrsProfile};
    use crate::nvapi::session::get_session;
    use crate::nvapi::error::NVAPI_OK;

    let api = get_nvapi().ok()?;
    let session = get_session().ok()?;
    let get_profile_info = api.drs_get_profile_info?;

    unsafe {
        let mut profile_info = NvdrsProfile::default();
        let status = get_profile_info(session, profile_handle, &mut profile_info);
        if status == NVAPI_OK {
            Some(wchar_to_string(&profile_info.profile_name))
        } else {
            None
        }
    }
}
