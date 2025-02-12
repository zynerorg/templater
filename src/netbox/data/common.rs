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
    pub value: i64,
    pub label: String,
}

impl TryFrom<IntermediateFamily> for Family {
    type Error = String;
    fn try_from(value: IntermediateFamily) -> Result<Self, Self::Error> {
        Ok(match value.value {
            4 => Family::IPv4,
            6 => Family::IPv6,
            _ => return Err("Unexpected IP family".into()),
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
