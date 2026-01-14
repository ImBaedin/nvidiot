use thiserror::Error;

#[derive(Error, Debug)]
pub enum NvApiError {
    #[error("NVAPI library not found - ensure NVIDIA drivers are installed")]
    LibraryNotFound,

    #[error("NVAPI initialization failed: {0}")]
    InitializationFailed(i32),

    #[error("No NVIDIA GPU found")]
    NoGpuFound,

    #[error("DRS session creation failed: {0}")]
    SessionCreationFailed(i32),

    #[error("Failed to load settings: {0}")]
    LoadSettingsFailed(i32),

    #[error("Failed to save settings: {0}")]
    SaveSettingsFailed(i32),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Application not found: {0}")]
    ApplicationNotFound(String),

    #[error("Failed to create profile: {0}")]
    ProfileCreationFailed(i32),

    #[error("Failed to create application: {0}")]
    ApplicationCreationFailed(i32),

    #[error("Failed to set setting: {0}")]
    SetSettingFailed(i32),

    #[error("Failed to get setting: {0}")]
    GetSettingFailed(i32),

    #[error("Function not found in NVAPI: {0}")]
    FunctionNotFound(String),

    #[error("NVAPI error code: {0}")]
    NvApiStatus(i32),

    #[error("Not supported on this platform")]
    NotSupported,
}

impl From<NvApiError> for String {
    fn from(err: NvApiError) -> String {
        err.to_string()
    }
}

// NVAPI status codes
pub const NVAPI_OK: i32 = 0;
pub const NVAPI_ERROR: i32 = -1;
pub const NVAPI_LIBRARY_NOT_FOUND: i32 = -2;
pub const NVAPI_NO_IMPLEMENTATION: i32 = -3;
pub const NVAPI_API_NOT_INITIALIZED: i32 = -4;
pub const NVAPI_INVALID_ARGUMENT: i32 = -5;
pub const NVAPI_NVIDIA_DEVICE_NOT_FOUND: i32 = -6;
pub const NVAPI_END_ENUMERATION: i32 = -7;
pub const NVAPI_INVALID_HANDLE: i32 = -8;
pub const NVAPI_INCOMPATIBLE_STRUCT_VERSION: i32 = -9;
pub const NVAPI_PROFILE_NOT_FOUND: i32 = -175;
pub const NVAPI_PROFILE_NAME_IN_USE: i32 = -176;
pub const NVAPI_EXECUTABLE_NOT_FOUND: i32 = -183;
pub const NVAPI_EXECUTABLE_ALREADY_IN_USE: i32 = -184;
pub const NVAPI_SETTING_NOT_FOUND: i32 = -179;
