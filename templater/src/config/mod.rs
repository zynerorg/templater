use std::{
    fs::File,
    ops::{Add, AddAssign},
    path::PathBuf,
};

use anyhow::Result;
use clap::Subcommand;
use serde_derive::{Deserialize, Serialize};
use tldextract::TldOption;

use crate::{
    consumer::{
        Consumer as ConsumerTrait, cloudflare::Cloudflare, prometheus::Prometheus, rfc1035::Rfc1035,
    },
    data::{AddressMain, VecAddressFilter},
    provider::{Provider as ProviderTrait, netbox::Netbox, yaml::Yaml},
};

pub mod cli;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    providers: Vec<Provider>,
    consumers: Vec<Consumer>,
    filters: Option<VecAddressFilter>,
}

impl Add for Config {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut config = self.clone();
        config.add_assign(rhs);
        config
    }
}

impl AddAssign for Config {
    fn add_assign(&mut self, rhs: Self) {
        self.providers.extend(rhs.providers);
        self.consumers.extend(rhs.consumers);
        self.filters = rhs.filters;
    }
}

impl Config {
    pub fn parse(path: PathBuf) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_yaml::from_reader(file)?)
    }

    pub fn execute(self) -> Result<()> {
        let addresses: Result<Vec<Vec<AddressMain>>> =
            self.providers.into_iter().map(Provider::provide).collect();
        let mut addresses: Vec<AddressMain> = addresses?.into_iter().flatten().collect();

        if let Some(filters) = &self.filters {
            addresses.retain(|address| filters.0.iter().any(|f| f == address));
        }

        for consumer in self.consumers {
            let mut addresses = addresses.clone();
            if let Some(filters) = &consumer.filters {
                addresses.retain(|address| filters.0.iter().any(|f| f == address));
            }

            consumer.consume(addresses)?;
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Provider {
    config: ProviderConfig,
    filters: Option<VecAddressFilter>,
}

impl ProviderTrait for Provider {
    fn provide(self) -> Result<Vec<AddressMain>> {
        let tld_extractor = TldOption::default()
            .naive_mode(
                true, // Required because it does not like the internal TLD, will break domains like co.uk
            )
            .build();

        match self.config {
            ProviderConfig::Netbox(n) => n.provide(),
            ProviderConfig::Yaml(n) => n.provide(),
            ProviderConfig::Null => Ok(Vec::new()),
        }
        .map(|addresses| {
            let mut addresses: Vec<AddressMain> = addresses
                .into_iter()
                .filter(|address| {
                    self.filters
                        .as_ref()
                        .is_none_or(|filters| filters.0.iter().any(|f| f == address))
                })
                .collect();
            for address in &mut addresses {
                address.fetch_domain(&tld_extractor);
            }
            addresses
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Subcommand)]
#[serde(deny_unknown_fields, tag = "type")]
enum ProviderConfig {
    Netbox(Netbox),
    Yaml(Yaml),
    #[default]
    Null,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Consumer {
    config: ConsumerConfig,
    filters: Option<VecAddressFilter>,
}

impl ConsumerTrait for Consumer {
    fn consume(&self, addresses: Vec<AddressMain>) -> Result<()> {
        match &self.config {
            ConsumerConfig::Prometheus(n) => n.consume(addresses),
            ConsumerConfig::Rfc1035(n) => n.consume(addresses),
            ConsumerConfig::Cloudflare(n) => n.consume(addresses),
            ConsumerConfig::Null => Ok(()),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Subcommand)]
#[serde(deny_unknown_fields, tag = "type")]
enum ConsumerConfig {
    Rfc1035(Rfc1035),
    Prometheus(Prometheus),
    Cloudflare(Cloudflare),
    #[default]
    Null,
}
