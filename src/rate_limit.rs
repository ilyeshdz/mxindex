use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub struct RateLimiterState {
    pub requests_per_minute: u64,
    pub client_requests: Arc<Mutex<HashMap<String, (u64, Instant)>>>,
}

#[allow(dead_code)]
impl RateLimiterState {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute: requests_per_minute as u64,
            client_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub fn check(&self, client_id: &str) -> Result<(), RateLimitError> {
        if self.requests_per_minute == 0 {
            return Ok(());
        }

        let mut requests = self.client_requests.lock().unwrap();
        let now = Instant::now();

        let should_reset = requests
            .get(client_id)
            .map(|(_, start)| start.elapsed() >= Duration::from_secs(60))
            .unwrap_or(true);

        if should_reset {
            requests.insert(client_id.to_string(), (1, now));
            return Ok(());
        }

        let current_count = requests
            .get(client_id)
            .map(|(count, _)| *count)
            .unwrap_or(0);

        if current_count >= self.requests_per_minute {
            return Err(RateLimitError);
        }

        requests.insert(client_id.to_string(), (current_count + 1, now));

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct RateLimitError;

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rate limit exceeded")
    }
}

impl std::error::Error for RateLimitError {}

impl<'r> rocket::response::Responder<'r, 'r> for RateLimitError {
    fn respond_to(self, _req: &rocket::Request) -> rocket::response::Result<'r> {
        rocket::Response::build()
            .status(rocket::http::Status::TooManyRequests)
            .header(rocket::http::Header::new("X-RateLimit-Limit", "60"))
            .header(rocket::http::Header::new("X-RateLimit-Remaining", "0"))
            .header(rocket::http::Header::new(
                "Content-Type",
                "application/json",
            ))
            .sized_body(
                None,
                std::io::Cursor::new(
                    r#"{"error":"rate_limit_exceeded","message":"Too many requests"}"#,
                ),
            )
            .ok()
    }
}

pub fn rate_limiter_from_config() -> Option<RateLimiterState> {
    let requests_per_minute = std::env::var("RATE_LIMIT_PER_MINUTE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);

    if requests_per_minute > 0 {
        Some(RateLimiterState::new(requests_per_minute))
    } else {
        None
    }
}
