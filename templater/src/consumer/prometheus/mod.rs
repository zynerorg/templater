use std::{
    fmt::Debug,
    fs::File,
    io::{Write, stdout},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Error, Result, anyhow};
use clap::{Args, ValueEnum, builder::PossibleValue};
use derive_more::From;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};

use super::Consumer;
use crate::data::{AddressMain, Family};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
#[serde(deny_unknown_fields)]
pub struct Prometheus {
    /// Output format
    #[arg(long, env("PROMETHEUS_FORMAT"), value_enum, default_value_t)]
    pub format: PrometheusFormat,
    // Output file
    #[arg(long, env("PROMETHEUS_OUTPUT"), value_name = "FILE")]
    pub output: Option<PathBuf>,
}

impl Consumer for Prometheus {
    fn consume(&self, addresses: Vec<AddressMain>) -> Result<()> {
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
    Float(f64),
    #[default]
    Null,
}

macro_rules! gen_label {
    ($ip:expr, $labels:expr, $($name:ident),+) => {
        ($(
            if let Some(value) = $ip.$name {
                $labels.push((format!("__meta_netbox_{}", stringify!($name)), value.into()));
            }
        ),+)
    };
}

impl TryFrom<AddressMain> for Data {
    type Error = Error;
    fn try_from(ip: AddressMain) -> Result<Self, Self::Error> {
        let mut labels: Vec<(String, Target)> = Vec::new();
        gen_label!(
            ip,
            labels,
            status,
            id,
            family,
            tenant,
            tenant_group,
            site,
            dns_name,
            role
        );

        if let Some(value) = ip.location {
            labels.push(("__meta_netbox_latitude".to_string(), value.latitude.into()));
            labels.push((
                "__meta_netbox_longitude".to_string(),
                value.longitude.into(),
            ));
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

impl From<Family> for Target {
    fn from(family: Family) -> Self {
        let family: u8 = family.into();
        i64::from(family).into()
    }
}

impl Data {
    fn push(config: &Prometheus, mut addresses: Vec<AddressMain>) -> Result<()> {
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
