use std::borrow::Cow;

use anyhow::Result;
use data::{ip_address::IpAddress, prefix::Prefix, List};
use log::{debug, info};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Url,
};
use serde::de::DeserializeOwned;
use tldextract::{TldExtractor, TldOption};

use crate::{cli::Cli, netbox::data::tenant::Tenant};

pub mod data;

// TODO: Dedup
macro_rules! gen_filter {
    ($address:ident, $cli:ident, $name:ident, $sub:ident) => {
        if let Some(a) = &$cli.$name {
            let mut state = false;
            let Some(address_name) = &$address.$name else {
                return false;
            };
            for value in a {
                state = state || value == &address_name.$sub;
            }
            if !state {
                return false;
            };
        }
    };
    ($address:ident, $cli:ident, $name:ident) => {
        if let Some(a) = &$cli.$name {
            let mut state = false;
            let Some(address_name) = $name else {
                return false;
            };
            for value in a {
                state = state || value == address_name;
            }
            if !state {
                return false;
            };
        }
    };
}

pub struct Netbox {
    client: Client,
    base_address: String,
    tld_extractor: TldExtractor,
}

impl Netbox {
    pub fn new(base_address: String, token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let mut auth = HeaderValue::from_str(&format!("Token {token}"))?;
        auth.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            base_address,
            tld_extractor: TldOption::default().naive_mode(true).build(),
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

    pub fn fetch_addresses(&self) -> Result<Vec<IpAddress>> {
        info!("Fetching netbox addresses");
        let addresses: Vec<IpAddress> = self.get_list("/ipam/ip-addresses", None)?;
        self.populate(addresses)
    }

    fn populate(&self, mut addresses: Vec<IpAddress>) -> Result<Vec<IpAddress>> {
        info!("Populating netbox addresses with useful data");
        let prefixes: Vec<Prefix> =
            self.get_list("/ipam/prefixes", Some(&[("ordering", "prefix")]))?;

        let tenants: Vec<Tenant> = self.get_list("/tenancy/tenants", None)?;

        for address in &mut addresses {
            let mut site = None;
            let mut vlan = None;
            for prefix in &prefixes {
                if prefix.prefix.contains(&address.address) {
                    if let Some(site_i) = &prefix.site {
                        site = Some(site_i);
                    }
                    if let Some(vlan_i) = &prefix.vlan {
                        vlan = Some(vlan_i);
                    }
                }
            }

            address.site = site.cloned();
            address.vlan = vlan.cloned();

            if let Some(tenant) = &address.tenant {
                if let Some(other) = tenants.iter().find(|t| tenant.id == t.id) {
                    address.full_tenant = Some(other.clone());
                }
            }

            if let Some(dns_name) = &address.dns_name {
                let res = self.tld_extractor.extract(dns_name)?;
                if let (Some(domain), Some(suffix)) = (res.domain, res.suffix) {
                    address.domain = Some(format!("{domain}.{suffix}"));
                }
            }
        }

        Ok(addresses)
    }

    pub fn filter(&self, cli: &Cli, addresses: Vec<IpAddress>) -> Vec<IpAddress> {
        info!("Filtering netbox addresses");
        addresses
            .into_iter()
            .filter(|address| {
                gen_filter!(address, cli, tenant, slug);
                gen_filter!(address, cli, site, slug);
                gen_filter!(address, cli, vlan, vid);
                let status = Some(&address.status);
                gen_filter!(address, cli, status);
                let domain = &address.domain;
                gen_filter!(address, cli, domain);
                true
            })
            .collect()
    }
}
