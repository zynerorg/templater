use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use super::common::{CustomFields, Tag};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tenant {
    pub id: i64,
    pub url: String,
    pub display_url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    pub group: Option<Group>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub description: Option<String>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub comments: Option<String>,
    pub tags: Vec<Tag>,
    pub custom_fields: CustomFields,
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub circuit_count: i64,
    pub device_count: i64,
    pub ipaddress_count: i64,
    pub prefix_count: i64,
    pub rack_count: i64,
    pub site_count: i64,
    pub virtualmachine_count: i64,
    pub vlan_count: i64,
    pub vrf_count: i64,
    pub cluster_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub tenant_count: i64,
    #[serde(rename = "_depth")]
    pub depth: i64,
}
