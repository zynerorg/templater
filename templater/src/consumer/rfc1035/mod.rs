use std::{
    cmp::PartialEq,
    fs::{File, create_dir, read_dir, remove_file},
    io::{Write, stdout},
    net::IpAddr,
    path::{Path, PathBuf},
};

use chrono::Utc;
use clap::Args;
use derive_more::Display;
use ipnet::IpNet;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use zoneparser::{RRType, Record as ZoneRecord, ZoneParser};

use super::Consumer;
use crate::data::{AddressMain, Domains, ip_net_to_reverse_dns};

#[serde_inline_default]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
#[serde(deny_unknown_fields)]
pub struct Rfc1035 {
    /// Zonefile output directory
    #[arg(long, env("RFC1035_OUTPUT"))]
    pub output: Option<PathBuf>,
    /// Record TTL
    #[arg(long, env("RFC1035_TTL"), default_value = "3600")]
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
    #[arg(long, env("RFC1035_REFRESH"), default_value = "86400")]
    #[serde_inline_default(86_400)]
    pub refresh: usize,
    /// Zone retry time
    #[arg(long, env("RFC1035_RETRY"), default_value = "7200")]
    #[serde_inline_default(7_200)]
    pub retry: usize,
    // Zone expire time
    #[arg(long, env("RFC1035_EXPIRE"), default_value = "3600000")]
    #[serde_inline_default(3_600_000)]
    pub expire: usize,
    /// Zone minimum TTL
    #[arg(long, env("RFC1035_MINIMUM"), default_value = "172800")]
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
    fn consume(&self, addresses: Vec<AddressMain>) -> anyhow::Result<()> {
        Record::push(self, addresses)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Record {
    name: String,
    rtype: RType,
    rdata: String,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Display, Debug, Clone, PartialEq, Serialize, Deserialize)]
enum RType {
    A,
    AAAA,
    SOA,
    CNAME,
    PTR,
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

impl From<IpAddr> for RType {
    fn from(addr: IpAddr) -> Self {
        if addr.is_ipv4() { Self::A } else { Self::AAAA }
    }
}

impl Record {
    fn push(config: &Rfc1035, mut addresses: Vec<AddressMain>) -> anyhow::Result<()> {
        addresses.sort_by(|a, b| {
            a.dns_name.cmp(&b.dns_name).then(
                a.address
                    .map(|net| net.is_ipv6())
                    .cmp(&b.address.map(|net| net.is_ipv6())),
            )
        });
        // FIXME: Cross-domain CNAME
        let domains: Domains = addresses.clone().into();
        let reverse_domains = Domains::reverse_from_addresses(addresses);

        if let Some(directory) = &config.output {
            info!("Cleaning directory");
            let domains: Vec<&str> = domains
                .0
                .iter()
                .chain(reverse_domains.0.iter())
                .map(|d| d.name.as_str())
                .collect();
            Self::clean_directory(directory, &domains)?;
        }

        for domain in domains
            .0
            .into_iter()
            .zip([false].into_iter().cycle())
            .chain(
                reverse_domains
                    .0
                    .into_iter()
                    .zip([true].into_iter().cycle()),
            )
        {
            info!("Converting addresses to RFC1035 format");
            let reverse = domain.1;
            let domain = domain.0;
            let records = domain
                .addresses
                .into_iter()
                .filter_map(|addr| Self::from_address(addr, reverse))
                .flatten()
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

            info!("Writing records to zone {}", domain.name);
            debug!("Writing SOA record");
            Self::write_soa(&mut w, config, width, domain.name)?;
            debug!("Writing regular records");
            for record in records {
                writeln!(w, "{}", record.format(width))?;
            }
            info!("Finished writing records");
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

    fn from_address(ip: AddressMain, reverse: bool) -> Option<Vec<Self>> {
        let mut records = Vec::new();
        if let Some(address) = ip.address {
            let rtype = if reverse { RType::PTR } else { address.into() };

            let dns = ip.dns_name.as_ref()?.to_owned();
            let (name, rdata) = if reverse {
                let address = ip_net_to_reverse_dns(
                    &IpNet::new(address, ip.prefix?.prefix_len()).ok()?,
                    false,
                );
                (address, dns)
            } else {
                (dns, address.to_string())
            };

            records.push(Self { name, rtype, rdata });
        }

        if !reverse {
            if let Some(aliases) = ip.alias {
                for alias in aliases {
                    records.push(Self {
                        name: alias,
                        rtype: RType::CNAME,
                        rdata: ip.dns_name.as_ref()?.to_owned(),
                    });
                }
            }
        }

        Some(records)
    }

    fn format(&self, width: usize) -> String {
        let rtype_width = 5usize;
        format!(
            "{:width$}    IN    {:rtype_width$}    {}",
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

    fn clean_directory(directory: &Path, domains: &[&str]) -> anyhow::Result<()> {
        let dir = read_dir(directory)?;
        let files = dir
            .filter_map(Result::ok)
            .filter(|f| f.path().is_file())
            .filter(|f| {
                let path = f.path();
                let Some(name) = path.file_name() else {
                    return true;
                };
                let Some(name) = name.to_str() else {
                    return true;
                };

                !domains.contains(&name)
            });
        for file in files {
            debug!("Removing file {:?}", file.path().display());
            remove_file(file.path())?;
        }
        Ok(())
    }
}
