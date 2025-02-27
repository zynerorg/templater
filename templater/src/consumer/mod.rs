use crate::data::Address;

pub mod prometheus;
pub mod rfc1035;

pub trait Consumer {
    fn consume(&self, addresses: Vec<Address>) -> anyhow::Result<()>;
}
