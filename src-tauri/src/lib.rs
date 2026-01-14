//! Nvidiot - NVIDIA ShadowPlay Application Manager
//!
//! A Tauri application for managing NVIDIA ShadowPlay application profiles.

mod nvapi;

#[cfg(target_os = "windows")]
mod windows;

use nvapi::{
    types::{DrsProfile, DrsApplication, RunningProcess, FocusApplication, BlacklistResult, NvApiStatus},
    profiles, applications, settings, session,
};

/// Get all DRS profiles
#[tauri::command]
async fn get_profiles() -> Result<Vec<DrsProfile>, String> {
    profiles::enumerate_profiles().map_err(|e| e.to_string())
}

/// Get all applications across all profiles
#[tauri::command]
async fn get_all_applications() -> Result<Vec<DrsApplication>, String> {
    applications::get_all_applications().map_err(|e| e.to_string())
}

/// Get all running processes with visible windows
#[tauri::command]
async fn get_running_processes() -> Result<Vec<RunningProcess>, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(windows::get_running_processes())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

/// Get the currently focused application
#[tauri::command]
async fn get_focus_application() -> Result<Option<FocusApplication>, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(windows::get_focus_application())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

/// Create a new profile for an executable
#[tauri::command]
async fn create_profile(executable: String, profile_name: String) -> Result<(), String> {
    applications::create_profile_for_executable(&executable, &profile_name)
        .map_err(|e| e.to_string())
}

/// Blacklist an application (disable ShadowPlay for it)
#[tauri::command]
async fn blacklist_application(executable: String) -> Result<BlacklistResult, String> {
    settings::blacklist_application(&executable).map_err(|e| e.to_string())
}

/// Unblacklist an application (enable ShadowPlay for it)
#[tauri::command]
async fn unblacklist_application(executable: String) -> Result<BlacklistResult, String> {
    settings::unblacklist_application(&executable).map_err(|e| e.to_string())
}

/// Check NVAPI availability
#[tauri::command]
async fn check_nvapi_status() -> NvApiStatus {
    match session::check_nvapi() {
        Ok(_) => NvApiStatus {
            available: true,
            error: None,
        },
        Err(e) => NvApiStatus {
            available: false,
            error: Some(e.to_string()),
        },
    }
}

/// Reload DRS settings from disk
#[tauri::command]
async fn reload_settings() -> Result<(), String> {
    session::reload_settings().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_profiles,
            get_all_applications,
            get_running_processes,
            get_focus_application,
            create_profile,
            blacklist_application,
            unblacklist_application,
            check_nvapi_status,
            reload_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
