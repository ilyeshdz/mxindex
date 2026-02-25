use crate::http_client::get_http_client;
use crate::models::DiscoveredServerInfo;
use matrix_sdk::Client;
use matrix_sdk::ruma::UInt;
use matrix_sdk::ruma::api::client::directory::get_public_rooms_filtered;
use matrix_sdk::ruma::api::client::discovery::get_supported_versions;
use serde::Deserialize;

pub struct MatrixService;

#[derive(Deserialize)]
struct WellKnownClientInfo {
    name: Option<String>,
    description: Option<String>,
    logo_url: Option<String>,
    theme: Option<String>,
}

#[derive(Deserialize)]
struct WellKnownServerInfo {
    #[serde(rename = "m.server")]
    m_server: Option<String>,
}

#[derive(Deserialize)]
struct FederationVersionInfo {
    server: Option<String>,
}

impl MatrixService {
    pub async fn check_server_status(
        server: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_url = format!("https://{}", server);

        let client = Client::builder()
            .homeserver_url(&server_url)
            .build()
            .await?;

        client.get_capabilities().await?;

        Ok(())
    }

    pub async fn discover_server_info(
        domain: &str,
    ) -> Result<DiscoveredServerInfo, Box<dyn std::error::Error + Send + Sync>> {
        let server_url = format!("https://{}", domain);

        let client = Client::builder()
            .homeserver_url(&server_url)
            .build()
            .await?;

        let capabilities = client.get_capabilities().await?;

        let registration_open = Some(capabilities.change_password.enabled);

        let room_versions: Vec<String> = capabilities
            .room_versions
            .available
            .keys()
            .map(|v| v.to_string())
            .collect();
        let room_versions_str = if room_versions.is_empty() {
            None
        } else {
            Some(room_versions.join(","))
        };

        let public_rooms_count = Self::get_public_rooms_count(&client).await.ok();

        let (name, description, logo_url, theme) =
            Self::fetch_well_known_client_info(domain).await?;

        let version = Self::get_server_version(domain).await.ok();
        let federation_version = Self::get_federation_version(domain).await.ok();
        let delegated_server = Self::fetch_well_known_server_info(domain).await?;

        Ok(DiscoveredServerInfo {
            name,
            description,
            logo_url,
            theme,
            registration_open,
            public_rooms_count,
            version,
            federation_version,
            delegated_server,
            room_versions: room_versions_str,
        })
    }

    async fn get_public_rooms_count(
        client: &Client,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let request = get_public_rooms_filtered::v3::Request::new();

        let response = client.send(request).await?;

        let total: UInt = response
            .total_room_count_estimate
            .unwrap_or(UInt::from(0u32));
        let total_count: i32 = total.to_string().parse::<i64>().unwrap_or(0) as i32;

        Ok(total_count)
    }

    async fn fetch_well_known_client_info(
        domain: &str,
    ) -> Result<
        (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let well_known_url = format!("https://{}/.well-known/matrix/client", domain);

        let http_client = get_http_client();

        let response = http_client.get(&well_known_url).send().await?;

        if !response.status().is_success() {
            return Ok((None, None, None, None));
        }

        let well_known: WellKnownClientInfo = response.json().await?;

        Ok((
            well_known.name,
            well_known.description,
            well_known.logo_url,
            well_known.theme,
        ))
    }

    async fn fetch_well_known_server_info(
        domain: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let well_known_url = format!("https://{}/.well-known/matrix/server", domain);

        let http_client = get_http_client();

        let response = http_client.get(&well_known_url).send().await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let well_known: WellKnownServerInfo = response.json().await?;

        Ok(well_known.m_server)
    }

    pub async fn get_server_version(
        server: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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

    pub async fn get_federation_version(
        server: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let federation_url = format!("https://{}/_matrix/federation/v1/version", server);

        let http_client = get_http_client();

        let response = http_client.get(&federation_url).send().await?;

        if !response.status().is_success() {
            return Err("Failed to get federation version".into());
        }

        let info: FederationVersionInfo = response.json().await?;

        Ok(info.server.unwrap_or_default())
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
    fn test_well_known_client_url_format() {
        let domain = "matrix.org";
        let expected = "https://matrix.org/.well-known/matrix/client";
        assert_eq!(
            format!("https://{}/.well-known/matrix/client", domain),
            expected
        );
    }

    #[test]
    fn test_well_known_server_url_format() {
        let domain = "matrix.org";
        let expected = "https://matrix.org/.well-known/matrix/server";
        assert_eq!(
            format!("https://{}/.well-known/matrix/server", domain),
            expected
        );
    }

    #[test]
    fn test_federation_version_url_format() {
        let server = "matrix.org";
        let expected = "https://matrix.org/_matrix/federation/v1/version";
        assert_eq!(
            format!("https://{}/_matrix/federation/v1/version", server),
            expected
        );
    }

    #[test]
    fn test_discovered_server_info_default() {
        let info = DiscoveredServerInfo {
            name: None,
            description: None,
            logo_url: None,
            theme: None,
            registration_open: None,
            public_rooms_count: None,
            version: None,
            federation_version: None,
            delegated_server: None,
            room_versions: None,
        };

        assert!(info.name.is_none());
        assert!(info.description.is_none());
        assert!(info.logo_url.is_none());
        assert!(info.theme.is_none());
        assert!(info.registration_open.is_none());
        assert!(info.public_rooms_count.is_none());
    }

    #[test]
    fn test_discovered_server_info_with_values() {
        let info = DiscoveredServerInfo {
            name: Some("Test Server".to_string()),
            description: Some("A test Matrix server".to_string()),
            logo_url: Some("https://test.org/logo.png".to_string()),
            theme: Some("dark".to_string()),
            registration_open: Some(true),
            public_rooms_count: Some(100),
            version: Some("v1.11".to_string()),
            federation_version: Some("Synapse/1.99".to_string()),
            delegated_server: Some("test.org:8448".to_string()),
            room_versions: Some("1,2,6".to_string()),
        };

        assert_eq!(info.name, Some("Test Server".to_string()));
        assert_eq!(info.description, Some("A test Matrix server".to_string()));
        assert_eq!(info.logo_url, Some("https://test.org/logo.png".to_string()));
        assert_eq!(info.theme, Some("dark".to_string()));
        assert_eq!(info.registration_open, Some(true));
        assert_eq!(info.public_rooms_count, Some(100));
    }

    #[test]
    fn test_well_known_client_info_deserialization() {
        let json = r#"{"name": "Test Server", "description": "A test server", "logo_url": "https://test.org/logo.png", "theme": "dark"}"#;
        let info: WellKnownClientInfo = serde_json::from_str(json).unwrap();

        assert_eq!(info.name, Some("Test Server".to_string()));
        assert_eq!(info.description, Some("A test server".to_string()));
        assert_eq!(info.logo_url, Some("https://test.org/logo.png".to_string()));
        assert_eq!(info.theme, Some("dark".to_string()));
    }

    #[test]
    fn test_well_known_server_info_deserialization() {
        let json = r#"{"m.server": "matrix.org:8448"}"#;
        let info: WellKnownServerInfo = serde_json::from_str(json).unwrap();

        assert_eq!(info.m_server, Some("matrix.org:8448".to_string()));
    }

    #[test]
    fn test_federation_version_info_deserialization() {
        let json = r#"{"server": "Synapse/1.99.0"}"#;
        let info: FederationVersionInfo = serde_json::from_str(json).unwrap();

        assert_eq!(info.server, Some("Synapse/1.99.0".to_string()));
    }
}
