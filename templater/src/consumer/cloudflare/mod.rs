mod data;

use anyhow::{Result, anyhow};
use clap::Args;
use data::{List, batch::Batch, record::Records, zone::Zone};
use log::{debug, error, info};
use reqwest::{
    blocking::Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

use super::Consumer;
use crate::data::{AddressMain, Domains};

const BASE_URL: &str = "https://api.cloudflare.com/client/v4";

#[serde_inline_default]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Args)]
#[serde(deny_unknown_fields)]
pub struct Cloudflare {
    #[arg(long, env("CLOUDFLARE_TOKEN"))]
    pub token: String,
    #[arg(long, env("CLOUDFLARE_TTL"), default_value = "1")]
    #[serde_inline_default(1)]
    pub ttl: usize,
    #[arg(long, env("CLOUDFLARE_DRY_RUN"), default_value_t)]
    #[serde(default)]
    pub dry_run: bool,
    #[arg(long, env("CLOUDFLARE_ZONES"))]
    pub zones: Option<Vec<String>>,
}

impl Default for Cloudflare {
    fn default() -> Self {
        Self {
            token: String::default(),
            ttl: 1,
            dry_run: Default::default(),
            zones: Option::default(),
        }
    }
}

impl Consumer for Cloudflare {
    fn consume(&self, addresses: Vec<AddressMain>) -> anyhow::Result<()> {
        CloudflareClient::new(self)?.push(addresses)?;
        Ok(())
    }
}

struct CloudflareClient {
    client: Client,
    config: Cloudflare,
}

impl CloudflareClient {
    fn new(config: &Cloudflare) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let mut auth = HeaderValue::from_str(&format!("Bearer {}", config.token))?;
        auth.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    // TODO: Possibly combine with netbox get_list, type and page check different
    fn get_list<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
    ) -> Result<Vec<T>> {
        debug!("Getting {endpoint}");
        let mut builder = self.client.get(format!("{BASE_URL}/{endpoint}"));
        if let Some(query) = query {
            builder = builder.query(&query);
        }

        let response: List<T> = builder.send()?.error_for_status()?.json()?;
        let mut results = response.result;

        if response.result_info.page < response.result_info.total_pages {
            debug!("Pagination detection, running request again");
            let page = (response.result_info.page + 1).to_string();
            let mut query: Vec<(&str, &str)> = query.map_or(vec![], |v| {
                v.iter().filter(|q| q.0 != "page").copied().collect()
            });
            query.push(("page", &page));
            let response_i: Vec<T> = self.get_list(endpoint, Some(&query))?;
            results.extend(response_i);
        }
        Ok(results)
    }

    fn push(&self, addresses: Vec<AddressMain>) -> Result<()> {
        info!("Fetching data from cloudflare");
        let zones: Vec<Zone> = self.get_list("/zones", None)?;

        let mut domains: Domains = addresses.clone().into();
        let reverse_domains = Domains::reverse_from_addresses(addresses);
        domains.0.extend(reverse_domains.0);

        let zip = zones
            .into_iter()
            .filter(|zone| {
                let Some(filter) = &self.config.zones else {
                    return true;
                };
                filter.contains(&zone.name)
            })
            .filter_map(|zone| {
                let mut domain_out = None;
                for domain in &domains.0 {
                    if zone.name == domain.name {
                        domain_out = Some(domain);
                    }
                }
                let old: Records = self
                    .get_list(&format!("/zones/{}/dns_records", zone.id), None)
                    .unwrap()
                    .into();
                domain_out.cloned().map(|domain| {
                    let new = Records::from_domain(&self.config, domain)?;
                    Ok((zone.id, old.filter_other(), new))
                })
            })
            .collect::<Result<Vec<_>>>()?;

        info!("Sending batch updates to cloudflare");
        for (id, old, new) in zip {
            let remove = old.clone() - &new;
            let add = new - &old;

            for record in &remove.0 {
                let Some(record) = record.generic() else {
                    continue;
                };
                debug!(
                    "Removing record {:?} {} {} {} {:?}",
                    record.id, record.name, record.content, record.ttl, record.comment
                );
            }

            for record in &add.0 {
                let Some(record) = record.generic() else {
                    continue;
                };
                debug!(
                    "Adding record {} {} {} {:?}",
                    record.name, record.content, record.ttl, record.comment
                );
            }

            let batch = Batch {
                deletes: remove.0,
                posts: add.0,
            };

            if batch.deletes.is_empty() && batch.posts.is_empty() {
                debug!("Nothing to modify, skipping");
                continue;
            }

            if self.config.dry_run {
                debug!("Dry run, skipping");
                continue;
            }

            let response = self
                .client
                .post(format!("{BASE_URL}/zones/{id}/dns_records/batch"))
                .body(serde_json::to_string(&batch)?)
                .send()?;
            if let Err(err) = response.error_for_status_ref() {
                error!("Cloudflare responded with: {}", response.text()?);
                return Err(anyhow!(err));
            }
        }

        Ok(())
    }
}
