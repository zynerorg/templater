use anyhow::Error;
use serde::Deserializer;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intermediate {
    pub value: String,
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Family {
    pub value: u8, // Can only be 4 or 6 so u8 is enough
    pub label: String,
}

use crate::data::Family as DataFamily;
impl TryFrom<Family> for DataFamily {
    type Error = Error;
    fn try_from(value: Family) -> Result<Self, Self::Error> {
        value.value.try_into()
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
