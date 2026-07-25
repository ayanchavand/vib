use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDto {
    #[serde(default)]
    pub alias: String,
    #[serde(default)]
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<DeviceType>,
    #[serde(default)]
    pub fingerprint: String,
    pub port: Option<u16>,
    pub protocol: Option<String>,
    pub download: Option<bool>,
    pub announce: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoDto {
    pub alias: String,
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<DeviceType>,
    pub fingerprint: String,
    pub download: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDto {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub size: u64,
    pub file_type: Option<String>,
    pub sha256: Option<String>,
    pub preview: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploadReqDto {
    pub info: RegisterDto,
    pub files: HashMap<String, FileDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploadRespDto {
    pub session_id: String,
    pub files: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub alias: String,
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<DeviceType>,
    pub fingerprint: String,
    pub ip: String,
    pub port: u16,
    pub protocol: String,
}

pub const LOCALSEND_MULTICAST_ADDR: &str = "224.0.0.167";
pub const LOCALSEND_DEFAULT_PORT: u16 = 53317;
pub const PROTOCOL_VERSION: &str = "2.1";
