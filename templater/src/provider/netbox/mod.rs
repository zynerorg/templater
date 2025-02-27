use std::borrow::Cow;

use anyhow::Result;
use clap::Args;
use data::{ip_address::IpAddress, prefix::Prefix, tenant::Tenant, List};
use log::{debug, info};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Url,
};
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};

use super::Provider;
use crate::data::AddressMain;

mod data;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
#[serde(deny_unknown_fields)]
pub struct Netbox {
    /// Netbox API endpoint
    #[arg(long, env("NETBOX_ENDPOINT"))]
    pub endpoint: String,
    /// Netbox API token
    #[arg(long, env("NETBOX_TOKEN"))]
    pub token: String,
}

impl Provider for Netbox {
    fn provide(self) -> Result<Vec<AddressMain>> {
        NetboxClient::new(self.endpoint, &self.token)?.fetch_addresses()
    }
}

struct NetboxClient {
    client: Client,
    base_address: String,
}

impl NetboxClient {
    fn new(base_address: String, token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let mut auth = HeaderValue::from_str(&format!("Token {token}"))?;
        auth.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            base_address,
        })
    }

    fn get_list<T: DeserializeOwned>(
        &self,
        variant: &str,
        query: Option<&[(&str, &str)]>,
    ) -> Result<Vec<T>> {
        debug!("Getting {}", variant);
        let mut builder = self
            .client
            .get(format!("{}/{}/", self.base_address, variant))
            .query(&[("format", "json")]);
        if let Some(query) = query {
            builder = builder.query(&query);
        }

        let response: List<T> = builder.send()?.error_for_status()?.json()?;
        let mut results = response.results;

        if let Some(next) = response.next {
            debug!("Pagination detected, running request again");
            let url = Url::parse(&next)?;
            let query: Vec<(Cow<'_, str>, Cow<'_, str>)> = url.query_pairs().collect();
            let query: Vec<(&str, &str)> = query
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
                .collect();
            let response_i: Vec<T> = self.get_list(variant, Some(&query))?;
            results.extend(response_i);
        }
        Ok(results)
    }

    fn fetch_addresses(&self) -> Result<Vec<AddressMain>> {
        info!("Fetching netbox addresses");
        let addresses: Vec<IpAddress> = self.get_list("/ipam/ip-addresses", None)?;
        let addresses = self.populate(addresses)?;
        Ok(addresses.into_iter().map(Into::into).collect())
    }

    fn populate(&self, mut addresses: Vec<IpAddress>) -> Result<Vec<IpAddress>> {
        info!("Populating netbox addresses with useful data");
        let prefixes: Vec<Prefix> =
            self.get_list("/ipam/prefixes", Some(&[("ordering", "prefix")]))?;

        let tenants: Vec<Tenant> = self.get_list("/tenancy/tenants", None)?;

        for address in &mut addresses {
            let mut scope = None;
            let mut vlan = None;
            for prefix in &prefixes {
                if prefix.prefix.contains(&address.address.addr()) {
                    if let Some(scope_i) = &prefix.scope {
                        scope = Some(scope_i);
                    }
                    if let Some(vlan_i) = &prefix.vlan {
                        vlan = Some(vlan_i);
                    }
                }
            }

            address.scope = scope.cloned();
            address.vlan = vlan.cloned();

            if let Some(tenant) = &address.tenant {
                if let Some(other) = tenants.iter().find(|t| tenant.id == t.id) {
                    address.full_tenant = Some(other.clone());
                }
            }
        }

        Ok(addresses)
    }
}
