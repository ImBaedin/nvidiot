//! DRS Application management
//!
//! Handles enumerating and creating applications within profiles.

use super::error::{NvApiError, NVAPI_OK, NVAPI_END_ENUMERATION, NVAPI_EXECUTABLE_NOT_FOUND};
use super::ffi::{
    get_nvapi, wchar_to_string, string_to_wchar,
    NvDRSProfileHandle, NvdrsApplication, NvdrsProfile, NVDRS_APPLICATION_VER,
};
use super::session::get_session;
use super::profiles::{enumerate_profiles, find_profile_by_name};
use super::settings::get_shadowplay_status;
use super::types::DrsApplication;

/// Enumerate applications in a specific profile
#[cfg(target_os = "windows")]
pub fn enumerate_applications(profile_handle: NvDRSProfileHandle, profile_name: &str) -> Result<Vec<DrsApplication>, NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let enum_apps = api.drs_enum_applications
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_EnumApplications".to_string()))?;
    let get_profile_info = api.drs_get_profile_info
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_GetProfileInfo".to_string()))?;

    // Get profile info to know how many apps
    let mut profile_info = NvdrsProfile::default();
    let num_apps = unsafe {
        let status = get_profile_info(session, profile_handle, &mut profile_info);
        if status != NVAPI_OK {
            // If we can't get profile info, try enumerating anyway
            u32::MAX
        } else {
            profile_info.num_of_apps
        }
    };

    let mut applications = Vec::new();
    let mut start_index: u32 = 0;

    unsafe {
        while start_index < num_apps {
            // Enumerate in batches
            let mut apps: [NvdrsApplication; 32] = std::array::from_fn(|_| NvdrsApplication::default());
            let mut count: u32 = 32;

            let status = enum_apps(session, profile_handle, start_index, &mut count, apps.as_mut_ptr());

            if status == NVAPI_END_ENUMERATION || count == 0 {
                break;
            }
            if status != NVAPI_OK {
                // Continue even on error - some profiles may have issues
                break;
            }

            for i in 0..(count as usize) {
                let app = &apps[i];
                let executable = wchar_to_string(&app.app_name);

                // Check blacklist status for this app
                let is_blacklisted = get_shadowplay_status(profile_handle).unwrap_or(false);

                applications.push(DrsApplication {
                    name: wchar_to_string(&app.user_friendly_name),
                    executable: executable.clone(),
                    profile_name: profile_name.to_string(),
                    is_predefined: app.is_predefined != 0,
                    is_blacklisted,
                });
            }

            start_index += count;
        }
    }

    Ok(applications)
}

#[cfg(not(target_os = "windows"))]
pub fn enumerate_applications(_profile_handle: NvDRSProfileHandle, _profile_name: &str) -> Result<Vec<DrsApplication>, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Get all applications across all profiles
#[cfg(target_os = "windows")]
pub fn get_all_applications() -> Result<Vec<DrsApplication>, NvApiError> {
    let profiles = enumerate_profiles()?;
    let mut all_apps = Vec::new();

    for profile in profiles {
        // Only query profiles that have applications
        if profile.application_count == 0 {
            continue;
        }

        if let Ok(profile_handle) = find_profile_by_name(&profile.name) {
            if let Ok(apps) = enumerate_applications(profile_handle, &profile.name) {
                all_apps.extend(apps);
            }
        }
    }

    Ok(all_apps)
}

#[cfg(not(target_os = "windows"))]
pub fn get_all_applications() -> Result<Vec<DrsApplication>, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Find an application by executable name
#[cfg(target_os = "windows")]
pub fn find_application(executable: &str) -> Result<(NvDRSProfileHandle, NvdrsApplication), NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let find_fn = api.drs_find_application_by_name
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_FindApplicationByName".to_string()))?;

    let mut wide_name = [0u16; 2048];
    string_to_wchar(executable, &mut wide_name);

    unsafe {
        let mut profile_handle: NvDRSProfileHandle = std::ptr::null_mut();
        let mut app = NvdrsApplication::default();

        let status = find_fn(session, wide_name.as_ptr(), &mut profile_handle, &mut app);

        if status == NVAPI_EXECUTABLE_NOT_FOUND {
            return Err(NvApiError::ApplicationNotFound(executable.to_string()));
        }
        if status != NVAPI_OK {
            return Err(NvApiError::NvApiStatus(status));
        }

        Ok((profile_handle, app))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn find_application(_executable: &str) -> Result<(NvDRSProfileHandle, NvdrsApplication), NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Create a new application in a profile
#[cfg(target_os = "windows")]
pub fn create_application(profile_handle: NvDRSProfileHandle, executable: &str, friendly_name: &str) -> Result<(), NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let create_fn = api.drs_create_application
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_CreateApplication".to_string()))?;

    let mut app = NvdrsApplication {
        version: NVDRS_APPLICATION_VER,
        ..Default::default()
    };
    string_to_wchar(executable, &mut app.app_name);
    string_to_wchar(friendly_name, &mut app.user_friendly_name);

    unsafe {
        let status = create_fn(session, profile_handle, &mut app);

        if status != NVAPI_OK {
            return Err(NvApiError::ApplicationCreationFailed(status));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn create_application(_profile_handle: NvDRSProfileHandle, _executable: &str, _friendly_name: &str) -> Result<(), NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Create a profile for an executable (combines create_profile + create_application)
#[cfg(target_os = "windows")]
pub fn create_profile_for_executable(executable: &str, profile_name: &str) -> Result<(), NvApiError> {
    use super::profiles::create_profile;
    use super::session::save_settings;

    // Create the profile
    let profile_handle = create_profile(profile_name)?;

    // Add the application to it
    create_application(profile_handle, executable, profile_name)?;

    // Save settings
    save_settings()?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn create_profile_for_executable(_executable: &str, _profile_name: &str) -> Result<(), NvApiError> {
    Err(NvApiError::NotSupported)
}
