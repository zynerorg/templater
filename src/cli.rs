use std::{path::PathBuf, str::FromStr};

use clap::{builder::PossibleValue, Args, Parser, Subcommand, ValueEnum};

use crate::netbox::data::ip_address::Status;

#[derive(Debug, Clone, Parser)]
#[command()]
pub struct Cli {
    /// Netbox API endpoint
    #[arg(short('e'), long, env)]
    pub netbox_endpoint: String,
    /// Netbox API token
    #[arg(short('n'), long, env)]
    pub netbox_token: String,

    /// Netbox ip address tenant
    #[arg(short, long, env, value_delimiter = ',')]
    pub tenant: Option<Vec<String>>,
    /// Netbox parent prefix site
    #[arg(short, long, env, value_delimiter = ',')]
    pub site: Option<Vec<String>>,
    /// Netbox parent prefix vlan
    #[arg(short, long, env, value_delimiter = ',')]
    pub vlan: Option<Vec<i64>>,
    /// Netbox ip address dns domain
    #[arg(short, long, env, value_delimiter = ',')]
    pub domain: Option<Vec<String>>,
    /// Netbox ip address status
    #[arg(short('r'), long, env, value_delimiter = ',')]
    pub status: Option<Vec<Status>>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Outputs Netbox ip addresses in Prometheus File SD format
    Prometheus(Prometheus),
    /// Outputs Netbox ip addresses in RFC1035 (BIND) format
    RFC1035(Rfc1035),
    /// Outputs raw json dump of all netbox ip addresses
    Dump(Dump),
}

#[derive(Debug, Clone, Args)]
pub struct Prometheus {
    /// Output format
    #[arg(short, long, env, value_enum, default_value_t)]
    pub format: PrometheusFormat,
}

#[derive(Default, Debug, Clone)]
pub enum PrometheusFormat {
    #[default]
    Yaml,
    Json,
}

impl FromStr for PrometheusFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "yaml" => PrometheusFormat::Yaml,
            "json" => PrometheusFormat::Json,
            _ => return Err("Unexpected format".into()),
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

#[derive(Debug, Clone, Args)]
pub struct Rfc1035 {
    /// Record TTL
    #[arg(short, long, env, default_value = "3600")]
    pub ttl: usize,
    /// Zonefile output directory
    #[arg(short('o'), long, env, value_name = "DIRECTORY")]
    pub directory: Option<PathBuf>,
    /// Disable SOA record generation
    #[arg(short, long, env)]
    pub disable_soa: bool,
    /// Zone Primary nameserver
    #[arg(short, long, env, required_unless_present = "disable_soa")]
    pub primary_nameserver: Option<String>,
    /// Zone Administrator email
    #[arg(short, long, env, required_unless_present = "disable_soa")]
    pub administrator_email: Option<String>,
    /// Zone refresh time
    #[arg(long, env, default_value = "86400")]
    pub refresh: usize,
    /// Zone retry time
    #[arg(long, env, default_value = "7200")]
    pub retry: usize,
    /// Zone expire time
    #[arg(long, env, default_value = "3600000")]
    pub expire: usize,
    /// Zone minimum TTL
    #[arg(long, env, default_value = "172800")]
    pub minimum: usize,
}

#[derive(Debug, Clone, Args)]
pub struct Dump {
    /// Split up ip addresses by domain
    #[arg(short, long, env)]
    pub by_domain: bool,
}
