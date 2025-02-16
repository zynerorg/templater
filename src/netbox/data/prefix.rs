use core::convert::TryFrom;

use chrono::{DateTime, Utc};
use derive_more::TryInto;
use ipnet::IpNet;
use serde_derive::{Deserialize, Serialize};

use super::common::{
    BriefSite, BriefTenant, BriefVlan, BriefVrf, CustomFields, Family, Intermediate, Tag,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prefix {
    pub id: i64,
    pub url: String,
    pub display_url: String,
    pub display: String,
    pub family: Family,
    pub prefix: IpNet,
    pub scope: Option<Scope>,
    pub vrf: Option<BriefVrf>,
    pub tenant: Option<BriefTenant>,
    pub vlan: Option<BriefVlan>,
    pub status: Status,
    pub role: Option<Role>,
    pub is_pool: bool,
    pub mark_utilized: bool,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub description: Option<String>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub comments: Option<String>,
    pub tags: Vec<Tag>,
    pub custom_fields: CustomFields,
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub children: i64,
    #[serde(rename = "_depth")]
    pub depth: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Intermediate")]
pub enum Status {
    #[default]
    Container,
    Active,
    Reserved,
    Deprecated,
}

impl TryFrom<Intermediate> for Status {
    type Error = String;
    fn try_from(value: Intermediate) -> Result<Self, Self::Error> {
        Ok(match value.value.as_str() {
            "container" => Status::Container,
            "active" => Status::Active,
            "reserved" => Status::Reserved,
            "deprecated" => Status::Deprecated,
            _ => return Err("Unexpected status".into()),
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub prefix_count: Option<i64>,
    pub vlan_count: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, TryInto)]
#[try_into(owned, ref, ref_mut)]
#[serde(untagged)] // FIXME: Cannot get adjacently tagged to work
pub enum Scope {
    #[default]
    Location,
    Region,
    Site(BriefSite),
    SiteGroup,
}
