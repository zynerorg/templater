use std::{net::IpAddr, str::FromStr};

use anyhow::{anyhow, Error};
use clap::Args;
use serde_derive::{Deserialize, Serialize};

use crate::{data::Address, provider::Provider};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
pub struct Yaml {
    /// List of addresses
    #[arg(env("YAML_ADDRESS"), value_delimiter = ',')]
    data: Vec<Data>,
}

impl Provider for Yaml {
    fn provide(&self) -> anyhow::Result<Vec<Address>> {
        Ok(self.data.clone().into_iter().map(|s| s.0).collect())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Data(Address);

impl FromStr for Data {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.split('=');
        let address: IpAddr = data
            .next()
            .ok_or(anyhow!("No address in parameter"))?
            .parse()?;
        let dns_name = data.collect::<Vec<&str>>().join("=");

        Ok(Self(Address {
            address: Some(address),
            dns_name: Some(dns_name),
            ..Default::default()
        }))
    }
}
