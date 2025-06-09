use std::{fmt::Display, str::FromStr};

use anyhow::{Error, anyhow};
use chrono::{DateTime, Utc};
use ipnet::IpNet;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    common::{
        AssignedObject, BriefSite, BriefTenant, BriefVlan, BriefVrf, Family, Intermediate, Tag,
    },
    prefix::Scope,
    site::Site,
    tenant::Tenant,
};
use crate::data::{AddressMain, Location};

#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IpAddress {
    pub id: i64,
    pub url: String,
    pub display_url: String,
    pub display: String,
    pub family: Family,
    pub address: IpNet,
    pub vrf: Option<BriefVrf>,
    pub tenant: Option<BriefTenant>,
    pub status: Status,
    pub role: Option<Role>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub assigned_object_type: Option<String>,
    pub assigned_object_id: Option<i64>,
    pub assigned_object: Option<AssignedObject>,
    pub nat_inside: Option<Value>,
    pub nat_outside: Vec<Value>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub dns_name: Option<String>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub description: Option<String>,
    #[serde(deserialize_with = "super::common::non_empty_str")]
    pub comments: Option<String>,
    pub tags: Vec<Tag>,
    pub custom_fields: CustomFields,
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,

    // Custom
    pub scope: Option<Scope>,
    pub vlan: Option<BriefVlan>,
    pub full_tenant: Option<Tenant>,
    pub full_site: Option<Site>,
    pub domain: Option<String>,
    pub prefix: Option<IpNet>,
}

impl From<IpAddress> for AddressMain {
    fn from(ip: IpAddress) -> Self {
        Self {
            address: Some(ip.address.addr()),
            family: ip.family.try_into().ok(),
            id: Some(ip.id),
            dns_name: ip.dns_name,
            tenant: ip.tenant.map(|s| s.slug),
            tenant_group: ip.full_tenant.and_then(|s| s.group.map(|s| s.slug)),
            status: Some(ip.status.to_string()),
            site: ip
                .scope
                .and_then(|s| TryInto::<BriefSite>::try_into(s).ok().map(|s| s.slug)),
            vlan: None,
            alias: ip
                .custom_fields
                .alias
                .map(|s| s.lines().map(|s| s.trim().to_string()).collect()),
            tags: Some(ip.tags.into_iter().map(|tag| tag.slug).collect()),
            location: ip.full_site.and_then(|s| {
                s.latitude.and_then(|latitude| {
                    s.longitude.map(|longitude| Location {
                        latitude,
                        longitude,
                    })
                })
            }),
            role: ip.role.map(|r| r.to_string()),
            prefix: ip.prefix,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Intermediate")]
pub enum Status {
    #[default]
    Active,
    Reserved,
    Deprecated,
    Dhcp,
    Slaac,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Intermediate")]
pub enum Role {
    #[default]
    Loopback,
    Secondary,
    Anycast,
    Vip,
    Vrrp,
    Hsrp,
    Glbp,
    Carp,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomFields {
    /// TODO: Make this optional
    pub alias: Option<String>,
}

impl FromStr for Status {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "active" => Status::Active,
            "reserved" => Status::Reserved,
            "deprecated" => Status::Deprecated,
            "dhcp" => Status::Dhcp,
            "slaac" => Status::Slaac,
            _ => return Err(anyhow!("Unexpected status")),
        })
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Status::Active => "active",
            Status::Reserved => "reserved",
            Status::Deprecated => "deprecated",
            Status::Dhcp => "dhcp",
            Status::Slaac => "slaac",
        };
        write!(f, "{str}")
    }
}

impl TryFrom<Intermediate> for Status {
    type Error = Error;
    fn try_from(value: Intermediate) -> Result<Self, Self::Error> {
        Self::from_str(&value.value)
    }
}

impl TryFrom<Intermediate> for Role {
    type Error = Error;
    fn try_from(value: Intermediate) -> Result<Self, Self::Error> {
        Ok(match value.value.as_str() {
            "loopback" => Role::Loopback,
            "secondary" => Role::Secondary,
            "anycast" => Role::Anycast,
            "vip" => Role::Vip,
            "vrrp" => Role::Vrrp,
            "hsrp" => Role::Hsrp,
            "glbp" => Role::Glbp,
            "carp" => Role::Carp,
            _ => return Err(anyhow!("Unexpected role")),
        })
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Role::Loopback => "loopback",
            Role::Secondary => "secondary",
            Role::Anycast => "anycast",
            Role::Vip => "vip",
            Role::Vrrp => "vrrp",
            Role::Hsrp => "hsrp",
            Role::Glbp => "glbp",
            Role::Carp => "carp",
        };
        write!(f, "{str}")
    }
}