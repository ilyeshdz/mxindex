use matrix_sdk::Client;
use matrix_sdk::ruma::api::client::directory::get_public_rooms_filtered;
use matrix_sdk::ruma::api::client::discovery::get_supported_versions;
use matrix_sdk::ruma::UInt;
use crate::models::DiscoveredServerInfo;
use reqwest;
use serde::Deserialize;

pub struct MatrixService;

#[derive(Deserialize)]
struct WellKnownInfo {
    name: Option<String>,
    description: Option<String>,
}

impl MatrixService {
    pub async fn check_server_status(server: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_url = format!("https://{}", server);

        let client = Client::builder()
            .homeserver_url(&server_url)
            .build()
            .await?;

        client.get_capabilities().await?;

        Ok(())
    }

    pub async fn discover_server_info(domain: &str) -> Result<DiscoveredServerInfo, Box<dyn std::error::Error + Send + Sync>> {
        let server_url = format!("https://{}", domain);

        let client = Client::builder()
            .homeserver_url(&server_url)
            .build()
            .await?;

        let capabilities = client.get_capabilities().await?;
        
        let registration_open = Some(capabilities.change_password.enabled);

        let public_rooms_count = Self::get_public_rooms_count(&client).await.ok();

        let (name, description) = Self::fetch_well_known_info(domain).await?;

        Ok(DiscoveredServerInfo {
            name,
            description,
            registration_open,
            public_rooms_count,
        })
    }

    async fn get_public_rooms_count(client: &Client) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let request = get_public_rooms_filtered::v3::Request::new();
        
        let response = client.send(request).await?;
        
        let total: UInt = response.total_room_count_estimate.unwrap_or(UInt::from(0u32));
        let total_count: i32 = total.to_string().parse::<i64>().unwrap_or(0) as i32;
        
        Ok(total_count)
    }

    async fn fetch_well_known_info(domain: &str) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
        let well_known_url = format!("https://{}/.well-known/matrix/client", domain);
        
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        
        let response = http_client.get(&well_known_url).send().await?;
        
        if !response.status().is_success() {
            return Ok((None, None));
        }
        
        let well_known: WellKnownInfo = response.json().await?;
        
        Ok((well_known.name, well_known.description))
    }

    pub async fn get_server_version(server: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let server_url = format!("https://{}", server);
        
        let request = get_supported_versions::Request::new();
        
        let client = Client::builder()
            .homeserver_url(&server_url)
            .build()
            .await?;
        
        let response = client.send(request).await?;
        
        let versions: Vec<String> = response.versions.iter().map(|v| v.to_string()).collect();
        
        Ok(versions.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_url_format() {
        let server = "matrix.org";
        let expected = "https://matrix.org";
        assert_eq!(format!("https://{}", server), expected);
    }

    #[test]
    fn test_well_known_url_format() {
        let domain = "matrix.org";
        let expected = "https://matrix.org/.well-known/matrix/client";
        assert_eq!(format!("https://{}/.well-known/matrix/client", domain), expected);
    }

    #[test]
    fn test_discovered_server_info_default() {
        let info = DiscoveredServerInfo {
            name: None,
            description: None,
            registration_open: None,
            public_rooms_count: None,
        };
        
        assert!(info.name.is_none());
        assert!(info.description.is_none());
        assert!(info.registration_open.is_none());
        assert!(info.public_rooms_count.is_none());
    }

    #[test]
    fn test_discovered_server_info_with_values() {
        let info = DiscoveredServerInfo {
            name: Some("Test Server".to_string()),
            description: Some("A test Matrix server".to_string()),
            registration_open: Some(true),
            public_rooms_count: Some(100),
        };
        
        assert_eq!(info.name, Some("Test Server".to_string()));
        assert_eq!(info.description, Some("A test Matrix server".to_string()));
        assert_eq!(info.registration_open, Some(true));
        assert_eq!(info.public_rooms_count, Some(100));
    }

    #[test]
    fn test_well_known_info_deserialization() {
        let json = r#"{"name": "Test Server", "description": "A test server"}"#;
        let info: WellKnownInfo = serde_json::from_str(json).unwrap();
        
        assert_eq!(info.name, Some("Test Server".to_string()));
        assert_eq!(info.description, Some("A test server".to_string()));
    }

    #[test]
    fn test_well_known_info_partial_deserialization() {
        let json = r#"{"name": "Test Server"}"#;
        let info: WellKnownInfo = serde_json::from_str(json).unwrap();
        
        assert_eq!(info.name, Some("Test Server".to_string()));
        assert!(info.description.is_none());
    }
}
