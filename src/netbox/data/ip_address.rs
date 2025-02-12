use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use ipnet::IpNet;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    common::{
        AssignedObject, BriefSite, BriefTenant, BriefVlan, BriefVrf, CustomFields, Family,
        Intermediate, Tag,
    },
    tenant::Tenant,
};

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
    pub site: Option<BriefSite>,
    pub vlan: Option<BriefVlan>,
    pub full_tenant: Option<Tenant>,
    pub domain: Option<String>,
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
pub struct Domains(pub Vec<Domain>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub addresses: Vec<IpAddress>,
}

impl From<Vec<IpAddress>> for Domains {
    fn from(mut addresses: Vec<IpAddress>) -> Self {
        let mut domains = Vec::new();
        let mut domain = Domain::default();
        addresses.sort_by(|a, b| a.domain.cmp(&b.domain));

        for address in addresses {
            let Some(domain_i) = &address.domain else {
                continue;
            };
            if &domain.name != domain_i {
                if !domain.name.is_empty() {
                    domains.push(domain);
                }
                domain = Domain {
                    name: domain_i.to_string(),
                    addresses: Vec::new(),
                };
            }
            domain.addresses.push(address);
        }

        domains.push(domain);

        Self(domains)
    }
}

impl FromStr for Status {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "active" => Status::Active,
            "reserved" => Status::Reserved,
            "deprecated" => Status::Deprecated,
            "dhcp" => Status::Dhcp,
            "slaac" => Status::Slaac,
            _ => return Err("Unexpected status".into()),
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
    type Error = String;
    fn try_from(value: Intermediate) -> Result<Self, Self::Error> {
        Self::from_str(&value.value)
    }
}

impl TryFrom<Intermediate> for Role {
    type Error = String;
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
            _ => return Err("Unexpected role".into()),
        })
    }
}
