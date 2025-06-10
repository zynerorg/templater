use std::{net::IpAddr, str::FromStr};

use anyhow::{Error, anyhow};
use clap::Args;
use serde_derive::{Deserialize, Serialize};

use crate::{data::AddressMain, provider::Provider};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
#[serde(deny_unknown_fields)]
pub struct Yaml {
    /// List of addresses
    #[arg(env("YAML_ADDRESS"), value_delimiter = ',')]
    data: Vec<Data>,
}

impl Provider for Yaml {
    fn provide(self) -> anyhow::Result<Vec<AddressMain>> {
        Ok(self.data.into_iter().map(|s| s.0).collect())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Data(AddressMain);

impl FromStr for Data {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.split('=');
        let address: IpAddr = data
            .next()
            .ok_or(anyhow!("No address in parameter"))?
            .parse()?;
        let dns_name = data.collect::<Vec<&str>>().join("=");

        Ok(Self(AddressMain {
            address: Some(address),
            dns_name: Some(dns_name),
            ..Default::default()
        }))
    }
}
