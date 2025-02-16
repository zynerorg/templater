use std::str::FromStr;

use anyhow::{anyhow, Error};
use serde::Deserializer;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intermediate {
    pub value: String,
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "IntermediateFamily")]
pub enum Family {
    #[default]
    IPv4,
    IPv6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateFamily {
    pub value: u8, // Can only be 4 or 6 so u8 is enough
    pub label: String,
}

impl TryFrom<IntermediateFamily> for Family {
    type Error = Error;
    fn try_from(value: IntermediateFamily) -> Result<Self, Self::Error> {
        value.value.try_into()
    }
}

impl From<Family> for u8 {
    fn from(value: Family) -> Self {
        match value {
            Family::IPv4 => 4,
            Family::IPv6 => 6,
        }
    }
}

impl TryFrom<u8> for Family {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            4 => Self::IPv4,
            6 => Self::IPv6,
            _ => return Err(anyhow!("Unexpected integer")),
        })
    }
}

impl FromStr for Family {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ipv4" => Self::IPv4,
            "ipv6" => Self::IPv6,
            _ => u8::from_str(s)?.try_into()?,
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BriefTenant {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BriefVrf {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub name: String,
    pub rd: String,
    pub description: String,
    pub prefix_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignedObject {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub device: BriefDevice,
    pub name: String,
    pub description: String,
    pub cable: BriefCable,
    #[serde(rename = "_occupied")]
    pub occupied: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BriefDevice {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub name: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BriefCable {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub label: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub url: String,
    pub display_url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    pub color: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BriefSite {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BriefVlan {
    pub id: i64,
    pub url: String,
    pub display: String,
    pub vid: i64,
    pub name: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomFields {}

pub fn non_empty_str<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    use serde::Deserialize;
    let o: Option<String> = Option::deserialize(d)?;
    Ok(o.filter(|s| !s.is_empty()))
}
