use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct EndpointLabels {
    pub method: String,
    pub endpoint: String,
    pub status: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct CacheLabels {
    pub operation: String,
    pub result: String,
}

pub struct Metrics {
    pub http_requests_total: Family<EndpointLabels, Counter>,
    pub cache_operations: Family<CacheLabels, Counter>,
    pub servers_indexed: Gauge,
    pub servers_online: Gauge,
    pub servers_offline: Gauge,
    pub discovery_errors: Counter,
    pub registry: Registry,
}

impl Metrics {
    pub fn new() -> Arc<RwLock<Metrics>> {
        let mut registry = Registry::default();

        let http_requests_total = Family::default();
        registry.register(
            "http_requests_total",
            "Total number of HTTP requests",
            http_requests_total.clone(),
        );

        let cache_operations = Family::default();
        registry.register(
            "cache_operations_total",
            "Total number of cache operations",
            cache_operations.clone(),
        );

        let servers_indexed = Gauge::default();
        registry.register(
            "servers_indexed",
            "Number of indexed servers",
            servers_indexed.clone(),
        );

        let servers_online = Gauge::default();
        registry.register(
            "servers_online",
            "Number of online servers",
            servers_online.clone(),
        );

        let servers_offline = Gauge::default();
        registry.register(
            "servers_offline",
            "Number of offline servers",
            servers_offline.clone(),
        );

        let discovery_errors = Counter::default();
        registry.register(
            "discovery_errors_total",
            "Total number of discovery errors",
            discovery_errors.clone(),
        );

        Arc::new(RwLock::new(Metrics {
            http_requests_total,
            cache_operations,
            servers_indexed,
            servers_online,
            servers_offline,
            discovery_errors,
            registry,
        }))
    }

    #[allow(dead_code)]
    pub fn increment_http_requests(&self, method: &str, endpoint: &str, status: &str) {
        self.http_requests_total
            .get_or_create(&EndpointLabels {
                method: method.to_string(),
                endpoint: endpoint.to_string(),
                status: status.to_string(),
            })
            .inc();
    }

    #[allow(dead_code)]
    pub fn increment_cache_operations(&self, operation: &str, result: &str) {
        self.cache_operations
            .get_or_create(&CacheLabels {
                operation: operation.to_string(),
                result: result.to_string(),
            })
            .inc();
    }

    #[allow(dead_code)]
    pub fn set_servers_indexed(&self, count: i64) {
        self.servers_indexed.set(count);
    }

    #[allow(dead_code)]
    pub fn set_servers_online(&self, count: i64) {
        self.servers_online.set(count);
    }

    #[allow(dead_code)]
    pub fn set_servers_offline(&self, count: i64) {
        self.servers_offline.set(count);
    }

    #[allow(dead_code)]
    pub fn increment_discovery_errors(&self) {
        self.discovery_errors.inc();
    }

    pub fn encode(&self) -> String {
        let mut output = String::new();

        output.push_str("# HELP http_requests_total Total number of HTTP requests\n");
        output.push_str("# TYPE http_requests_total counter\n");

        output.push_str("# HELP cache_operations_total Total number of cache operations\n");
        output.push_str("# TYPE cache_operations_total counter\n");

        output.push_str("# HELP servers_indexed Number of indexed servers\n");
        output.push_str("# TYPE servers_indexed gauge\n");
        output.push_str(&format!("servers_indexed {}\n", self.servers_indexed.get()));

        output.push_str("# HELP servers_online Number of online servers\n");
        output.push_str("# TYPE servers_online gauge\n");
        output.push_str(&format!("servers_online {}\n", self.servers_online.get()));

        output.push_str("# HELP servers_offline Number of offline servers\n");
        output.push_str("# TYPE servers_offline gauge\n");
        output.push_str(&format!("servers_offline {}\n", self.servers_offline.get()));

        output.push_str("# HELP discovery_errors_total Total number of discovery errors\n");
        output.push_str("# TYPE discovery_errors_total counter\n");
        output.push_str(&format!(
            "discovery_errors_total {}\n",
            self.discovery_errors.get()
        ));

        output
    }
}

impl Default for Metrics {
    fn default() -> Self {
        panic!("Metrics::new() must be called before using Metrics")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        let metrics = Metrics::new();
        let guard = metrics.try_read();
        assert!(guard.is_ok());
    }

    #[test]
    fn test_encode_output() {
        let metrics = Metrics::new();
        let guard = metrics.try_read();
        let metrics = guard.unwrap();
        let output = metrics.encode();
        assert!(output.contains("servers_indexed"));
    }
}
