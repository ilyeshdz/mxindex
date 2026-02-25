use once_cell::sync::Lazy;
use reqwest::Client;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
});

pub fn get_http_client() -> &'static Client {
    &HTTP_CLIENT
}
