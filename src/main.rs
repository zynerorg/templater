#![warn(clippy::pedantic)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::unused_self)]

mod cli;
mod consumer;
mod netbox;

use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use log::info;

use crate::{
    cli::{Cli, Commands},
    consumer::{prometheus::Prometheus, rfc1035::Rfc1035},
    netbox::{
        data::ip_address::{Domains, IpAddress},
        Netbox,
    },
};

fn main() -> Result<()> {
    dotenv().ok();
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    let cli = Cli::parse();

    let netbox = Netbox::new(cli.netbox_endpoint.clone(), &cli.netbox_token)?;
    let addresses: Vec<IpAddress> = netbox.fetch_addresses()?;
    let addresses = netbox.filter(&cli, addresses);

    match cli.command {
        Commands::Prometheus(cmd) => Prometheus::push(addresses, &cmd)?,
        Commands::RFC1035(cmd) => Rfc1035::push(addresses, &cmd)?,
        Commands::Dump(cmd) => {
            info!("Dumping addresses");
            if cmd.by_domain {
                let domains = Domains::from(addresses);
                println!("{}", serde_json::to_string_pretty(&domains)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&addresses)?);
            }
        }
    }

    Ok(())
}
