use crate::schema::servers;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

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

pub fn create_pool() -> DbPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create DB pool")
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

    let search_pattern = filter
        .search
        .as_ref()
        .map(|s| format!("%{}%", s.to_lowercase()));
    let room_version_pattern = filter.room_version.as_ref().map(|rv| format!("%{}%", rv));

    let mut count_query = servers.into_boxed();

    if let Some(ref pattern) = search_pattern {
        count_query = count_query.filter(
            domain
                .ilike(pattern)
                .or(name.ilike(pattern))
                .or(description.ilike(pattern)),
        );
    }

    if let Some(reg_open) = filter.registration_open {
        count_query = count_query.filter(registration_open.eq(reg_open));
    }

    if let Some(has_rooms) = filter.has_rooms {
        if has_rooms {
            count_query = count_query.filter(public_rooms_count.gt(0));
        } else {
            count_query =
                count_query.filter(public_rooms_count.le(0).or(public_rooms_count.is_null()));
        }
    }

    if let Some(ref pattern) = room_version_pattern {
        count_query = count_query.filter(room_versions.like(pattern));
    }

    let total = count_query.count().get_result::<i64>(conn)?;

    let mut result_query = servers.into_boxed();

    if let Some(ref pattern) = search_pattern {
        result_query = result_query.filter(
            domain
                .ilike(pattern)
                .or(name.ilike(pattern))
                .or(description.ilike(pattern)),
        );
    }

    if let Some(reg_open) = filter.registration_open {
        result_query = result_query.filter(registration_open.eq(reg_open));
    }

    if let Some(has_rooms) = filter.has_rooms {
        if has_rooms {
            result_query = result_query.filter(public_rooms_count.gt(0));
        } else {
            result_query =
                result_query.filter(public_rooms_count.le(0).or(public_rooms_count.is_null()));
        }
    }

    if let Some(ref pattern) = room_version_pattern {
        result_query = result_query.filter(room_versions.like(pattern));
    }

    let result_servers: Vec<Server> = match sort_by {
        "name" => {
            if sort_order == "asc" {
                result_query.order(name.asc())
            } else {
                result_query.order(name.desc())
            }
        }
        "domain" => {
            if sort_order == "asc" {
                result_query.order(domain.asc())
            } else {
                result_query.order(domain.desc())
            }
        }
        "public_rooms_count" => {
            if sort_order == "asc" {
                result_query.order(public_rooms_count.asc())
            } else {
                result_query.order(public_rooms_count.desc())
            }
        }
        _ => {
            if sort_order == "asc" {
                result_query.order(created_at.asc())
            } else {
                result_query.order(created_at.desc())
            }
        }
    }
    .offset(offset as i64)
    .limit(limit as i64)
    .load(conn)?;

    Ok(PaginatedServers {
        servers: result_servers,
        total,
        limit,
        offset,
    })
}

pub fn run_migrations(conn: &mut PgConnection) {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

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
