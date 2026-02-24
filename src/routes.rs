use rocket::serde::json::Json;
use rocket_okapi::openapi;
use crate::models::{ApiInfo, ServerInfo, ErrorResponse, CreateServerRequest, ServerResponse, PaginatedServersResponse};
use crate::services::MatrixService;
use crate::db::{establish_connection, insert_server, get_server_by_domain, get_filtered_servers, ServerFilter};

#[openapi]
#[get("/")]
pub fn index() -> Json<ApiInfo> {
    Json(ApiInfo {
        name: "mxindex".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Matrix homeserver index API".to_string(),
    })
}

#[openapi]
#[get("/servers/<server>")]
pub async fn server_info(server: &str) -> Result<Json<ServerInfo>, Json<ErrorResponse>> {
    if server.is_empty() || server.contains('/') || server.contains(':') {
        return Err(Json(ErrorResponse {
            error: "invalid_server".to_string(),
            message: "Server name must be a valid domain name without path or port".to_string(),
        }));
    }

    let version = MatrixService::get_server_version(server).await.ok();

    match MatrixService::check_server_status(server).await {
        Ok(_) => Ok(Json(ServerInfo {
            server: server.to_string(),
            status: "online".to_string(),
            version,
            error: None,
        })),
        Err(e) => {
            let error_type = if e.to_string().contains("dns") {
                "dns_error"
            } else if e.to_string().contains("connect") {
                "connection_error"
            } else {
                "server_error"
            };

            Ok(Json(ServerInfo {
                server: server.to_string(),
                status: "offline".to_string(),
                version,
                error: Some(error_type.to_string()),
            }))
        }
    }
}

#[openapi]
#[post("/servers", data = "<request>")]
pub async fn add_server(request: Json<CreateServerRequest>) -> Result<Json<ServerResponse>, Json<ErrorResponse>> {
    if request.domain.is_empty() || request.domain.contains('/') || request.domain.contains(':') {
        return Err(Json(ErrorResponse {
            error: "invalid_domain".to_string(),
            message: "Domain must be a valid domain name without path or port".to_string(),
        }));
    }

    let mut conn = establish_connection();
    if let Ok(Some(_)) = get_server_by_domain(&mut conn, &request.domain) {
        return Err(Json(ErrorResponse {
            error: "server_exists".to_string(),
            message: "Server already exists in the index".to_string(),
        }));
    }

    match MatrixService::discover_server_info(&request.domain).await {
        Ok(discovered) => {
            let new_server = crate::db::NewServer {
                domain: &request.domain,
                name: discovered.name.as_deref(),
                description: discovered.description.as_deref(),
                logo_url: discovered.logo_url.as_deref(),
                theme: discovered.theme.as_deref(),
                registration_open: discovered.registration_open,
                public_rooms_count: discovered.public_rooms_count,
                version: discovered.version.as_deref(),
                federation_version: discovered.federation_version.as_deref(),
                delegated_server: discovered.delegated_server.as_deref(),
                room_versions: discovered.room_versions.as_deref(),
            };

            match insert_server(&mut conn, &new_server) {
                Ok(server) => Ok(Json(ServerResponse {
                    id: server.id,
                    domain: server.domain,
                    name: server.name,
                    description: server.description,
                    logo_url: server.logo_url,
                    theme: server.theme,
                    registration_open: server.registration_open,
                    public_rooms_count: server.public_rooms_count,
                    version: server.version,
                    federation_version: server.federation_version,
                    delegated_server: server.delegated_server,
                    room_versions: server.room_versions,
                    created_at: server.created_at,
                    updated_at: server.updated_at,
                })),
                Err(e) => Err(Json(ErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("Failed to save server: {}", e),
                })),
            }
        }
        Err(e) => Err(Json(ErrorResponse {
            error: "discovery_failed".to_string(),
            message: format!("Failed to discover server information: {}", e),
        })),
    }
}

#[openapi]
#[get("/servers")]
pub fn list_servers() -> Result<Json<PaginatedServersResponse>, Json<ErrorResponse>> {
    let mut conn = establish_connection();
    
    let filter = ServerFilter::default();
    
    match get_filtered_servers(&mut conn, &filter) {
        Ok(result) => {
            let responses = result.servers.into_iter().map(|s| ServerResponse {
                id: s.id,
                domain: s.domain,
                name: s.name,
                description: s.description,
                logo_url: s.logo_url,
                theme: s.theme,
                registration_open: s.registration_open,
                public_rooms_count: s.public_rooms_count,
                version: s.version,
                federation_version: s.federation_version,
                delegated_server: s.delegated_server,
                room_versions: s.room_versions,
                created_at: s.created_at,
                updated_at: s.updated_at,
            }).collect();
            
            Ok(Json(PaginatedServersResponse {
                servers: responses,
                total: result.total,
                limit: result.limit,
                offset: result.offset,
            }))
        },
        Err(e) => Err(Json(ErrorResponse {
            error: "database_error".to_string(),
            message: format!("Failed to fetch servers: {}", e),
        })),
    }
}

#[openapi]
#[get("/servers/search?<search>&<registration_open>&<has_rooms>&<room_version>&<sort_by>&<sort_order>&<limit>&<offset>")]
pub fn search_servers(
    search: Option<String>,
    registration_open: Option<bool>,
    has_rooms: Option<bool>,
    room_version: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Json<PaginatedServersResponse>, Json<ErrorResponse>> {
    let mut conn = establish_connection();
    
    let filter = ServerFilter {
        search,
        registration_open,
        has_rooms,
        room_version,
        sort_by,
        sort_order,
        limit,
        offset,
    };
    
    match get_filtered_servers(&mut conn, &filter) {
        Ok(result) => {
            let responses = result.servers.into_iter().map(|s| ServerResponse {
                id: s.id,
                domain: s.domain,
                name: s.name,
                description: s.description,
                logo_url: s.logo_url,
                theme: s.theme,
                registration_open: s.registration_open,
                public_rooms_count: s.public_rooms_count,
                version: s.version,
                federation_version: s.federation_version,
                delegated_server: s.delegated_server,
                room_versions: s.room_versions,
                created_at: s.created_at,
                updated_at: s.updated_at,
            }).collect();
            
            Ok(Json(PaginatedServersResponse {
                servers: responses,
                total: result.total,
                limit: result.limit,
                offset: result.offset,
            }))
        },
        Err(e) => Err(Json(ErrorResponse {
            error: "database_error".to_string(),
            message: format!("Failed to fetch servers: {}", e),
        })),
    }
}
