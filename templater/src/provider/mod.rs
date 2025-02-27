use anyhow::Result;

use crate::data::AddressMain;

pub mod netbox;
pub mod yaml;

pub trait Provider {
    fn provide(self) -> Result<Vec<AddressMain>>;
}
