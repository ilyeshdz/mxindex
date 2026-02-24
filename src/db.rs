use crate::schema::servers;
use diesel::pg::PgConnection;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug, serde::Serialize)]
#[diesel(table_name = servers)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: i64,
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
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = servers)]
pub struct NewServer<'a> {
    pub domain: &'a str,
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub logo_url: Option<&'a str>,
    pub theme: Option<&'a str>,
    pub registration_open: Option<bool>,
    pub public_rooms_count: Option<i32>,
    pub version: Option<&'a str>,
    pub federation_version: Option<&'a str>,
    pub delegated_server: Option<&'a str>,
    pub room_versions: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct ServerFilter {
    pub search: Option<String>,
    pub registration_open: Option<bool>,
    pub has_rooms: Option<bool>,
    pub room_version: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(serde::Serialize)]
pub struct PaginatedServers {
    pub servers: Vec<Server>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

pub fn establish_connection() -> PgConnection {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    if let Some(parent) = std::path::Path::new(&database_url).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn insert_server(
    conn: &mut PgConnection,
    new_server: &NewServer,
) -> Result<Server, diesel::result::Error> {
    use crate::schema::servers::dsl::*;

    diesel::insert_into(servers)
        .values(new_server)
        .execute(conn)?;

    servers.order(id.desc()).first(conn)
}

pub fn get_server_by_domain(
    conn: &mut PgConnection,
    server_domain: &str,
) -> Result<Option<Server>, diesel::result::Error> {
    use crate::schema::servers::dsl::*;

    servers
        .filter(domain.eq(server_domain))
        .first(conn)
        .optional()
}

#[allow(dead_code)]
pub fn get_all_servers(conn: &mut PgConnection) -> Result<Vec<Server>, diesel::result::Error> {
    use crate::schema::servers::dsl::*;

    servers.load(conn)
}

pub fn get_filtered_servers(
    conn: &mut PgConnection,
    filter: &ServerFilter,
) -> Result<PaginatedServers, diesel::result::Error> {
    use crate::schema::servers::dsl::*;

    let limit = filter.limit.unwrap_or(50).clamp(1, 100);
    let offset = filter.offset.unwrap_or(0).max(0);

    let sort_by = filter.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = filter.sort_order.as_deref().unwrap_or("desc");

    let mut all_servers: Vec<Server> = servers.load(conn)?;

    if let Some(ref search) = filter.search {
        let search_lower = search.to_lowercase();
        all_servers.retain(|s| {
            s.domain.to_lowercase().contains(&search_lower)
                || s.name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&search_lower))
                    .unwrap_or(false)
                || s.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&search_lower))
                    .unwrap_or(false)
        });
    }

    if let Some(reg_open) = filter.registration_open {
        all_servers.retain(|s| s.registration_open == Some(reg_open));
    }

    if let Some(has_rooms) = filter.has_rooms {
        if has_rooms {
            all_servers.retain(|s| s.public_rooms_count.unwrap_or(0) > 0);
        } else {
            all_servers.retain(|s| s.public_rooms_count.unwrap_or(0) <= 0);
        }
    }

    if let Some(ref room_version) = filter.room_version {
        all_servers.retain(|s| {
            s.room_versions
                .as_ref()
                .map(|rv| rv.contains(room_version))
                .unwrap_or(false)
        });
    }

    let total = all_servers.len() as i64;

    match sort_by {
        "name" => {
            all_servers.sort_by(|a, b| {
                let a_name = a.name.as_deref().unwrap_or("");
                let b_name = b.name.as_deref().unwrap_or("");
                if sort_order == "asc" {
                    a_name.cmp(b_name)
                } else {
                    b_name.cmp(a_name)
                }
            });
        }
        "domain" => {
            all_servers.sort_by(|a, b| {
                if sort_order == "asc" {
                    a.domain.cmp(&b.domain)
                } else {
                    b.domain.cmp(&a.domain)
                }
            });
        }
        "public_rooms_count" => {
            all_servers.sort_by(|a, b| {
                let a_rooms = a.public_rooms_count.unwrap_or(0);
                let b_rooms = b.public_rooms_count.unwrap_or(0);
                if sort_order == "asc" {
                    a_rooms.cmp(&b_rooms)
                } else {
                    b_rooms.cmp(&a_rooms)
                }
            });
        }
        _ => {
            all_servers.sort_by(|a, b| {
                if sort_order == "asc" {
                    a.created_at.cmp(&b.created_at)
                } else {
                    b.created_at.cmp(&a.created_at)
                }
            });
        }
    }

    let result_servers: Vec<Server> = all_servers
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    Ok(PaginatedServers {
        servers: result_servers,
        total,
        limit,
        offset,
    })
}

pub fn run_migrations(conn: &mut PgConnection) {
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_server_struct() {
        let new_server = NewServer {
            domain: "matrix.org",
            name: Some("Matrix.org"),
            description: Some("The Matrix.org homeserver"),
            logo_url: Some("https://matrix.org/logo.png"),
            theme: Some("light"),
            registration_open: Some(true),
            public_rooms_count: Some(100),
            version: Some("v1.11"),
            federation_version: Some("Synapse/1.99"),
            delegated_server: Some("matrix.org:8448"),
            room_versions: Some("1,2,6"),
        };

        assert_eq!(new_server.domain, "matrix.org");
        assert_eq!(new_server.name, Some("Matrix.org"));
        assert_eq!(new_server.registration_open, Some(true));
    }

    #[test]
    fn test_new_server_partial() {
        let new_server = NewServer {
            domain: "test.org",
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

        assert_eq!(new_server.domain, "test.org");
        assert!(new_server.name.is_none());
        assert!(new_server.description.is_none());
    }

    #[test]
    fn test_server_filter_default() {
        let filter = ServerFilter::default();
        assert!(filter.search.is_none());
        assert!(filter.registration_open.is_none());
        assert!(filter.limit.is_none());
    }

    #[test]
    fn test_server_filter_with_values() {
        let filter = ServerFilter {
            search: Some("matrix".to_string()),
            registration_open: Some(true),
            has_rooms: Some(true),
            room_version: Some("6".to_string()),
            sort_by: Some("name".to_string()),
            sort_order: Some("asc".to_string()),
            limit: Some(10),
            offset: Some(0),
        };

        assert_eq!(filter.search, Some("matrix".to_string()));
        assert_eq!(filter.registration_open, Some(true));
        assert_eq!(filter.limit, Some(10));
    }
}
