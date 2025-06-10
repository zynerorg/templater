pub mod batch;
pub mod record;
pub mod zone;

use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct List<T> {
    pub result: Vec<T>,
    pub result_info: ResultInfo,
    pub success: bool,
    pub errors: Vec<Value>,
    pub messages: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultInfo {
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    pub count: usize,
    pub total_count: usize,
}
