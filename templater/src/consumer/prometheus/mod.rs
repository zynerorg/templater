use std::{
    fmt::Debug,
    fs::File,
    io::{stdout, Write},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{anyhow, Error, Result};
use clap::{builder::PossibleValue, Args, ValueEnum};
use derive_more::From;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};

use super::Consumer;
use crate::data::Address;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
pub struct Prometheus {
    /// Output format
    #[arg(long, env("PROMETHEUS_FORMAT"), value_enum, default_value_t)]
    pub format: PrometheusFormat,
    // Output file
    #[arg(long, env("PROMETHEUS_OUTPUT"), value_name = "FILE")]
    pub output: Option<PathBuf>,
}

impl Consumer for Prometheus {
    fn consume(&self, addresses: Vec<Address>) -> Result<()> {
        Data::push(self, addresses)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrometheusFormat {
    Yaml,
    #[default]
    Json,
}

impl FromStr for PrometheusFormat {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "yaml" => PrometheusFormat::Yaml,
            "json" => PrometheusFormat::Json,
            _ => return Err(anyhow!("Unexpected format")),
        })
    }
}

impl ValueEnum for PrometheusFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[PrometheusFormat::Yaml, PrometheusFormat::Json]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(match self {
            PrometheusFormat::Yaml => "yaml",
            PrometheusFormat::Json => "json",
        }))
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Data {
    targets: Vec<String>,
    #[serde(with = "tuple_vec_map")]
    labels: Vec<(String, Target)>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, From)]
#[serde(untagged)]
enum Target {
    String(String),
    Integer(i64),
    #[default]
    Null,
}

impl TryFrom<Address> for Data {
    type Error = Error;
    fn try_from(ip: Address) -> Result<Self, Self::Error> {
        let mut labels: Vec<(String, Target)> = Vec::new();
        if let Some(status) = ip.status {
            labels.push(("__meta_netbox_status".into(), status.into()));
        }
        if let Some(id) = ip.id {
            labels.push(("__meta_netbox_id".into(), id.into()));
        }
        if let Some(family) = ip.family {
            let family: u8 = family.into();
            labels.push(("__meta_netbox_family".into(), i64::from(family).into()));
        }
        if let Some(tenant) = ip.tenant {
            labels.push(("__meta_netbox_tenant".into(), tenant.into()));
        }
        if let Some(group) = ip.tenant_group {
            labels.push(("__meta_netbox_tenant_group".into(), group.into()));
        }
        if let Some(site) = ip.site {
            labels.push(("__meta_netbox_site".into(), site.into()));
        }
        if let Some(dns_name) = ip.dns_name {
            labels.push(("__meta_netbox_dns_name".into(), dns_name.into()));
        }

        Ok(Self {
            targets: vec![
                ip.address
                    .ok_or(anyhow!("No IP address found"))?
                    .to_string(),
            ],
            labels,
        })
    }
}

impl Data {
    fn push(config: &Prometheus, mut addresses: Vec<Address>) -> Result<()> {
        info!("Converting addresses to Prometheus File SD format");
        addresses.sort_by(|a, b| {
            a.address
                .map(|net| net.is_ipv6())
                .cmp(&b.address.map(|net| net.is_ipv6()))
                .then(a.address.cmp(&b.address))
        });
        let configs = addresses
            .into_iter()
            .filter_map(|address| Self::try_from(address).ok())
            .collect::<Vec<Self>>();

        info!("Printing in Prometheus File SD format");
        let mut w = if let Some(path) = &config.output {
            debug!("Opening file {} for writing", path.display());
            Box::new(File::create(path)?) as Box<dyn Write>
        } else {
            debug!("Opening stdout for writing");
            Box::new(stdout()) as Box<dyn Write>
        };

        writeln!(
            w,
            "{}",
            match config.format {
                PrometheusFormat::Yaml => serde_yaml::to_string(&configs)?,
                PrometheusFormat::Json => serde_json::to_string_pretty(&configs)?,
            }
        )?;

        Ok(())
    }
}
