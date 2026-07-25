//! LocalSend v2 Protocol DTOs, data structures, and network constants.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Device classification types supported by the LocalSend protocol.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

/// Registration payload exchanged over Multicast UDP or HTTP `/api/localsend/v2/register`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Device info response for `/api/localsend/v2/info`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InfoDto {
    pub alias: String,
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<DeviceType>,
    pub fingerprint: String,
    pub download: bool,
}

/// Metadata describing a single file in a transfer session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Request payload sent to `/api/localsend/v2/prepare-upload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploadReqDto {
    pub info: RegisterDto,
    pub files: HashMap<String, FileDto>,
}

/// Response payload returned by `/api/localsend/v2/prepare-upload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploadRespDto {
    pub session_id: String,
    pub files: HashMap<String, String>,
}

/// Represents a discovered LocalSend peer device on the network.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// LocalSend multicast IPv4 address.
pub const LOCALSEND_MULTICAST_ADDR: &str = "224.0.0.167";
/// Default LocalSend UDP/HTTPS port.
pub const LOCALSEND_DEFAULT_PORT: u16 = 53317;
/// LocalSend protocol version string.
pub const PROTOCOL_VERSION: &str = "2.1";
