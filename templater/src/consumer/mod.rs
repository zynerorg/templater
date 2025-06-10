use crate::data::AddressMain;

pub mod cloudflare;
pub mod prometheus;
pub mod rfc1035;

pub trait Consumer {
    fn consume(&self, addresses: Vec<AddressMain>) -> anyhow::Result<()>;
}
