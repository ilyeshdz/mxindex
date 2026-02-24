use chrono::NaiveDateTime;
use rocket::serde::Serialize;
use rocket_okapi::JsonSchema;

#[derive(Serialize, JsonSchema)]
pub struct ApiInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ServerInfo {
    pub server: String,
    pub status: String,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(rocket::serde::Deserialize, JsonSchema)]
pub struct CreateServerRequest {
    pub domain: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ServerResponse {
    pub id: i32,
    pub domain: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub theme: Option<String>,
    pub registration_open: Option<bool>,
    pub public_rooms_count: Option<i32>,
    pub version: Option<String>,
    pub federation_version: Option<String>,
    pub delegated_server: Option<String>,
    pub room_versions: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, JsonSchema)]
pub struct PaginatedServersResponse {
    pub servers: Vec<ServerResponse>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug)]
pub struct DiscoveredServerInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub theme: Option<String>,
    pub registration_open: Option<bool>,
    pub public_rooms_count: Option<i32>,
    pub version: Option<String>,
    pub federation_version: Option<String>,
    pub delegated_server: Option<String>,
    pub room_versions: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_info_default() {
        let info = ApiInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test API".to_string(),
        };

        assert_eq!(info.name, "test");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_server_info_online() {
        let info = ServerInfo {
            server: "matrix.org".to_string(),
            status: "online".to_string(),
            version: Some("v1.0".to_string()),
            error: None,
        };

        assert_eq!(info.status, "online");
        assert!(info.error.is_none());
    }

    #[test]
    fn test_server_info_offline() {
        let info = ServerInfo {
            server: "invalid.server".to_string(),
            status: "offline".to_string(),
            version: None,
            error: Some("connection_error".to_string()),
        };

        assert_eq!(info.status, "offline");
        assert!(info.error.is_some());
    }

    #[test]
    fn test_error_response() {
        let err = ErrorResponse {
            error: "invalid_domain".to_string(),
            message: "Domain is invalid".to_string(),
        };

        assert_eq!(err.error, "invalid_domain");
    }

    #[test]
    fn test_create_server_request() {
        let req = CreateServerRequest {
            domain: "matrix.org".to_string(),
        };

        assert_eq!(req.domain, "matrix.org");
    }

    #[test]
    fn test_server_response() {
        let response = ServerResponse {
            id: 1,
            domain: "matrix.org".to_string(),
            name: Some("Matrix.org".to_string()),
            description: Some("The Matrix.org homeserver".to_string()),
            logo_url: Some("https://matrix.org/images/logo.png".to_string()),
            theme: Some("light".to_string()),
            registration_open: Some(true),
            public_rooms_count: Some(500),
            version: Some("v1.11".to_string()),
            federation_version: Some("Synapse/1.99".to_string()),
            delegated_server: None,
            room_versions: Some("1,2,6,9".to_string()),
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            updated_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        };

        assert_eq!(response.id, 1);
        assert_eq!(response.domain, "matrix.org");
    }
}
