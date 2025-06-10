use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub paused: bool,
    #[serde(rename = "type")]
    pub type_field: Type,
    pub development_mode: i64,
    pub name_servers: Vec<String>,
    pub original_name_servers: Option<Vec<String>>,
    pub original_registrar: Option<String>,
    pub original_dnshost: Option<String>,
    pub modified_on: DateTime<Utc>,
    pub created_on: DateTime<Utc>,
    pub activated_on: DateTime<Utc>,
    pub meta: Meta,
    pub owner: Owner,
    pub account: Account,
    pub tenant: Tenant,
    pub tenant_unit: TenantUnit,
    pub permissions: Vec<String>,
    pub plan: Value, // Deprecated in API
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Initializing,
    Pending,
    Active,
    Moved,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    #[default]
    Full,
    Partial,
    Secondary,
    Internal,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub step: isize,
    pub custom_certificate_quota: isize,
    pub page_rule_quota: isize,
    #[serde(default)]
    pub phishing_detected: bool,
    #[serde(default)]
    pub cdn_only: bool,
    #[serde(default)]
    pub dns_only: bool,
    #[serde(default)]
    pub foundation_dns: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Owner {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub email: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TenantUnit {
    pub id: Option<String>,
}
