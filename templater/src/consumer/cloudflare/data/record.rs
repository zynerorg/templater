use std::ops::Sub;

use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_more::From;
use ipnet::IpNet;
use serde_derive::{Deserialize, Serialize};

use super::super::Cloudflare;
use crate::data::{Domain, ip_net_to_reverse_dns};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, From)]
pub struct Records(pub Vec<Record>);

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Record {
    A(Generic),
    AAAA(Generic),
    CNAME(Generic),
    PTR(Generic),
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde_with::skip_serializing_none]
pub struct Generic {
    pub id: Option<String>,
    pub name: String,
    pub content: String,
    pub proxied: bool,
    pub ttl: usize,
    pub comment: Option<String>,
    pub created_on: Option<DateTime<Utc>>,
    pub modified_on: Option<DateTime<Utc>>,
}

impl PartialEq for Generic {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.content == other.content
            && self.proxied == other.proxied
            && self.ttl == other.ttl
            && self.comment == other.comment
    }
}

impl Records {
    pub fn from_domain(config: &Cloudflare, domain: Domain) -> Result<Self> {
        let mut records = Vec::new();
        for address in domain.addresses {
            let Some(dns_name) = address.dns_name else {
                continue;
            };
            let Some(ip) = address.address else {
                continue;
            };

            let (name, content) = if domain.reverse {
                let Some(prefix) = address.prefix else {
                    continue;
                };
                let rdns = ip_net_to_reverse_dns(&IpNet::new(ip, prefix.prefix_len())?, false);
                (rdns, dns_name)
            } else {
                (dns_name, ip.to_string())
            };

            let record = Generic {
                name: name.clone(),
                content,
                ttl: config.ttl,
                ..Default::default()
            };

            let record = if domain.reverse {
                Record::PTR(record)
            } else {
                if let Some(alias) = address.alias {
                    for alias in alias {
                        records.push(Record::CNAME(Generic {
                            name: alias,
                            content: name.clone(),
                            ttl: config.ttl,
                            ..Default::default()
                        }));
                    }
                }

                if ip.is_ipv4() {
                    Record::A(record)
                } else {
                    Record::AAAA(record)
                }
            };

            records.push(record);
        }
        Ok(records.into())
    }

    pub fn filter_other(self) -> Self {
        Self(self.0.into_iter().filter(|r| !matches!(r, Record::Other)).collect())
    }
}

impl Sub<&Self> for Records {
    type Output = Self;
    fn sub(self, other: &Self) -> Self::Output {
        Records(
            self.0
                .into_iter()
                .filter(|record| other.0.iter().all(|other| record != other))
                .collect(),
        )
    }
}

impl Record {
    pub fn generic(&self) -> Option<&Generic> {
        Some(match self {
            Self::A(record) | Self::AAAA(record) | Self::CNAME(record) | Self::PTR(record) => {
                record
            }
            Self::Other => return None,
        })
    }
}
