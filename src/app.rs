use crate::cache::Cache;
use crate::db::DbPool;
use std::sync::Arc;

pub struct AppState {
    pub cache: Arc<Cache>,
    pub db_pool: DbPool,
}
