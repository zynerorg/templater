use std::{
    cmp::PartialEq,
    fmt::Display,
    fs::{create_dir, File},
    io::{stdout, Write},
    path::PathBuf,
};

use chrono::Utc;
use log::{debug, info};
use zoneparser::{RRType, Record, ZoneParser};

use crate::{
    cli::Rfc1035 as Cmd,
    netbox::data::{
        common::Family,
        ip_address::{Domains, IpAddress},
    },
};

pub struct Rfc1035 {
    name: String,
    rtype: RType,
    rdata: String,
}

#[allow(clippy::upper_case_acronyms)]
enum RType {
    A,
    AAAA,
    SOA,
}

impl PartialEq<Record> for Rfc1035 {
    fn eq(&self, other: &Record) -> bool {
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

impl Rfc1035 {
    pub fn push(addresses: Vec<IpAddress>, cmd: &Cmd) -> anyhow::Result<()> {
        let domains: Domains = addresses.into();
        for domain in domains.0 {
            info!("Converting addresses to RFC1035 format");
            let records = domain
                .addresses
                .into_iter()
                .filter_map(Self::from_ipaddress)
                .collect::<Vec<Self>>();

            let mut w = if let Some(directory) = &cmd.output {
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
            writeln!(w, "$TTL {}", cmd.ttl)?;

            let width = records.iter().fold(usize::MIN, |a, b| a.max(b.name.len()));

            debug!("Writing SOA record to zone {}", domain.name);
            Self::write_soa(&mut w, cmd, width, domain.name.clone())?;

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
        cmd: &Cmd,
        width: usize,
        domain_name: String,
    ) -> anyhow::Result<()> {
        if let (Some(ns), Some(email)) = (&cmd.primary_nameserver, &cmd.administrator_email) {
            let soa = Rfc1035 {
                name: domain_name,
                rtype: RType::SOA,
                rdata: format!(
                    "{} {} {} {} {} {} {}",
                    ns,
                    email,
                    Utc::now().timestamp(),
                    cmd.refresh,
                    cmd.retry,
                    cmd.expire,
                    cmd.minimum
                ),
            };
            writeln!(writer, "{}", soa.format(width))?;
        }

        Ok(())
    }

    fn from_ipaddress(address: IpAddress) -> Option<Self> {
        Some(Self {
            name: address.dns_name?,
            rtype: address.family.into(),
            rdata: address.address.addr().to_string(),
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

    fn extract_zone(path: &PathBuf) -> anyhow::Result<Vec<Record>> {
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

    fn compare_zones(a: &[Rfc1035], b: &[Record]) -> bool {
        let b: Vec<&Record> = b.iter().filter(|r| r.rrtype != RRType::SOA).collect();
        if b.is_empty() || a.len() != b.len() {
            return false;
        }

        a.iter().zip(b.iter()).filter(|(a, b)| **a == ***b).count() == a.len()
    }
}

impl Display for RType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::A => "A",
                Self::AAAA => "AAAA",
                Self::SOA => "SOA",
            }
        )
    }
}

impl From<Family> for RType {
    fn from(family: Family) -> Self {
        match family {
            Family::IPv4 => Self::A,
            Family::IPv6 => Self::AAAA,
        }
    }
}
