use std::{net::IpAddr, str::FromStr};

use anyhow::{anyhow, Error, Result};
use clap::Args;
use serde_derive::{Deserialize, Serialize};
use tldextract::TldExtractor;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub address: Option<IpAddr>,
    pub family: Option<Family>,
    pub id: Option<i64>,
    pub dns_name: Option<String>,
    pub domain: Option<String>,
    pub tenant: Option<String>,
    pub tenant_group: Option<String>,
    pub status: Option<String>,
    pub site: Option<String>,
    pub vlan: Option<u16>,
}

// There must be a better way than to dupe this, macro maybe?
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
pub struct AddressFilter {
    #[arg(long, value_delimiter = ',')]
    pub address: Option<Vec<IpAddr>>,
    #[arg(long, value_delimiter = ',')]
    pub family: Option<Vec<Family>>,
    #[arg(long, value_delimiter = ',')]
    pub id: Option<Vec<i64>>,
    #[arg(long, value_delimiter = ',')]
    pub dns_name: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    pub domain: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    pub tenant: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    pub tenant_group: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    pub status: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    pub site: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    pub vlan: Option<Vec<u16>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Domains(pub Vec<Domain>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub addresses: Vec<Address>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Family {
    #[default]
    IPv4,
    IPv6,
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
    fn try_from(value: u8) -> Result<Self> {
        Ok(match value {
            4 => Self::IPv4,
            6 => Self::IPv6,
            _ => return Err(anyhow!("Unexpected integer")),
        })
    }
}

impl FromStr for Family {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "ipv4" => Self::IPv4,
            "ipv6" => Self::IPv6,
            _ => u8::from_str(s)?.try_into()?,
        })
    }
}

impl From<Vec<Address>> for Domains {
    fn from(mut addresses: Vec<Address>) -> Self {
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

macro_rules! check {
    ($a:expr, $b:expr, $($field:ident),+) => {
        $(check($a.$field.as_ref(), $b.$field.as_ref())) &&+
    };
}

impl PartialEq<Address> for AddressFilter {
    fn eq(&self, other: &Address) -> bool {
        check!(
            self,
            other,
            id,
            tenant,
            tenant_group,
            vlan,
            status,
            domain,
            site,
            family,
            dns_name
        )
    }
}

fn check<T>(a: Option<&Vec<T>>, b: Option<&T>) -> bool
where
    T: PartialEq,
{
    let Some(a) = a else { return true };
    let Some(b) = b else { return false };
    a.iter().any(|a| a == b)
}

impl Address {
    pub fn fetch_domain(&mut self, tld_extractor: &TldExtractor) -> Option<&String> {
        if self.domain.is_some() {
            return self.domain.as_ref();
        }

        let res = tld_extractor.extract((self.dns_name).as_ref()?).ok()?;
        if let (Some(domain), Some(suffix)) = (res.domain, res.suffix) {
            self.domain = Some(format!("{domain}.{suffix}"));
            return self.domain.as_ref();
        }

        None
    }
}
