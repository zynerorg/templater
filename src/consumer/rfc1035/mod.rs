use std::{
    fmt::Display,
    fs::{create_dir, File},
    io::{stdout, Write},
};

use chrono::Utc;
use log::info;

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

impl Rfc1035 {
    pub fn push(addresses: Vec<IpAddress>, cmd: &Cmd) -> anyhow::Result<()> {
        let domains: Domains = addresses.into();
        for domain in domains.0 {
            let mut w = if let Some(directory) = &cmd.output {
                if let Err(err) = create_dir(directory) {
                    if err.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(err.into());
                    }
                }
                let file = directory.join(&domain.name);
                info!("Writing zone {} to file {}", domain.name, file.display());
                Box::new(File::create(file)?) as Box<dyn Write>
            } else {
                info!("Writing zone {} to stdout", domain.name);
                Box::new(stdout()) as Box<dyn Write>
            };

            info!("Converting addresses to RFC1035 format");
            let records = domain
                .addresses
                .into_iter()
                .filter_map(Self::from_ipaddress)
                .collect::<Vec<Self>>();

            writeln!(w, "$ORIGIN .")?;
            writeln!(w, "$TTL {}", cmd.ttl)?;

            let width = records.iter().fold(usize::MIN, |a, b| a.max(b.name.len()));

            if let (Some(ns), Some(email)) = (&cmd.primary_nameserver, &cmd.administrator_email) {
                let soa = Rfc1035 {
                    name: domain.name.clone(),
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
                writeln!(w, "{}", soa.format(width))?;
            }

            for record in records {
                writeln!(w, "{}", record.format(width))?;
            }
            info!("Finished writing domain {}", domain.name);
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
