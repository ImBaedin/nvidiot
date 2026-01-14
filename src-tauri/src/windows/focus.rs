//! Foreground window detection
//!
//! Uses Windows API to detect the currently focused application.

use crate::nvapi::types::FocusApplication;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::HWND,
    Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
    Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    Win32::System::ProcessStatus::GetModuleBaseNameW,
};

#[cfg(target_os = "windows")]
use crate::nvapi::applications::find_application;
#[cfg(target_os = "windows")]
use crate::nvapi::settings::get_shadowplay_status;
#[cfg(target_os = "windows")]
use crate::nvapi::ffi::wchar_to_string;

/// Get the currently focused application
#[cfg(target_os = "windows")]
pub fn get_focus_application() -> Option<FocusApplication> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        // Get process ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == 0 {
            return None;
        }

        // Get window title
        let mut title_buffer = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buffer);
        let window_title = String::from_utf16_lossy(&title_buffer[..title_len as usize]);

        // Get process name
        let process_name = get_process_name(process_id).unwrap_or_default();

        // Check if this application is in DRS
        let (is_in_drs, profile_name, is_blacklisted) = match find_application(&process_name) {
            Ok((profile_handle, _app)) => {
                let profile_name = get_profile_name_for_app(&process_name);
                let is_blacklisted = get_shadowplay_status(profile_handle).ok();
                (true, profile_name, is_blacklisted)
            }
            Err(_) => (false, None, None),
        };

        Some(FocusApplication {
            process_name,
            window_title,
            process_id,
            is_in_drs,
            profile_name,
            is_blacklisted,
        })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_focus_application() -> Option<FocusApplication> {
    None
}

#[cfg(target_os = "windows")]
fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut name_buffer = [0u16; 260];
        let len = GetModuleBaseNameW(&handle, None, &mut name_buffer);
        if len > 0 {
            Some(String::from_utf16_lossy(&name_buffer[..len as usize]))
        } else {
            None
        }
    }
}

#[cfg(target_os = "windows")]
fn get_profile_name_for_app(executable: &str) -> Option<String> {
    use crate::nvapi::profiles::enumerate_profiles;
    use crate::nvapi::profiles::find_profile_by_name;
    use crate::nvapi::applications::enumerate_applications;

    // Search through all profiles to find which one contains this app
    if let Ok(profiles) = enumerate_profiles() {
        for profile in profiles {
            if let Ok(profile_handle) = find_profile_by_name(&profile.name) {
                if let Ok(apps) = enumerate_applications(profile_handle, &profile.name) {
                    for app in apps {
                        if app.executable.to_lowercase() == executable.to_lowercase() {
                            return Some(profile.name);
                        }
                    }
                }
            }
        }
    }
    None
}
