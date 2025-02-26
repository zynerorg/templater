use std::{
    cmp::PartialEq,
    fs::{create_dir, File},
    io::{stdout, Write},
    path::PathBuf,
};

use chrono::Utc;
use clap::Args;
use derive_more::Display;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use zoneparser::{RRType, Record as ZoneRecord, ZoneParser};

use super::Consumer;
use crate::data::{Address, Domains};

#[serde_inline_default]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
pub struct Rfc1035 {
    /// Zonefile output directory
    #[arg(long, env("RFC1035_OUTPUT"))]
    pub output: Option<PathBuf>,
    /// Record TTL
    #[arg(long, env("RFC1035_TTL"), default_value = "3_600")]
    #[serde_inline_default(3_600)]
    pub ttl: usize,
    /// Disable SOA record generation
    #[arg(long, env("RFC1035_DISABLE_SOA"))]
    #[serde(default)]
    pub disable_soa: bool,
    /// Zone Primary nameserver
    #[arg(
        long,
        env("RFC1035_PRIMARY_NAMESERVER"),
        required_unless_present = "disable_soa"
    )]
    pub primary_nameserver: Option<String>,
    /// Zone Administrator email
    #[arg(
        long,
        env("RFC1035_ADMINISTRATOR_EMAIL"),
        required_unless_present = "disable_soa"
    )]
    pub administrator_email: Option<String>,
    /// Zone refresh time
    #[arg(long, env("RFC1035_REFRESH"), default_value = "86_400")]
    #[serde_inline_default(86_400)]
    pub refresh: usize,
    /// Zone retry time
    #[arg(long, env("RFC1035_RETRY"), default_value = "7_200")]
    #[serde_inline_default(7_200)]
    pub retry: usize,
    // Zone expire time
    #[arg(long, env("RFC1035_EXPIRE"), default_value = "3_600_000")]
    #[serde_inline_default(3_600_000)]
    pub expire: usize,
    /// Zone minimum TTL
    #[arg(long, env("RFC1035_MINIMUM"), default_value = "172_800")]
    #[serde_inline_default(172_800)]
    pub minimum: usize,
}

impl Default for Rfc1035 {
    fn default() -> Self {
        Self {
            ttl: 3_600,
            output: None,
            disable_soa: false,
            primary_nameserver: None,
            administrator_email: None,
            refresh: 86_400,
            retry: 7_200,
            expire: 3_600_000,
            minimum: 172_800,
        }
    }
}

impl Consumer for Rfc1035 {
    fn consume(&self, addresses: Vec<Address>) -> anyhow::Result<()> {
        Record::push(self, addresses)
    }
}

struct Record {
    name: String,
    rtype: RType,
    rdata: String,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Display)]
enum RType {
    A,
    AAAA,
    SOA,
}

impl PartialEq<ZoneRecord> for Record {
    fn eq(&self, other: &ZoneRecord) -> bool {
        if other.data.len() != 1 {
            return false;
        }
        let other_data = &other.data[0].data;
        self.rtype == other.rrtype && self.name == other.name && self.rdata == *other_data
    }
}

impl PartialEq<RRType> for RType {
    fn eq(&self, other: &RRType) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Record {
    fn push(config: &Rfc1035, mut addresses: Vec<Address>) -> anyhow::Result<()> {
        addresses.sort_by(|a, b| {
            a.dns_name.cmp(&b.dns_name).then(
                a.address
                    .map(|net| net.addr().is_ipv6())
                    .cmp(&b.address.map(|net| net.addr().is_ipv6())),
            )
        });
        let domains: Domains = addresses.into();
        for domain in domains.0 {
            info!("Converting addresses to RFC1035 format");
            let records = domain
                .addresses
                .into_iter()
                .filter_map(Self::from_address)
                .collect::<Vec<Self>>();

            let mut w = if let Some(directory) = &config.output {
                let path = directory.join(&domain.name);
                if let Err(err) = create_dir(directory) {
                    if err.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(err.into());
                    }
                }
                debug!(
                    "Extracting zone {} from file {}",
                    domain.name,
                    path.display()
                );
                let zone = Self::extract_zone(&path).unwrap_or_default();

                if Self::compare_zones(&records, &zone) {
                    info!("Zones are equal, not modifying");
                    continue;
                }

                debug!("Opening file {} for writing", path.display());
                Box::new(File::create(&path)?) as Box<dyn Write>
            } else {
                debug!("Opening stdout for writing");
                Box::new(stdout()) as Box<dyn Write>
            };

            // let soa = Self::extract_soa(&zone).unwrap_or(Utc::now().timestamp() as usize);

            writeln!(w, "$ORIGIN .")?;
            writeln!(w, "$TTL {}", config.ttl)?;

            let width = records.iter().fold(usize::MIN, |a, b| a.max(b.name.len()));

            debug!("Writing SOA record to zone {}", domain.name);
            Self::write_soa(&mut w, config, width, domain.name.clone())?;

            info!("Writing records to zone {}", domain.name);
            for record in records {
                writeln!(w, "{}", record.format(width))?;
            }
            info!("Finished writing domain {}", domain.name);
        }

        Ok(())
    }

    fn write_soa(
        writer: &mut Box<dyn Write>,
        config: &Rfc1035,
        width: usize,
        domain_name: String,
    ) -> anyhow::Result<()> {
        if let (Some(ns), Some(email)) = (&config.primary_nameserver, &config.administrator_email) {
            let soa = Record {
                name: domain_name,
                rtype: RType::SOA,
                rdata: format!(
                    "{} {} {} {} {} {} {}",
                    ns,
                    email,
                    Utc::now().timestamp(),
                    config.refresh,
                    config.retry,
                    config.expire,
                    config.minimum
                ),
            };
            writeln!(writer, "{}", soa.format(width))?;
        }

        Ok(())
    }

    fn from_address(ip: Address) -> Option<Self> {
        let rtype = if ip.address?.addr().is_ipv4() {
            RType::A
        } else {
            RType::AAAA
        };
        Some(Self {
            name: ip.dns_name?,
            rtype,
            rdata: ip.address?.addr().to_string(),
        })
    }

    fn format(&self, width: usize) -> String {
        let abc = 4usize;
        format!(
            "{:width$}    IN    {:abc$}    {}",
            self.name,
            self.rtype.to_string(),
            self.rdata
        )
    }

    fn extract_zone(path: &PathBuf) -> anyhow::Result<Vec<ZoneRecord>> {
        Ok(ZoneParser::new(&File::open(path)?, ".").collect())
    }

    // fn extract_soa(zone: &[Record]) -> anyhow::Result<usize> {
    // let soa_record = zone
    // .iter()
    // .find(|r| r.rrtype == RRType::SOA)
    // .ok_or(anyhow!("Cannot find SOA record in zonefile"))?;
    // let soa_data = soa_record
    // .data
    // .get(2)
    // .ok_or(anyhow!("Cannot find serial in SOA record"))?;
    // let soa: usize = soa_data.data.parse()?;
    // Ok(soa)
    // }

    fn compare_zones(a: &[Record], b: &[ZoneRecord]) -> bool {
        let b: Vec<&ZoneRecord> = b.iter().filter(|r| r.rrtype != RRType::SOA).collect();
        if b.is_empty() || a.len() != b.len() {
            return false;
        }

        a.iter().zip(b.iter()).filter(|(a, b)| **a == ***b).count() == a.len()
    }
}
