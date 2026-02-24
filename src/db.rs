use crate::schema::servers;
use diesel::prelude::*;
use diesel::SqliteConnection;

#[derive(Queryable, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: i32,
    pub domain: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub registration_open: Option<bool>,
    pub public_rooms_count: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = servers)]
pub struct NewServer<'a> {
    pub domain: &'a str,
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub registration_open: Option<bool>,
    pub public_rooms_count: Option<i32>,
}

pub fn establish_connection() -> SqliteConnection {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    if let Some(parent) = std::path::Path::new(&database_url).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn insert_server(
    conn: &mut SqliteConnection,
    new_server: &NewServer,
) -> Result<Server, diesel::result::Error> {
    use crate::schema::servers::dsl::*;

    diesel::insert_into(servers)
        .values(new_server)
        .execute(conn)?;

    servers.order(id.desc()).first(conn)
}

pub fn get_server_by_domain(
    conn: &mut SqliteConnection,
    server_domain: &str,
) -> Result<Option<Server>, diesel::result::Error> {
    use crate::schema::servers::dsl::*;

    servers
        .filter(domain.eq(server_domain))
        .first(conn)
        .optional()
}

pub fn get_all_servers(conn: &mut SqliteConnection) -> Result<Vec<Server>, diesel::result::Error> {
    use crate::schema::servers::dsl::*;

    servers.load(conn)
}

pub fn run_migrations(conn: &mut SqliteConnection) {
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
            registration_open: Some(true),
            public_rooms_count: Some(100),
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
            registration_open: None,
            public_rooms_count: None,
        };

        assert_eq!(new_server.domain, "test.org");
        assert!(new_server.name.is_none());
        assert!(new_server.description.is_none());
    }
}
