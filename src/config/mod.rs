use std::{
    fs::File,
    ops::{Add, AddAssign},
    path::PathBuf,
};

use anyhow::{Error, Result};
use clap::Subcommand;
use serde_derive::{Deserialize, Serialize};
use tldextract::TldOption;

use crate::{
    consumer::{prometheus::Prometheus, rfc1035::Rfc1035, Consumer as ConsumerTrait},
    data::{Address, AddressFilter},
    provider::{netbox::Netbox, yaml::Yaml, Provider as ProviderTrait},
};

pub mod cli;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    providers: Vec<Provider>,
    consumers: Vec<Consumer>,
    filter: Option<AddressFilter>,
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
        self.filter = rhs.filter;
    }
}

impl Config {
    pub fn parse(path: PathBuf) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_yaml::from_reader(file)?)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Provider {
    config: ProviderConfig,
    filter: Option<AddressFilter>,
}

impl ProviderTrait for Provider {
    fn provide(&self) -> Result<Vec<Address>> {
        match &self.config {
            ProviderConfig::Netbox(n) => n.provide(),
            ProviderConfig::Yaml(n) => n.provide(),
            ProviderConfig::Null => Ok(Vec::new()),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Subcommand)]
#[serde(tag = "type")]
enum ProviderConfig {
    Netbox(Netbox),
    Yaml(Yaml),
    #[default]
    Null,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Consumer {
    config: ConsumerConfig,
    filter: Option<AddressFilter>,
}

impl ConsumerTrait for Consumer {
    fn consume(&self, addresses: Vec<Address>) -> Result<()> {
        match &self.config {
            ConsumerConfig::Prometheus(n) => n.consume(addresses),
            ConsumerConfig::Rfc1035(n) => n.consume(addresses),
            ConsumerConfig::Null => Ok(()),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Subcommand)]
#[serde(tag = "type")]
enum ConsumerConfig {
    Rfc1035(Rfc1035),
    Prometheus(Prometheus),
    #[default]
    Null,
}

impl Config {
    pub fn execute(&self) -> Result<()> {
        let tld_extractor = TldOption::default()
            .naive_mode(
                true, // Required because it does not like the internal TLD, will break domains like co.uk
            )
            .build();
        let mut addresses: Vec<Address> = self
            .providers
            .iter()
            .map(|provider| {
                provider.provide().map(|addresses| {
                    let mut addresses: Vec<Address> = addresses
                        .into_iter()
                        .filter(|address| {
                            provider
                                .filter
                                .as_ref()
                                .is_none_or(|filter| *filter == *address)
                        })
                        .collect();
                    for addr in &mut addresses {
                        addr.fetch_domain(&tld_extractor);
                    }
                    addresses
                })
            })
            .try_fold(Vec::new(), |mut a, b| {
                a.extend(b?);
                Ok::<Vec<Address>, Error>(a)
            })?;

        if let Some(filter) = &self.filter {
            addresses.retain(|address| filter == address);
        }

        for consumer in &self.consumers {
            let mut addresses = addresses.clone();
            if let Some(filter) = &consumer.filter {
                addresses.retain(|address| filter == address);
            }

            consumer.consume(addresses)?;
        }

        Ok(())
    }
}
