//! FFI bindings to NVAPI DRS functions
//!
//! NVAPI uses a query interface pattern where functions are retrieved by ID.
//! We load nvapi64.dll, get NvAPI_QueryInterface, then use it to get other functions.

#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::c_void;
use std::ptr;
use once_cell::sync::OnceCell;
use super::error::{NvApiError, NVAPI_OK};

#[cfg(target_os = "windows")]
use libloading::{Library, Symbol};

// Type aliases for NVAPI handles
pub type NvDRSSessionHandle = *mut c_void;
pub type NvDRSProfileHandle = *mut c_void;

// NVAPI function IDs (from nvapi headers)
const NVAPI_INITIALIZE: u32 = 0x0150E828;
const NVAPI_UNLOAD: u32 = 0xD22BDD7E;
const NVAPI_DRS_CREATE_SESSION: u32 = 0x0694D52E;
const NVAPI_DRS_DESTROY_SESSION: u32 = 0xDAD9CFF8;
const NVAPI_DRS_LOAD_SETTINGS: u32 = 0x375DBD6B;
const NVAPI_DRS_SAVE_SETTINGS: u32 = 0xFCBC7E14;
const NVAPI_DRS_GET_NUM_PROFILES: u32 = 0x1DAE4FBC;
const NVAPI_DRS_ENUM_PROFILES: u32 = 0xBC371EE0;
const NVAPI_DRS_GET_PROFILE_INFO: u32 = 0x61CD6FD6;
const NVAPI_DRS_FIND_PROFILE_BY_NAME: u32 = 0x7E4A9A0B;
const NVAPI_DRS_CREATE_PROFILE: u32 = 0xCC176068;
const NVAPI_DRS_DELETE_PROFILE: u32 = 0x17093206;
const NVAPI_DRS_ENUM_APPLICATIONS: u32 = 0x7FA2173A;
const NVAPI_DRS_FIND_APPLICATION_BY_NAME: u32 = 0xEEE566B2;
const NVAPI_DRS_CREATE_APPLICATION: u32 = 0x4347A9DE;
const NVAPI_DRS_DELETE_APPLICATION: u32 = 0x2C694BC6;
const NVAPI_DRS_GET_SETTING: u32 = 0x73BF8338;
const NVAPI_DRS_SET_SETTING: u32 = 0x577DD202;
const NVAPI_DRS_GET_BASE_PROFILE: u32 = 0xDA8466A0;

// Structure versions (from nvapi headers)
pub const NVDRS_PROFILE_VER: u32 = 0x10028; // MAKE_NVAPI_VERSION(NVDRS_PROFILE, 1)
pub const NVDRS_APPLICATION_VER: u32 = 0x30038; // MAKE_NVAPI_VERSION(NVDRS_APPLICATION, 3)
pub const NVDRS_SETTING_VER: u32 = 0x10058; // MAKE_NVAPI_VERSION(NVDRS_SETTING, 1)

// Constants
pub const NVAPI_UNICODE_STRING_MAX: usize = 2048;
pub const NVAPI_SETTING_MAX_VALUES: usize = 100;

// ShadowPlay setting
pub const SHADOWPLAY_SETTING_ID: u32 = 0x809D5F60;
pub const SHADOWPLAY_DISABLED: u32 = 0x10000000;
pub const SHADOWPLAY_ENABLED: u32 = 0x08000001;

/// NVDRS_PROFILE structure
#[repr(C)]
#[derive(Clone)]
pub struct NvdrsProfile {
    pub version: u32,
    pub profile_name: [u16; NVAPI_UNICODE_STRING_MAX],
    pub gpu_support: u32,
    pub is_predefined: u32,
    pub num_of_apps: u32,
    pub num_of_settings: u32,
}

impl Default for NvdrsProfile {
    fn default() -> Self {
        Self {
            version: NVDRS_PROFILE_VER,
            profile_name: [0u16; NVAPI_UNICODE_STRING_MAX],
            gpu_support: 0,
            is_predefined: 0,
            num_of_apps: 0,
            num_of_settings: 0,
        }
    }
}

