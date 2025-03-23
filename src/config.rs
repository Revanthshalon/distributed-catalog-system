pub struct AppConfig {
    pub api_port: u16,
    pub env: String,
    pub instance_id: String,

    // Database Configuration
    pub database_url: String,
    pub database_max_connections: u32,
    pub follower_urls: Vec<String>,
    pub shard_counts: usize,

    // Redis Configuration
    pub redis_url: String,
    pub redis_pool_max_size: u32,
    pub redis_cache_ttl_seconds: u64,

    // RabbitMQ Configuration
    pub rabbitmq_url: String,
    pub rabbitmq_order_queue: String,
    pub rabbitmq_logging_queue: String,

    // Curcuite Breaker Configuration
    pub circuit_breaker_failure_threshold: u32,
    pub circuit_breaker_reset_timeout_ms: u64,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        todo!()
    }
}
