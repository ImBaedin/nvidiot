use serde::{Deserialize, Serialize};

/// A DRS profile containing driver settings for applications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrsProfile {
    pub name: String,
    pub is_predefined: bool,
    pub application_count: u32,
}

/// An application registered in a DRS profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrsApplication {
    pub name: String,
    pub executable: String,
    pub profile_name: String,
    pub is_predefined: bool,
    pub is_blacklisted: bool,
}

/// A running process on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningProcess {
    pub process_name: String,
    pub window_title: String,
    pub process_id: u32,
    pub executable_path: Option<String>,
    pub has_drs_profile: bool,
    pub profile_name: Option<String>,
    pub is_blacklisted: Option<bool>,
}

/// The currently focused application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusApplication {
    pub process_name: String,
    pub window_title: String,
    pub process_id: u32,
    pub is_in_drs: bool,
    pub profile_name: Option<String>,
    pub is_blacklisted: Option<bool>,
}

/// Result of a blacklist operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlacklistResult {
    pub success: bool,
    pub executable: String,
    pub message: String,
}

/// NVAPI connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NvApiStatus {
    pub available: bool,
    pub error: Option<String>,
}
