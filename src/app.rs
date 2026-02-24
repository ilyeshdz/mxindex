use crate::cache::Cache;
use std::sync::Arc;

pub struct AppState {
    pub cache: Arc<Cache>,
}
