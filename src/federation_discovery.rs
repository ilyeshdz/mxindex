use crate::db::{insert_server, DbPool};
use crate::http_client::get_http_client;
use crate::models::CreateServerRequest;
use crate::services::MatrixService;
use diesel::prelude::*;
use futures::stream::{self, StreamExt};
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct FederationDiscovery {
    db_pool: DbPool,
    max_concurrent: usize,
    max_depth: usize,
    batch_size: usize,
    seed_servers: Vec<String>,
}

impl FederationDiscovery {
    pub fn new(db_pool: DbPool) -> Self {
        let max_concurrent = std::env::var("FEDERATION_DISCOVERY_CONCURRENT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        let max_depth = std::env::var("FEDERATION_DISCOVERY_DEPTH")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        let batch_size = std::env::var("FEDERATION_DISCOVERY_BATCH")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let seed_servers = std::env::var("FEDERATION_SEED_SERVERS")
            .ok()
            .map(|s| s.split(',').map(String::from).collect())
            .unwrap_or_else(|| vec!["matrix.org".to_string()]);

        Self {
            db_pool,
            max_concurrent,
            max_depth,
            batch_size,
            seed_servers,
        }
    }

    pub async fn start_discovery(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting federation discovery with {} seed servers, max depth: {}, concurrent: {}",
            self.seed_servers.len(),
            self.max_depth,
            self.max_concurrent
        );

        let mut discovered: HashSet<String> = HashSet::new();
        let mut servers_to_check: Vec<String> = self.seed_servers.clone();
        let mut added_count = 0;

        for _depth in 0..self.max_depth {
            if servers_to_check.is_empty() {
                break;
            }

            info!(
                "Discovery round: checking {} servers",
                servers_to_check.len()
            );

            let semaphore = Arc::new(Semaphore::new(self.max_concurrent));

            #[allow(clippy::type_complexity)]
            let results: Vec<(
                String,
                Result<HashSet<String>, Box<dyn std::error::Error + Send + Sync>>,
            )> = stream::iter(servers_to_check.clone())
                .map(|server| {
                    let semaphore = semaphore.clone();
                    async move {
                        let _permit = semaphore.acquire().await.expect("Failed to acquire permit");
                        let result = tokio::time::timeout(
                            std::time::Duration::from_secs(10),
                            Self::discover_servers_from_federation(&server),
                        )
                        .await;
                        match result {
                            Ok(Ok(servers)) => (server, Ok(servers)),
                            Ok(Err(e)) => (server, Err(e)),
                            Err(_) => {
                                let err: Box<dyn std::error::Error + Send + Sync> =
                                    "Timeout".into();
                                (server, Err(err))
                            }
                        }
                    }
                })
                .buffer_unordered(self.max_concurrent)
                .collect()
                .await;

            servers_to_check.clear();

            for (server, result) in results {
                match result {
                    Ok(new_servers) => {
                        for new_server in new_servers {
                            if !discovered.contains(&new_server) {
                                discovered.insert(new_server.clone());
                                servers_to_check.push(new_server.clone());

                                if self.add_server_to_index(&new_server).await {
                                    added_count += 1;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to discover from {}: {}", server, e);
                    }
                }
            }

            if servers_to_check.len() > self.batch_size {
                servers_to_check.truncate(self.batch_size);
            }
        }

        info!(
            "Federation discovery complete. Added {} new servers",
            added_count
        );
        Ok(added_count)
    }

    async fn discover_servers_from_federation(
        server: &str,
    ) -> Result<HashSet<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut discovered_servers: HashSet<String> = HashSet::new();

        let servers_from_rooms = Self::discover_servers_from_public_rooms(server).await?;
        discovered_servers.extend(servers_from_rooms);

        Ok(discovered_servers)
    }

    async fn discover_servers_from_public_rooms(
        server: &str,
    ) -> Result<HashSet<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut servers: HashSet<String> = HashSet::new();
        let server_url = format!("https://{}/_matrix/client/r0/publicRooms", server);

        let http_client = get_http_client();

        let response = http_client
            .get(&server_url)
            .query(&[("limit", "100")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(servers);
        }

        let json: serde_json::Value = response.json().await?;

        if let Some(chunks) = json["chunk"].as_array() {
            for chunk in chunks {
                if let Some(heroes) = chunk["heroes"].as_array() {
                    for hero in heroes {
                        if let Some(mxid) = hero["mxid"].as_str() {
                            if let Some(domain) = extract_domain_from_mxid(mxid) {
                                if domain != server {
                                    servers.insert(domain);
                                }
                            }
                        }
                    }
                }

                if let Some(topic) = chunk["topic"].as_str() {
                    for domain in extract_domains_from_text(topic) {
                        if domain != server {
                            servers.insert(domain);
                        }
                    }
                }
            }
        }

        Ok(servers)
    }

    async fn add_server_to_index(&self, domain: &str) -> bool {
        if domain.is_empty() || domain.contains('/') || domain.contains(':') {
            return false;
        }

        if let Ok(exists) = self.server_exists_in_db(domain).await {
            if exists {
                return false;
            }
        }

        let request = CreateServerRequest {
            domain: domain.to_string(),
        };

        match MatrixService::discover_server_info(domain).await {
            Ok(info) => {
                use crate::db::NewServer;

                let mut conn = match self.db_pool.get() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to get DB connection: {}", e);
                        return false;
                    }
                };

                let domain_str = domain.to_string();

                let new_server = NewServer {
                    domain: &request.domain,
                    name: info.name.as_deref(),
                    description: info.description.as_deref(),
                    logo_url: info.logo_url.as_deref(),
                    theme: info.theme.as_deref(),
                    registration_open: info.registration_open,
                    public_rooms_count: info.public_rooms_count,
                    version: info.version.as_deref(),
                    federation_version: info.federation_version.as_deref(),
                    delegated_server: info.delegated_server.as_deref(),
                    room_versions: info.room_versions.as_deref(),
                };

                match insert_server(&mut conn, &new_server) {
                    Ok(_) => {
                        info!("Added server from federation discovery: {}", domain_str);
                        true
                    }
                    Err(e) => {
                        warn!("Failed to insert server {}: {}", domain_str, e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("Failed to discover server info for {}: {}", domain, e);
                false
            }
        }
    }

    async fn server_exists_in_db(
        &self,
        domain: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        use crate::schema::servers::dsl::servers;

        let mut conn = self.db_pool.get()?;
        let count = servers
            .filter(crate::schema::servers::domain.eq(domain))
            .count()
            .get_result::<i64>(&mut conn)?;
        Ok(count > 0)
    }
}

fn extract_domain_from_mxid(mxid: &str) -> Option<String> {
    if mxid.starts_with('@') {
        let parts: Vec<&str> = mxid.splitn(2, ':').collect();
        if parts.len() == 2 {
            return Some(parts[1].to_string());
        }
    }
    None
}

fn extract_domains_from_text(text: &str) -> Vec<String> {
    let mut domains = Vec::new();
    let domain_regex = Regex::new(r"[a-zA-Z0-9][-a-zA-Z0-9]*\.[a-zA-Z]{2,}[/:]?").ok();

    if let Some(regex) = domain_regex {
        for cap in regex.find_iter(text) {
            let domain = cap.as_str().trim_end_matches('/').to_string();
            if domain.contains('.') && !domain.ends_with(".onion") {
                domains.push(domain);
            }
        }
    }

    domains
}
