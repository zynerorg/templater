#![warn(clippy::pedantic)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::unused_self)]

mod config;
mod consumer;
mod data;
mod provider;

use anyhow::Result;
use dotenvy::dotenv;
use log::debug;

use crate::config::{cli::Cli, Config};

fn main() -> Result<()> {
    dotenv().ok();
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    debug!("Parsing cli");
    let cli = Cli::parse_full()?;
    debug!("Converting cli to config");
    let config = Cli::config(cli)?
        .into_iter()
        .fold(Config::default(), |a, b| a + b);
    debug!("Executing");
    config.execute()
}
