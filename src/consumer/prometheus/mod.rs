use std::collections::HashMap;

use anyhow::Result;
use log::info;
use serde_derive::{Deserialize, Serialize};

use crate::{
    cli::{Prometheus as Cmd, PrometheusFormat},
    netbox::data::ip_address::IpAddress,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prometheus {
    targets: Vec<String>,
    labels: HashMap<String, String>,
}

impl TryFrom<IpAddress> for Prometheus {
    type Error = ();
    fn try_from(value: IpAddress) -> Result<Self, Self::Error> {
        let mut labels = HashMap::new();
        labels.insert("__meta_netbox_status".into(), value.status.to_string());
        labels.insert("__meta_netbox_id".into(), value.id.to_string());
        if let Some(tenant) = value.full_tenant {
            labels.insert("__meta_netbox_tenant".into(), tenant.slug);
            if let Some(group) = tenant.group {
                labels.insert("__meta_netbox_tenant_group".into(), group.slug);
            }
        }
        if let Some(site) = value.site {
            labels.insert("__meta_netbox_site".into(), site.slug.to_string());
        }
        if let Some(dns_name) = value.dns_name {
            labels.insert("__meta_netbox_dns_name".into(), dns_name.to_string());
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
        println!(
            "{}",
            match cmd.format {
                PrometheusFormat::Yaml => serde_yaml::to_string(&configs)?,
                PrometheusFormat::Json => serde_json::to_string_pretty(&configs)?,
            }
        );

        Ok(())
    }
}