/// NVDRS_APPLICATION structure
#[repr(C)]
#[derive(Clone)]
pub struct NvdrsApplication {
    pub version: u32,
    pub is_predefined: u32,
    pub app_name: [u16; NVAPI_UNICODE_STRING_MAX],
    pub user_friendly_name: [u16; NVAPI_UNICODE_STRING_MAX],
    pub launcher: [u16; NVAPI_UNICODE_STRING_MAX],
}

impl Default for NvdrsApplication {
    fn default() -> Self {
        Self {
            version: NVDRS_APPLICATION_VER,
            is_predefined: 0,
            app_name: [0u16; NVAPI_UNICODE_STRING_MAX],
            user_friendly_name: [0u16; NVAPI_UNICODE_STRING_MAX],
            launcher: [0u16; NVAPI_UNICODE_STRING_MAX],
        }
    }
}

/// Setting type enum
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NvdrsSettingType {
    Dword = 0,
    Binary = 1,
    String = 2,
    WString = 3,
}

/// NVDRS_SETTING_UNION
#[repr(C)]
#[derive(Clone, Copy)]
pub union NvdrsSettingValue {
    pub dword_value: u32,
    pub binary_value: [u8; NVAPI_SETTING_MAX_VALUES],
}

impl Default for NvdrsSettingValue {
    fn default() -> Self {
        Self { dword_value: 0 }
    }
}

/// NVDRS_SETTING structure
#[repr(C)]
#[derive(Clone)]
pub struct NvdrsSetting {
    pub version: u32,
    pub setting_name: [u16; NVAPI_UNICODE_STRING_MAX],
    pub setting_id: u32,
    pub setting_type: u32,
    pub setting_location: u32,
    pub is_current_predefined: u32,
    pub is_predefined_valid: u32,
    pub predefined_value: NvdrsSettingValue,
    pub current_value: NvdrsSettingValue,
}

impl Default for NvdrsSetting {
    fn default() -> Self {
        Self {
            version: NVDRS_SETTING_VER,
            setting_name: [0u16; NVAPI_UNICODE_STRING_MAX],
            setting_id: 0,
            setting_type: 0,
            setting_location: 0,
            is_current_predefined: 0,
            is_predefined_valid: 0,
            predefined_value: NvdrsSettingValue::default(),
            current_value: NvdrsSettingValue::default(),
        }
    }
}

