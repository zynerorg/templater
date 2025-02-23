use std::{
    fmt::Debug,
    fs::File,
    io::{stdout, Write},
};

use anyhow::Result;
use derive_more::From;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};

use crate::{
    cli::{Prometheus as Cmd, PrometheusFormat},
    netbox::data::{ip_address::IpAddress, prefix::Scope},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prometheus {
    targets: Vec<String>,
    #[serde(with = "tuple_vec_map")]
    labels: Vec<(String, Data)>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, From)]
#[serde(untagged)]
enum Data {
    String(String),
    Integer(i64),
    #[default]
    Null,
}

impl TryFrom<IpAddress> for Prometheus {
    type Error = ();
    fn try_from(value: IpAddress) -> Result<Self, Self::Error> {
        let mut labels: Vec<(String, Data)> = Vec::new();
        labels.push((
            "__meta_netbox_status".into(),
            value.status.to_string().into(),
        ));
        labels.push(("__meta_netbox_id".into(), value.id.into()));
        let family: u8 = value.family.into();
        labels.push(("__meta_netbox_family".into(), i64::from(family).into()));
        if let Some(tenant) = value.full_tenant {
            labels.push(("__meta_netbox_tenant".into(), tenant.slug.into()));
            if let Some(group) = tenant.group {
                labels.push(("__meta_netbox_tenant_group".into(), group.slug.into()));
            }
        }
        if let Some(Scope::Site(site)) = value.scope {
            labels.push(("__meta_netbox_site".into(), site.slug.to_string().into()));
        }
        if let Some(dns_name) = value.dns_name {
            labels.push(("__meta_netbox_dns_name".into(), dns_name.to_string().into()));
        }

        Ok(Self {
            targets: vec![value.address.addr().to_string()],
            labels,
        })
    }
}

impl Prometheus {
    pub fn push(addresses: Vec<IpAddress>, cmd: &Cmd) -> Result<()> {
        info!("Converting addresses to Prometheus File SD format");
        let configs = addresses
            .into_iter()
            .filter_map(|address| Self::try_from(address).ok())
            .collect::<Vec<Self>>();

        info!("Printing in Prometheus File SD format");
        let mut w = if let Some(path) = &cmd.output {
            debug!("Opening file {} for writing", path.display());
            Box::new(File::create(path)?) as Box<dyn Write>
        } else {
            debug!("Opening stdout for writing");
            Box::new(stdout()) as Box<dyn Write>
        };

        writeln!(
            w,
            "{}",
            match cmd.format {
                PrometheusFormat::Yaml => serde_yaml::to_string(&configs)?,
                PrometheusFormat::Json => serde_json::to_string_pretty(&configs)?,
            }
        )?;

        Ok(())
    }
}
