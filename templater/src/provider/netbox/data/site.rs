use std::str::FromStr;

use anyhow::{Error, anyhow};
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use super::common::{BriefTenant, CustomFields, Intermediate, Tag};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Site {
    pub id: i64,
    pub url: String,
    pub display_url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    pub status: Status,
    pub region: Region,
    pub group: Group,
    pub tenant: BriefTenant,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub facility: Option<String>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub time_zone: Option<String>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub description: Option<String>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub physical_address: Option<String>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub shipping_address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub comments: Option<String>,
    pub asns: Vec<Asn>,
    pub tags: Vec<Tag>,
    pub custom_fields: CustomFields,
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub circuit_count: i64,
    pub device_count: i64,
    pub prefix_count: i64,
    pub rack_count: i64,
    pub virtualmachine_count: i64,
    pub vlan_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Intermediate")]
pub enum Status {
    Planned,
    Staging,
    #[default]
    Active,
    Decommissioning,
    Retired,
}

impl TryFrom<Intermediate> for Status {
    type Error = Error;
    fn try_from(value: Intermediate) -> Result<Self, Self::Error> {
        Self::from_str(&value.value)
    }
}

impl FromStr for Status {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "planned" => Status::Planned,
            "staging" => Status::Staging,
            "active" => Status::Active,
            "decommissioning" => Status::Decommissioning,
            "retired" => Status::Retired,
            _ => return Err(anyhow!("Unexpected status")),
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub description: Option<String>,
    pub site_count: i64,
    #[serde(rename = "_depth")]
    pub depth: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub description: Option<String>,
    pub site_count: i64,
    #[serde(rename = "_depth")]
    pub depth: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asn {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub asn: i64,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub description: Option<String>,
}
