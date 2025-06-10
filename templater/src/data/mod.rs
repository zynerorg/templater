use std::{fmt::Debug, net::IpAddr, num::ParseFloatError, str::FromStr};

use anyhow::{Error, anyhow};
use clap::{ArgMatches, Args, Command, FromArgMatches, error::ErrorKind};
use derive_more::From;
use ipnet::IpNet;
use serde_derive::{Deserialize, Serialize};
use templater_macro::Filter;
use tldextract::TldExtractor;

#[derive(Filter)]
#[allow(dead_code)]
pub struct Address {
    pub address: IpAddr,
    pub family: Family,
    pub id: i64,
    pub dns_name: String,
    pub domain: String,
    pub tenant: String,
    pub tenant_group: String,
    pub status: String,
    pub site: String,
    pub vlan: u16,
    #[filter(skip)]
    pub alias: Vec<String>,
    #[filter(vec)]
    pub tags: Vec<String>,
    pub location: Location,
    pub role: String,
    pub prefix: IpNet,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Domains(pub Vec<Domain>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub addresses: Vec<AddressMain>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
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
    fn try_from(value: u8) -> anyhow::Result<Self> {
        Ok(match value {
            4 => Self::IPv4,
            6 => Self::IPv6,
            _ => return Err(anyhow!("Unexpected integer")),
        })
    }
}

impl FromStr for Family {
    type Err = Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        Ok(match s {
            "ipv4" => Self::IPv4,
            "ipv6" => Self::IPv6,
            _ => u8::from_str(s)?.try_into()?,
        })
    }
}

impl FromStr for Location {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        let coords: Result<Vec<f64>, ParseFloatError> = s.split(',').map(str::parse).collect();
        let mut coords = coords?;
        Ok(Self {
            longitude: coords.pop().ok_or(anyhow!("Failed to parse longitude"))?,
            latitude: coords.pop().ok_or(anyhow!("Failed to parse latitude"))?,
        })
    }
}

impl From<Vec<AddressMain>> for Domains {
    fn from(mut addresses: Vec<AddressMain>) -> Self {
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

impl Domains {
    pub fn reverse_from_addresses(mut addresses: Vec<AddressMain>) -> Self {
        for address in &mut addresses {
            address.domain = address
                .prefix
                .map(|prefix| ip_net_to_reverse_dns(&prefix, true));
        }

        addresses.into()
    }
}

pub fn ip_net_to_reverse_dns(addr: &IpNet, strip: bool) -> String {
    let segments = match addr.addr() {
        IpAddr::V4(ip) => ip.octets().to_vec(),
        IpAddr::V6(ip) => {
            let mut nibbles = Vec::new();
            let bits = ip.to_bits().to_be_bytes();
            for byte in bits {
                nibbles.push(byte >> 4);
                nibbles.push(byte & 0xF);
            }
            nibbles
        }
    };

    let is_ipv4 = addr.addr().is_ipv4();
    let family = if is_ipv4 {
        (32, 8, ".in-addr.arpa")
    } else {
        (128, 4, ".ip6.arpa")
    };
    let skip_distance = if strip {
        (family.0 - addr.prefix_len() as usize).div_ceil(family.1)
    } else {
        0
    };

    let addr = segments
        .iter()
        .rev()
        .skip(skip_distance)
        .map(|segment| {
            if is_ipv4 {
                segment.to_string()
            } else {
                format!("{segment:1x}")
            }
        })
        .collect::<Vec<String>>()
        .join(".");
    format!("{}{}", addr, family.2)
}

fn check<T>(a: Option<&Vec<T>>, b: Option<&T>, default: bool) -> bool
where
    T: PartialEq,
{
    let Some(a) = a else {
        return default;
    };
    let Some(b) = b else {
        return !default;
    };
    a.iter().any(|a| a == b)
}

fn check_vec<T>(a: Option<&Vec<T>>, b: Option<&Vec<T>>, default: bool) -> bool
where
    T: PartialEq + Debug,
{
    let Some(a) = a else {
        return default;
    };
    let Some(b) = b else {
        return !default;
    };
    a.iter().any(|a| b.iter().any(|b| a == b))
}

impl AddressMain {
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, From)]
#[serde(deny_unknown_fields)]
pub struct VecAddressFilter(pub Vec<AddressFilter>);

impl FromArgMatches for VecAddressFilter {
    fn from_arg_matches(matches: &ArgMatches) -> std::result::Result<Self, clap::Error> {
        Ok(vec![AddressFilter::from_arg_matches(matches)?].into())
    }
    fn update_from_arg_matches(
        &mut self,
        matches: &ArgMatches,
    ) -> std::result::Result<(), clap::Error> {
        self.0
            .get_mut(0)
            .ok_or(clap::Error::new(ErrorKind::InvalidValue))?
            .update_from_arg_matches(matches)
    }
}

impl Args for VecAddressFilter {
    fn augment_args(cmd: Command) -> Command {
        AddressFilter::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        AddressFilter::augment_args_for_update(cmd)
    }
}
