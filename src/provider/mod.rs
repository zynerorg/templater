use anyhow::Result;

use crate::data::Address;

pub mod netbox;

pub trait Provider {
    fn provide(&self) -> Result<Vec<Address>>;
}