// Function pointer types
type NvApiQueryInterfaceFn = unsafe extern "C" fn(id: u32) -> *mut c_void;
type NvApiInitializeFn = unsafe extern "C" fn() -> i32;
type NvApiUnloadFn = unsafe extern "C" fn() -> i32;
type NvApiDrsCreateSessionFn = unsafe extern "C" fn(session: *mut NvDRSSessionHandle) -> i32;
type NvApiDrsDestroySessionFn = unsafe extern "C" fn(session: NvDRSSessionHandle) -> i32;
type NvApiDrsLoadSettingsFn = unsafe extern "C" fn(session: NvDRSSessionHandle) -> i32;
type NvApiDrsSaveSettingsFn = unsafe extern "C" fn(session: NvDRSSessionHandle) -> i32;
type NvApiDrsGetNumProfilesFn = unsafe extern "C" fn(session: NvDRSSessionHandle, count: *mut u32) -> i32;
type NvApiDrsEnumProfilesFn = unsafe extern "C" fn(session: NvDRSSessionHandle, index: u32, profile: *mut NvDRSProfileHandle) -> i32;
type NvApiDrsGetProfileInfoFn = unsafe extern "C" fn(session: NvDRSSessionHandle, profile: NvDRSProfileHandle, info: *mut NvdrsProfile) -> i32;
type NvApiDrsFindProfileByNameFn = unsafe extern "C" fn(session: NvDRSSessionHandle, name: *const u16, profile: *mut NvDRSProfileHandle) -> i32;
type NvApiDrsCreateProfileFn = unsafe extern "C" fn(session: NvDRSSessionHandle, info: *mut NvdrsProfile, profile: *mut NvDRSProfileHandle) -> i32;
type NvApiDrsEnumApplicationsFn = unsafe extern "C" fn(session: NvDRSSessionHandle, profile: NvDRSProfileHandle, start: u32, count: *mut u32, apps: *mut NvdrsApplication) -> i32;
type NvApiDrsFindApplicationByNameFn = unsafe extern "C" fn(session: NvDRSSessionHandle, name: *const u16, profile: *mut NvDRSProfileHandle, app: *mut NvdrsApplication) -> i32;
type NvApiDrsCreateApplicationFn = unsafe extern "C" fn(session: NvDRSSessionHandle, profile: NvDRSProfileHandle, app: *mut NvdrsApplication) -> i32;
type NvApiDrsGetSettingFn = unsafe extern "C" fn(session: NvDRSSessionHandle, profile: NvDRSProfileHandle, setting_id: u32, setting: *mut NvdrsSetting) -> i32;
type NvApiDrsSetSettingFn = unsafe extern "C" fn(session: NvDRSSessionHandle, profile: NvDRSProfileHandle, setting: *mut NvdrsSetting) -> i32;
type NvApiDrsGetBaseProfileFn = unsafe extern "C" fn(session: NvDRSSessionHandle, profile: *mut NvDRSProfileHandle) -> i32;

/// NVAPI function pointers
#[cfg(target_os = "windows")]
pub struct NvApi {
    _library: Library,
    query_interface: NvApiQueryInterfaceFn,
    pub initialize: Option<NvApiInitializeFn>,
    pub unload: Option<NvApiUnloadFn>,
    pub drs_create_session: Option<NvApiDrsCreateSessionFn>,
    pub drs_destroy_session: Option<NvApiDrsDestroySessionFn>,
    pub drs_load_settings: Option<NvApiDrsLoadSettingsFn>,
    pub drs_save_settings: Option<NvApiDrsSaveSettingsFn>,
    pub drs_get_num_profiles: Option<NvApiDrsGetNumProfilesFn>,
    pub drs_enum_profiles: Option<NvApiDrsEnumProfilesFn>,
    pub drs_get_profile_info: Option<NvApiDrsGetProfileInfoFn>,
    pub drs_find_profile_by_name: Option<NvApiDrsFindProfileByNameFn>,
    pub drs_create_profile: Option<NvApiDrsCreateProfileFn>,
    pub drs_enum_applications: Option<NvApiDrsEnumApplicationsFn>,
    pub drs_find_application_by_name: Option<NvApiDrsFindApplicationByNameFn>,
    pub drs_create_application: Option<NvApiDrsCreateApplicationFn>,
    pub drs_get_setting: Option<NvApiDrsGetSettingFn>,
    pub drs_set_setting: Option<NvApiDrsSetSettingFn>,
    pub drs_get_base_profile: Option<NvApiDrsGetBaseProfileFn>,
}

#[cfg(target_os = "windows")]
impl NvApi {
    fn get_fn<T>(&self, id: u32) -> Option<T> {
        unsafe {
            let ptr = (self.query_interface)(id);
            if ptr.is_null() {
                None
            } else {
                Some(std::mem::transmute_copy(&ptr))
            }
        }
    }

