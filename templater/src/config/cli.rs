use std::{env::args, path::PathBuf};

use anyhow::{Error, Result, anyhow};
use clap::{Args, FromArgMatches, Parser, Subcommand};

use super::{Config as ConfigI, Consumer, ConsumerConfig, Provider, ProviderConfig};
use crate::data::VecAddressFilter;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

impl Cli {
    pub fn parse_full() -> Result<Vec<Self>> {
        let mut args = args();
        let name = args.next().ok_or(anyhow!("Command name not found"))?;
        let mut cmd = Vec::new();
        let mut sub = vec![name.clone()];
        for i in args {
            if i == "--" {
                cmd.push(sub);
                sub = vec![name.clone()];
                continue;
            }
            sub.push(i);
        }
        cmd.push(sub);

        Ok(cmd.into_iter().map(Self::parse_from).collect())
    }

    pub fn config(cli: Vec<Self>) -> Result<Vec<ConfigI>> {
        let mut cli = cli.into_iter().peekable();
        let mut cmds = Vec::new();
        while let Some(c) = cli.next() {
            let mut config: ConfigI = c.try_into()?;
            if let Some(peek) = cli.peek()
                && matches!(peek.mode, Mode::Filter(_))
                && let Mode::Filter(filters) = cli.next().unwrap().mode
            {
                let mut filters = Some(filters);
                if let Some(f) = config.providers.get_mut(0) {
                    f.filters = filters.take();
                }
                if let Some(f) = config.consumers.get_mut(0) {
                    f.filters = filters.take();
                }
            }
            cmds.push(config);
        }
        Ok(cmds)
    }
}

#[derive(Debug, Clone, Subcommand)]
enum Mode {
    Config(Config),
    Provider(Shim<ProviderConfig>),
    Consumer(Shim<ConsumerConfig>),
    Filter(VecAddressFilter),
    GlobalFilter(VecAddressFilter),
}

#[derive(Debug, Clone, Args)]
struct Config {
    /// YAML config path
    path: PathBuf,
}

#[derive(Debug, Clone, Args)]
struct Shim<T: FromArgMatches + Subcommand> {
    #[command(subcommand)]
    a: T,
}

impl TryFrom<Cli> for ConfigI {
    type Error = Error;
    fn try_from(cli: Cli) -> Result<Self> {
        let mut config = Self::default();
        match cli.mode {
            Mode::Config(conf) => {
                config = Self::parse(conf.path)?;
            }
            Mode::Provider(provider) => config.providers.push(Provider {
                config: provider.a,
                ..Default::default()
            }),
            Mode::Consumer(consumer) => config.consumers.push(Consumer {
                config: consumer.a,
                ..Default::default()
            }),
            Mode::GlobalFilter(filter) => config.filters = Some(filter),
            Mode::Filter(_) => {}
        }

        Ok(config)
    }
}
