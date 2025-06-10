use serde_derive::{Deserialize, Serialize};

use super::record::Record;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Batch {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deletes: Vec<Record>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub posts: Vec<Record>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Id {
    id: String,
}