    pub fn load() -> Result<Self, NvApiError> {
        unsafe {
            let library = Library::new("nvapi64.dll")
                .map_err(|_| NvApiError::LibraryNotFound)?;

            let query_interface: Symbol<NvApiQueryInterfaceFn> = library
                .get(b"nvapi_QueryInterface\0")
                .map_err(|_| NvApiError::FunctionNotFound("nvapi_QueryInterface".to_string()))?;

            let query_interface = *query_interface;

            let mut api = Self {
                _library: library,
                query_interface,
                initialize: None,
                unload: None,
                drs_create_session: None,
                drs_destroy_session: None,
                drs_load_settings: None,
                drs_save_settings: None,
                drs_get_num_profiles: None,
                drs_enum_profiles: None,
                drs_get_profile_info: None,
                drs_find_profile_by_name: None,
                drs_create_profile: None,
                drs_enum_applications: None,
                drs_find_application_by_name: None,
                drs_create_application: None,
                drs_get_setting: None,
                drs_set_setting: None,
                drs_get_base_profile: None,
            };

            // Load function pointers
            api.initialize = api.get_fn(NVAPI_INITIALIZE);
            api.unload = api.get_fn(NVAPI_UNLOAD);
            api.drs_create_session = api.get_fn(NVAPI_DRS_CREATE_SESSION);
            api.drs_destroy_session = api.get_fn(NVAPI_DRS_DESTROY_SESSION);
            api.drs_load_settings = api.get_fn(NVAPI_DRS_LOAD_SETTINGS);
            api.drs_save_settings = api.get_fn(NVAPI_DRS_SAVE_SETTINGS);
            api.drs_get_num_profiles = api.get_fn(NVAPI_DRS_GET_NUM_PROFILES);
            api.drs_enum_profiles = api.get_fn(NVAPI_DRS_ENUM_PROFILES);
            api.drs_get_profile_info = api.get_fn(NVAPI_DRS_GET_PROFILE_INFO);
            api.drs_find_profile_by_name = api.get_fn(NVAPI_DRS_FIND_PROFILE_BY_NAME);
            api.drs_create_profile = api.get_fn(NVAPI_DRS_CREATE_PROFILE);
            api.drs_enum_applications = api.get_fn(NVAPI_DRS_ENUM_APPLICATIONS);
            api.drs_find_application_by_name = api.get_fn(NVAPI_DRS_FIND_APPLICATION_BY_NAME);
            api.drs_create_application = api.get_fn(NVAPI_DRS_CREATE_APPLICATION);
            api.drs_get_setting = api.get_fn(NVAPI_DRS_GET_SETTING);
            api.drs_set_setting = api.get_fn(NVAPI_DRS_SET_SETTING);
            api.drs_get_base_profile = api.get_fn(NVAPI_DRS_GET_BASE_PROFILE);

            // Initialize NVAPI
            if let Some(init) = api.initialize {
                let status = init();
                if status != NVAPI_OK {
                    return Err(NvApiError::InitializationFailed(status));
                }
            } else {
                return Err(NvApiError::FunctionNotFound("NvAPI_Initialize".to_string()));
            }

            Ok(api)
        }
    }
}

// Global NVAPI instance
#[cfg(target_os = "windows")]
static NVAPI: OnceCell<Result<NvApi, NvApiError>> = OnceCell::new();

#[cfg(target_os = "windows")]
pub fn get_nvapi() -> Result<&'static NvApi, NvApiError> {
    NVAPI.get_or_init(|| NvApi::load())
        .as_ref()
        .map_err(|e| NvApiError::InitializationFailed(match e {
            NvApiError::InitializationFailed(code) => *code,
            _ => -1,
        }))
}

// Non-Windows stub
#[cfg(not(target_os = "windows"))]
pub fn get_nvapi() -> Result<(), NvApiError> {
    Err(NvApiError::NotSupported)
}

// Helper functions
pub fn wchar_to_string(wchars: &[u16]) -> String {
    let end = wchars.iter().position(|&c| c == 0).unwrap_or(wchars.len());
    String::from_utf16_lossy(&wchars[..end])
}

pub fn string_to_wchar(s: &str, buffer: &mut [u16]) {
    let chars: Vec<u16> = s.encode_utf16().collect();
    let len = chars.len().min(buffer.len() - 1);
    buffer[..len].copy_from_slice(&chars[..len]);
    buffer[len] = 0;
}
