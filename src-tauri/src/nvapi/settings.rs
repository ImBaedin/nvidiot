//! DRS Settings management
//!
//! Handles getting and setting DRS settings, particularly the ShadowPlay blacklist.

use super::error::{NvApiError, NVAPI_OK, NVAPI_SETTING_NOT_FOUND};
use super::ffi::{
    get_nvapi, NvDRSProfileHandle, NvdrsSetting, NVDRS_SETTING_VER,
    SHADOWPLAY_SETTING_ID, SHADOWPLAY_DISABLED, SHADOWPLAY_ENABLED,
};
use super::session::{get_session, save_settings};
use super::applications::find_application;
use super::profiles::{find_profile_by_name, create_profile};
use super::types::BlacklistResult;

/// Get a DWORD setting value from a profile
#[cfg(target_os = "windows")]
pub fn get_dword_setting(profile_handle: NvDRSProfileHandle, setting_id: u32) -> Result<u32, NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let get_setting = api.drs_get_setting
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_GetSetting".to_string()))?;

    unsafe {
        let mut setting = NvdrsSetting::default();
        setting.version = NVDRS_SETTING_VER;

        let status = get_setting(session, profile_handle, setting_id, &mut setting);

        if status == NVAPI_SETTING_NOT_FOUND {
            return Err(NvApiError::GetSettingFailed(status));
        }
        if status != NVAPI_OK {
            return Err(NvApiError::GetSettingFailed(status));
        }

        Ok(setting.current_value.dword_value)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_dword_setting(_profile_handle: NvDRSProfileHandle, _setting_id: u32) -> Result<u32, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Set a DWORD setting value in a profile
#[cfg(target_os = "windows")]
pub fn set_dword_setting(profile_handle: NvDRSProfileHandle, setting_id: u32, value: u32) -> Result<(), NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let set_setting = api.drs_set_setting
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_SetSetting".to_string()))?;

    unsafe {
        let mut setting = NvdrsSetting::default();
        setting.version = NVDRS_SETTING_VER;
        setting.setting_id = setting_id;
        setting.setting_type = 0; // DWORD
        setting.current_value.dword_value = value;

        let status = set_setting(session, profile_handle, &mut setting);

        if status != NVAPI_OK {
            return Err(NvApiError::SetSettingFailed(status));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_dword_setting(_profile_handle: NvDRSProfileHandle, _setting_id: u32, _value: u32) -> Result<(), NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Check if ShadowPlay is disabled for a profile
#[cfg(target_os = "windows")]
pub fn get_shadowplay_status(profile_handle: NvDRSProfileHandle) -> Result<bool, NvApiError> {
    match get_dword_setting(profile_handle, SHADOWPLAY_SETTING_ID) {
        Ok(value) => Ok(value == SHADOWPLAY_DISABLED),
        Err(NvApiError::GetSettingFailed(_)) => {
            // Setting not found means default (enabled)
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_shadowplay_status(_profile_handle: NvDRSProfileHandle) -> Result<bool, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Blacklist an application (disable ShadowPlay for it)
#[cfg(target_os = "windows")]
pub fn blacklist_application(executable: &str) -> Result<BlacklistResult, NvApiError> {
    // Try to find existing application
    match find_application(executable) {
        Ok((profile_handle, _app)) => {
            // Application exists, set the ShadowPlay setting
            set_dword_setting(profile_handle, SHADOWPLAY_SETTING_ID, SHADOWPLAY_DISABLED)?;
            save_settings()?;

            Ok(BlacklistResult {
                success: true,
                executable: executable.to_string(),
                message: "Application blacklisted successfully".to_string(),
            })
        }
        Err(NvApiError::ApplicationNotFound(_)) => {
            // Application not in DRS, need to create a profile for it
            let profile_name = format!("Nvidiot - {}", executable);

            // Try to find or create the profile
            let profile_handle = match find_profile_by_name(&profile_name) {
                Ok(handle) => handle,
                Err(NvApiError::ProfileNotFound(_)) => {
                    create_profile(&profile_name)?
                }
                Err(e) => return Err(e),
            };

            // Add application to profile
            super::applications::create_application(profile_handle, executable, &profile_name)?;

            // Set the blacklist setting
            set_dword_setting(profile_handle, SHADOWPLAY_SETTING_ID, SHADOWPLAY_DISABLED)?;
            save_settings()?;

            Ok(BlacklistResult {
                success: true,
                executable: executable.to_string(),
                message: format!("Created profile '{}' and blacklisted application", profile_name),
            })
        }
        Err(e) => Err(e),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn blacklist_application(_executable: &str) -> Result<BlacklistResult, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Unblacklist an application (enable ShadowPlay for it)
#[cfg(target_os = "windows")]
pub fn unblacklist_application(executable: &str) -> Result<BlacklistResult, NvApiError> {
    match find_application(executable) {
        Ok((profile_handle, _app)) => {
            // Set the ShadowPlay setting to enabled
            set_dword_setting(profile_handle, SHADOWPLAY_SETTING_ID, SHADOWPLAY_ENABLED)?;
            save_settings()?;

            Ok(BlacklistResult {
                success: true,
                executable: executable.to_string(),
                message: "Application unblacklisted successfully".to_string(),
            })
        }
        Err(NvApiError::ApplicationNotFound(_)) => {
            // Application not in DRS, nothing to unblacklist
            Ok(BlacklistResult {
                success: false,
                executable: executable.to_string(),
                message: "Application not found in driver settings".to_string(),
            })
        }
        Err(e) => Err(e),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn unblacklist_application(_executable: &str) -> Result<BlacklistResult, NvApiError> {
    Err(NvApiError::NotSupported)
}
