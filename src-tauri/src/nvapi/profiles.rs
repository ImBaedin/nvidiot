//! DRS Profile management
//!
//! Handles enumerating, creating, and finding profiles.

use super::error::{NvApiError, NVAPI_OK, NVAPI_END_ENUMERATION, NVAPI_PROFILE_NOT_FOUND};
use super::ffi::{
    get_nvapi, wchar_to_string, string_to_wchar,
    NvDRSProfileHandle, NvdrsProfile, NVDRS_PROFILE_VER,
};
use super::session::get_session;
use super::types::DrsProfile;

/// Get the total number of profiles
#[cfg(target_os = "windows")]
pub fn get_profile_count() -> Result<u32, NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let get_num = api.drs_get_num_profiles
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_GetNumProfiles".to_string()))?;

    unsafe {
        let mut count: u32 = 0;
        let status = get_num(session, &mut count);
        if status != NVAPI_OK {
            return Err(NvApiError::NvApiStatus(status));
        }
        Ok(count)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_profile_count() -> Result<u32, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Enumerate all profiles
#[cfg(target_os = "windows")]
pub fn enumerate_profiles() -> Result<Vec<DrsProfile>, NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let enum_profiles = api.drs_enum_profiles
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_EnumProfiles".to_string()))?;
    let get_profile_info = api.drs_get_profile_info
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_GetProfileInfo".to_string()))?;

    let mut profiles = Vec::new();
    let mut index: u32 = 0;

    unsafe {
        loop {
            let mut profile_handle: NvDRSProfileHandle = std::ptr::null_mut();
            let status = enum_profiles(session, index, &mut profile_handle);

            if status == NVAPI_END_ENUMERATION {
                break;
            }
            if status != NVAPI_OK {
                return Err(NvApiError::NvApiStatus(status));
            }

            // Get profile info
            let mut profile_info = NvdrsProfile::default();
            let status = get_profile_info(session, profile_handle, &mut profile_info);

            if status == NVAPI_OK {
                profiles.push(DrsProfile {
                    name: wchar_to_string(&profile_info.profile_name),
                    is_predefined: profile_info.is_predefined != 0,
                    application_count: profile_info.num_of_apps,
                });
            }

            index += 1;
        }
    }

    Ok(profiles)
}

#[cfg(not(target_os = "windows"))]
pub fn enumerate_profiles() -> Result<Vec<DrsProfile>, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Find a profile by name
#[cfg(target_os = "windows")]
pub fn find_profile_by_name(name: &str) -> Result<NvDRSProfileHandle, NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let find_fn = api.drs_find_profile_by_name
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_FindProfileByName".to_string()))?;

    let mut wide_name = [0u16; 2048];
    string_to_wchar(name, &mut wide_name);

    unsafe {
        let mut profile_handle: NvDRSProfileHandle = std::ptr::null_mut();
        let status = find_fn(session, wide_name.as_ptr(), &mut profile_handle);

        if status == NVAPI_PROFILE_NOT_FOUND {
            return Err(NvApiError::ProfileNotFound(name.to_string()));
        }
        if status != NVAPI_OK {
            return Err(NvApiError::NvApiStatus(status));
        }

        Ok(profile_handle)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn find_profile_by_name(_name: &str) -> Result<NvDRSProfileHandle, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Create a new profile
#[cfg(target_os = "windows")]
pub fn create_profile(name: &str) -> Result<NvDRSProfileHandle, NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let create_fn = api.drs_create_profile
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_CreateProfile".to_string()))?;

    let mut profile_info = NvdrsProfile {
        version: NVDRS_PROFILE_VER,
        ..Default::default()
    };
    string_to_wchar(name, &mut profile_info.profile_name);

    unsafe {
        let mut profile_handle: NvDRSProfileHandle = std::ptr::null_mut();
        let status = create_fn(session, &mut profile_info, &mut profile_handle);

        if status != NVAPI_OK {
            return Err(NvApiError::ProfileCreationFailed(status));
        }

        Ok(profile_handle)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn create_profile(_name: &str) -> Result<NvDRSProfileHandle, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Get the base profile (applies to all applications by default)
#[cfg(target_os = "windows")]
pub fn get_base_profile() -> Result<NvDRSProfileHandle, NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let get_base = api.drs_get_base_profile
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_GetBaseProfile".to_string()))?;

    unsafe {
        let mut profile_handle: NvDRSProfileHandle = std::ptr::null_mut();
        let status = get_base(session, &mut profile_handle);

        if status != NVAPI_OK {
            return Err(NvApiError::NvApiStatus(status));
        }

        Ok(profile_handle)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_base_profile() -> Result<NvDRSProfileHandle, NvApiError> {
    Err(NvApiError::NotSupported)
}
