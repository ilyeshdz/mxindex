use redis::{aio::ConnectionManager, Client, RedisError};
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Cache not found")]
    NotFound,
    #[error("Connection not initialized")]
    NotInitialized,
}

pub struct Cache {
    connection: Arc<RwLock<Option<ConnectionManager>>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn connect(&self, url: &str) -> Result<(), CacheError> {
        let client = Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;
        let mut conn = self.connection.write().await;
        *conn = Some(manager);
        Ok(())
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, CacheError> {
        let mut guard = self.connection.write().await;
        let conn = guard.as_mut().ok_or(CacheError::NotInitialized)?;
        
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(data) => {
                let parsed: T = serde_json::from_str(&data)?;
                Ok(parsed)
            }
            None => Err(CacheError::NotFound),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: usize) -> Result<(), CacheError> {
        let mut guard = self.connection.write().await;
        let conn = guard.as_mut().ok_or(CacheError::NotInitialized)?;
        
        let data = serde_json::to_string(value)?;
        
        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(data)
            .arg("EX")
            .arg(ttl_seconds as u64)
            .query_async(conn)
            .await?;
        
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut guard = self.connection.write().await;
        let conn = guard.as_mut().ok_or(CacheError::NotInitialized)?;
        
        let result: usize = conn.del(key).await?;
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut guard = self.connection.write().await;
        let conn = guard.as_mut().ok_or(CacheError::NotInitialized)?;
        
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        let mut guard = self.connection.write().await;
        let conn = guard.as_mut().ok_or(CacheError::NotInitialized)?;
        
        let mut keys = Vec::new();
        let mut cursor = 0i64;
        
        loop {
            let (next_cursor, batch): (i64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(conn)
                .await?;
            
            keys.extend(batch);
            cursor = next_cursor;
            
            if cursor == 0 {
                break;
            }
        }
        
        if !keys.is_empty() {
            let _: () = redis::cmd("DEL")
                .arg(keys.as_slice())
                .query_async(conn)
                .await?;
        }
        
        Ok(())
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn cache_key(prefix: &str, parts: &[&str]) -> String {
    let mut key = prefix.to_string();
    for part in parts {
        key.push(':');
        key.push_str(part);
    }
    key
}
