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
    family: Family,
    rdata: String,
}

impl Rfc1035 {
    pub fn push(addresses: Vec<IpAddress>, cmd: &Cmd) -> anyhow::Result<()> {
        let domains: Domains = addresses.into();
        for domain in domains.0 {
            let mut w = if let Some(directory) = &cmd.directory {
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

            if let (Some(ns), Some(email)) = (&cmd.primary_nameserver, &cmd.administrator_email) {
                writeln!(
                    w,
                    "{}\tIN\tSOA\t{} {} {} {} {} {} {}",
                    domain.name,
                    ns,
                    email,
                    Utc::now().timestamp(),
                    cmd.refresh,
                    cmd.retry,
                    cmd.expire,
                    cmd.minimum
                )?;
            }
            for record in records {
                writeln!(w, "{record}")?;
            }
            info!("Finished writing domain {}", domain.name);
        }

        Ok(())
    }

    fn from_ipaddress(address: IpAddress) -> Option<Self> {
        Some(Self {
            name: address.dns_name?,
            family: address.family,
            rdata: address.address.addr().to_string(),
        })
    }
}

impl Display for Rfc1035 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_ = match self.family {
            Family::IPv4 => "A",
            Family::IPv6 => "AAAA",
        };
        write!(f, "{}\tIN\t{}\t{}", self.name, type_, self.rdata)
    }
}
