//! DRS Session management
//!
//! Handles creating, loading, saving, and destroying DRS sessions.

use std::sync::Mutex;
use once_cell::sync::OnceCell;
use super::error::NvApiError;
#[cfg(target_os = "windows")]
use super::error::NVAPI_OK;
use super::ffi::NvDRSSessionHandle;
#[cfg(target_os = "windows")]
use super::ffi::get_nvapi;

/// Wrapper for NvDRSSessionHandle that implements Send + Sync
/// SAFETY: NVAPI session handles are safe to use from multiple threads
/// when protected by synchronization (the Mutex provides this).
struct SessionHandle(NvDRSSessionHandle);

// SAFETY: The handle is protected by a Mutex, ensuring exclusive access
unsafe impl Send for SessionHandle {}
unsafe impl Sync for SessionHandle {}

/// Global DRS session handle with mutex for thread safety
static DRS_SESSION: OnceCell<Mutex<SessionHandle>> = OnceCell::new();

/// Create a new DRS session and load settings
#[cfg(target_os = "windows")]
pub fn create_session() -> Result<NvDRSSessionHandle, NvApiError> {
    let api = get_nvapi()?;

    let create_session = api.drs_create_session
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_CreateSession".to_string()))?;
    let load_settings = api.drs_load_settings
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_LoadSettings".to_string()))?;

    unsafe {
        let mut handle: NvDRSSessionHandle = std::ptr::null_mut();

        let status = create_session(&mut handle);
        if status != NVAPI_OK {
            return Err(NvApiError::SessionCreationFailed(status));
        }

        let status = load_settings(handle);
        if status != NVAPI_OK {
            // Clean up on failure
            if let Some(destroy) = api.drs_destroy_session {
                destroy(handle);
            }
            return Err(NvApiError::LoadSettingsFailed(status));
        }

        Ok(handle)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn create_session() -> Result<NvDRSSessionHandle, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Get or create the global DRS session
#[cfg(target_os = "windows")]
pub fn get_session() -> Result<NvDRSSessionHandle, NvApiError> {
    let mutex = DRS_SESSION.get_or_try_init(|| {
        let handle = create_session()?;
        Ok::<_, NvApiError>(Mutex::new(SessionHandle(handle)))
    })?;

    let guard = mutex.lock().unwrap();
    Ok(guard.0)
}

#[cfg(not(target_os = "windows"))]
pub fn get_session() -> Result<NvDRSSessionHandle, NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Save current settings to disk
#[cfg(target_os = "windows")]
pub fn save_settings() -> Result<(), NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let save_fn = api.drs_save_settings
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_SaveSettings".to_string()))?;

    unsafe {
        let status = save_fn(session);
        if status != NVAPI_OK {
            return Err(NvApiError::SaveSettingsFailed(status));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn save_settings() -> Result<(), NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Reload settings from disk (useful after external changes)
#[cfg(target_os = "windows")]
pub fn reload_settings() -> Result<(), NvApiError> {
    let api = get_nvapi()?;
    let session = get_session()?;

    let load_fn = api.drs_load_settings
        .ok_or_else(|| NvApiError::FunctionNotFound("NvAPI_DRS_LoadSettings".to_string()))?;

    unsafe {
        let status = load_fn(session);
        if status != NVAPI_OK {
            return Err(NvApiError::LoadSettingsFailed(status));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn reload_settings() -> Result<(), NvApiError> {
    Err(NvApiError::NotSupported)
}

/// Check if NVAPI is available
pub fn check_nvapi() -> Result<(), NvApiError> {
    #[cfg(target_os = "windows")]
    {
        get_nvapi()?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(NvApiError::NotSupported)
    }
}
