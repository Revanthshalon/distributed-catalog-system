# Production-Ready Distributed E-commerce System in Rust

I'll create a comprehensive distributed e-commerce system that meets all your requirements. This implementation will showcase key distributed system concepts while maintaining production-grade quality.

## Project Structure

Here's the complete project structure we'll implement:

```
distributed-ecommerce/
├── Cargo.toml
├── .cargo/
│   └── config.toml
├── docker/
│   ├── api/Dockerfile
│   └── db/init-shards.sh
├── docker-compose.yml
├── migrations/
│   ├── 20250301000000_create_products.sql
│   └── 20250301000001_create_orders.sql
├── seed/
│   └── seed-data.sql
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── product.rs
│   │   └── order.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── connection.rs
│   │   ├── sharding.rs
│   │   └── replication.rs
│   ├── cache/
│   │   ├── mod.rs
│   │   └── redis.rs
│   ├── queue/
│   │   ├── mod.rs
│   │   └── rabbitmq.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── product.rs
│   │   └── order.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── product_service.rs
│   │   └── order_service.rs
│   ├── telemetry/
│   │   ├── mod.rs
│   │   ├── metrics.rs
│   │   └── tracing.rs
│   └── utils/
│       ├── mod.rs
│       ├── circuit_breaker.rs
│       ├── consistent_hash.rs
│       └── health.rs
└── README.md
```

Let's start implementing each file:

## Cargo.toml

```toml name=Cargo.toml
[package]
name = "distributed-ecommerce"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A production-ready distributed e-commerce system"

[dependencies]
# Web framework
axum = "0.7.3"
tower = "0.4.13"
tower-http = { version = "0.4.4", features = ["trace", "cors", "timeout", "metrics"] }
tokio = { version = "1.35.1", features = ["full"] }
hyper = { version = "1.1.0", features = ["full"] }

# Database
sqlx = { version = "0.7.3", features = ["runtime-tokio-rustls", "postgres", "macros", "json", "uuid", "time", "migrate"] }
postgres-types = "0.2.6"
deadpool-postgres = "0.11.0"
tokio-postgres = { version = "0.7.10", features = ["with-uuid-1", "with-serde_json-1"] }

# Caching
redis = { version = "0.24.0", features = ["tokio-comp", "connection-manager"] }
deadpool-redis = "0.14.0"

# Message queue
lapin = "2.3.1"
deadpool-lapin = "0.11.0"

# Consistent hashing
twox-hash = "1.6.3"
consistent_hash_ring = "0.8.0"

# Circuit breaking
failsafe = "1.2.0"

# Serialization
serde = { version = "1.0.195", features = ["derive"] }
serde_json = "1.0.111"

# Error handling
thiserror = "1.0.56"
anyhow = "1.0.79"

# Logging and metrics
tracing = "0.1.40"
tracing-subscriber = { version = "0.3.18", features = ["env-filter"] }
tracing-opentelemetry = "0.22.0"
opentelemetry = { version = "0.21.0" }
opentelemetry_sdk = { version = "0.21.1", features = ["metrics", "rt-tokio"] }
opentelemetry-prometheus = "0.14.0"
prometheus = "0.13.3"

# Configuration
config = "0.13.4"
dotenv = "0.15.0"
uuid = { version = "1.6.1", features = ["serde", "v4"] }
chrono = { version = "0.4.31", features = ["serde"] }

# Utilities
async-trait = "0.1.77"
futures = "0.3.30"
rand = "0.8.5"
tokio-util = "0.7.10"
once_cell = "1.19.0"

[dev-dependencies]
tokio-test = "0.4.3"
mockall = "0.12.1"
```

## Config File

```rust name=src/config.rs
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub shutdown_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseShardConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
    pub is_leader: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub shards: Vec<DatabaseShardConfig>,
    pub shard_count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub ttl_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQConfig {
    pub url: String,
    pub order_queue: String,
    pub logging_queue: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryConfig {
    pub tracing_level: String,
    pub metrics_port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub rabbitmq: RabbitMQConfig,
    pub telemetry: TelemetryConfig,
    pub circuit_breaker: CircuitBreakerConfig,
    pub environment: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let mut builder = Config::builder()
            // Start with default values
            .add_source(File::with_name("config/default"))
            // Add environment-specific values
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add local overrides
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables (with a prefix of APP)
            .add_source(Environment::with_prefix("APP").separator("__"));

        // Build and convert
        let config = builder.build()?;
        config.try_deserialize()
    }
}

pub type AppConfig = Arc<Settings>;

pub fn load_config() -> AppConfig {
    // Load .env file if it exists
    dotenv::dotenv().ok();
    
    match Settings::new() {
        Ok(settings) => Arc::new(settings),
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Error Handling

```rust name=src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis cache error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("RabbitMQ error: {0}")]
    RabbitMQ(#[from] lapin::Error),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
    
    #[error("Request timeout: {0}")]
    Timeout(String),
    
    #[error("Shard unavailable: {0}")]
    ShardUnavailable(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::CircuitBreakerOpen(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::Timeout(msg) => (StatusCode::REQUEST_TIMEOUT, msg.clone()),
            AppError::ShardUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::Database(err) => {
                error!(error = %err, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
            },
            AppError::Redis(err) => {
                error!(error = %err, "Redis error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Cache error".into())
            },
            AppError::RabbitMQ(err) => {
                error!(error = %err, "RabbitMQ error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Message queue error".into())
            },
            AppError::Internal(msg) => {
                error!(error = %msg, "Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            },
            AppError::Serialization(err) => {
                error!(error = %err, "Serialization error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error".into())
            },
        };

        // Don't expose detailed internal errors in production
        let error_response = ErrorResponse {
            status: status_code.to_string(),
            message: error_message,
            details: if status_code == StatusCode::INTERNAL_SERVER_ERROR {
                None
            } else {
                Some(self.to_string())
            },
        };

        (status_code, Json(json!(error_response))).into_response()
    }
}
```

## Database Sharding & Consistent Hashing

```rust name=src/utils/consistent_hash.rs
use consistent_hash_ring::{ConsistentHashRing, Node};
use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

pub struct DbShard {
    pub id: u32,
    pub is_leader: bool,
    pub connection_string: String,
}

impl Node for DbShard {
    fn name(&self) -> &str {
        &self.connection_string
    }
    
    fn weight(&self) -> usize {
        if self.is_leader {
            200 // Leader nodes have higher weight
        } else {
            100
        }
    }
}

pub struct ConsistentHashSharding {
    ring: ConsistentHashRing<DbShard>,
    shard_count: usize,
}

impl ConsistentHashSharding {
    pub fn new(shards: Vec<DbShard>, virtual_nodes: usize) -> Self {
        let mut ring = ConsistentHashRing::new();
        
        // Add each shard to the ring with virtual nodes for better distribution
        for shard in shards.iter() {
            ring.add(shard.clone(), virtual_nodes);
        }
        
        Self {
            ring,
            shard_count: shards.len(),
        }
    }
    
    pub fn get_shard_for_key<K: Hash>(&self, key: K) -> Option<&DbShard> {
        // Calculate hash of the key
        let mut hasher = XxHash64::default();
        key.hash(&mut hasher);
        let hash_value = hasher.finish();
        
        // Find the appropriate shard
        self.ring.get_node(&hash_value.to_string())
    }
    
    pub fn get_all_leader_shards(&self) -> Vec<&DbShard> {
        self.ring
            .nodes()
            .filter(|shard| shard.is_leader)
            .collect()
    }
    
    pub fn get_shard_count(&self) -> usize {
        self.shard_count
    }
}
```

## Database Connection Management

```rust name=src/db/connection.rs
use crate::{
    config::AppConfig,
    error::AppError,
    utils::consistent_hash::{ConsistentHashSharding, DbShard},
};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::NoTls;
use tracing::{error, info};

#[derive(Clone)]
pub struct DbConnectionManager {
    sharding: Arc<ConsistentHashSharding>,
    shard_pools: Arc<HashMap<u32, Pool>>,
    config: AppConfig,
}

impl DbConnectionManager {
    pub async fn new(config: AppConfig) -> Result<Self, AppError> {
        let mut shards = Vec::new();
        let mut shard_pools = HashMap::new();
        
        // Create connection pool for each shard
        for (idx, shard_config) in config.database.shards.iter().enumerate() {
            let shard_id = idx as u32;
            
            let pg_config = Config {
                host: Some(shard_config.host.clone()),
                port: Some(shard_config.port),
                user: Some(shard_config.username.clone()),
                password: Some(shard_config.password.clone()),
                dbname: Some(shard_config.database.clone()),
                manager: Some(ManagerConfig {
                    recycling_method: RecyclingMethod::Fast,
                }),
                pool_size: shard_config.pool_size,
                ..Config::default()
            };
            
            // Create connection pool
            let pool = pg_config.create_pool(Some(Runtime::Tokio1), NoTls)?;
            
            // Test connection
            match pool.get().await {
                Ok(_) => info!("Successfully connected to database shard {}", shard_id),
                Err(err) => {
                    error!("Failed to connect to database shard {}: {}", shard_id, err);
                    return Err(AppError::Database(err.into()));
                }
            }
            
            // Create shard configuration
            let connection_string = format!(
                "postgres://{}:{}@{}:{}/{}",
                shard_config.username,
                "***", // Hide password in connection string
                shard_config.host,
                shard_config.port,
                shard_config.database
            );
            
            let shard = DbShard {
                id: shard_id,
                is_leader: shard_config.is_leader,
                connection_string,
            };
            
            shards.push(shard);
            shard_pools.insert(shard_id, pool);
        }
        
        // Create consistent hash ring with 100 virtual nodes per physical node
        let sharding = ConsistentHashSharding::new(shards, 100);
        
        Ok(Self {
            sharding: Arc::new(sharding),
            shard_pools: Arc::new(shard_pools),
            config,
        })
    }
    
    pub fn get_pool_for_key<K: std::hash::Hash>(&self, key: K) -> Result<Pool, AppError> {
        match self.sharding.get_shard_for_key(key) {
            Some(shard) => self
                .shard_pools
                .get(&shard.id)
                .cloned()
                .ok_or_else(|| AppError::Internal(format!("Shard pool not found for ID {}", shard.id))),
            None => Err(AppError::ShardUnavailable("No shard available".into())),
        }
    }
    
    pub fn get_all_leader_pools(&self) -> Vec<Pool> {
        self.sharding
            .get_all_leader_shards()
            .iter()
            .filter_map(|shard| self.shard_pools.get(&shard.id).cloned())
            .collect()
    }
    
    pub fn get_all_pools(&self) -> Vec<Pool> {
        self.shard_pools.values().cloned().collect()
    }
}
```

## Database Sharding Implementation

```rust name=src/db/sharding.rs
use crate::{
    db::connection::DbConnectionManager,
    error::AppError,
    models::{order::Order, product::Product},
};
use serde::Serialize;
use sqlx::{Pool, Postgres};
use std::hash::{Hash, Hasher};
use tracing::instrument;
use uuid::Uuid;

// Trait for entities that can be sharded
pub trait Shardable: Hash + Send + Sync {
    fn get_shard_key(&self) -> ShardKey;
}

// Keys used for sharding decisions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShardKey {
    UserId(Uuid),
    ProductId(Uuid),
    OrderId(Uuid),
    ProductCategory(String),
    Raw(String),
}

impl From<&str> for ShardKey {
    fn from(s: &str) -> Self {
        ShardKey::Raw(s.to_owned())
    }
}

impl From<String> for ShardKey {
    fn from(s: String) -> Self {
        ShardKey::Raw(s)
    }
}

impl From<Uuid> for ShardKey {
    fn from(id: Uuid) -> Self {
        ShardKey::OrderId(id)
    }
}

// ShardedRepo provides CRUD operations across sharded databases
pub struct ShardedRepo<T: Shardable + Serialize + Send + Sync> {
    conn_manager: DbConnectionManager,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Shardable + Serialize + Send + Sync> ShardedRepo<T> {
    pub fn new(conn_manager: DbConnectionManager) -> Self {
        Self {
            conn_manager,
            _marker: std::marker::PhantomData,
        }
    }

    // Helper to get the appropriate connection pool for a given shard key
    #[instrument(skip(self))]
    pub async fn get_pool_for_key(&self, key: &ShardKey) -> Result<Pool<Postgres>, AppError> {
        self.conn_manager.get_pool_for_key(key)
    }
    
    // Helper to get all leader pools for operations that need to go across all shards
    pub fn get_all_leader_pools(&self) -> Vec<Pool<Postgres>> {
        self.conn_manager.get_all_leader_pools()
    }
}

// Implementation for Product entity
impl Shardable for Product {
    fn get_shard_key(&self) -> ShardKey {
        match &self.category {
            Some(category) => ShardKey::ProductCategory(category.clone()),
            None => ShardKey::ProductId(self.id),
        }
    }
}

// Implementation for Order entity  
impl Shardable for Order {
    fn get_shard_key(&self) -> ShardKey {
        // Orders are sharded by customer_id for efficiency in customer-specific queries
        ShardKey::UserId(self.customer_id)
    }
}
```

## Database Replication

```rust name=src/db/replication.rs
use crate::{
    config::AppConfig,
    db::connection::DbConnectionManager,
    error::AppError,
    queue::rabbitmq::RabbitMQClient,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, query, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, instrument};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationEvent {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sql: String,
    pub params: Option<serde_json::Value>,
    pub source_shard: u32,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: String,
}

impl ReplicationEvent {
    pub fn new(
        sql: String,
        params: Option<serde_json::Value>,
        source_shard: u32,
        entity_type: String,
        entity_id: String,
        operation: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            sql,
            params,
            source_shard,
            entity_type,
            entity_id,
            operation,
        }
    }
}

#[derive(Clone)]
pub struct ReplicationManager {
    conn_manager: DbConnectionManager,
    rabbitmq_client: Arc<RabbitMQClient>,
    replication_queue: String,
    event_buffer: Arc<Mutex<Vec<ReplicationEvent>>>,
}

impl ReplicationManager {
    pub async fn new(
        conn_manager: DbConnectionManager,
        rabbitmq_client: Arc<RabbitMQClient>,
        config: AppConfig,
    ) -> Self {
        // Replication events queue name
        let replication_queue = "db_replication_events".to_string();
        
        let manager = Self {
            conn_manager,
            rabbitmq_client,
            replication_queue,
            event_buffer: Arc::new(Mutex::new(Vec::new())),
        };
        
        // Start background worker for processing replication events
        manager.start_replication_worker().await;
        
        manager
    }
    
    #[instrument(skip(self, transaction))]
    pub async fn record_write_operation<'a>(
        &self,
        transaction: &mut Transaction<'a, Postgres>,
        sql: String,
        params: Option<serde_json::Value>,
        source_shard: u32,
        entity_type: String,
        entity_id: String,
        operation: String,
    ) -> Result<(), AppError> {
        // Create replication event
        let event = ReplicationEvent::new(
            sql, params, source_shard, entity_type, entity_id, operation
        );
        
        // Insert event to replication log table within the same transaction
        query!("
            INSERT INTO replication_log 
            (id, timestamp, sql_statement, params, source_shard, entity_type, entity_id, operation)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ",
            event.id,
            event.timestamp,
            event.sql,
            event.params,
            event.source_shard as i32,
            event.entity_type,
            event.entity_id,
            event.operation
        )
        .execute(transaction)
        .await?;
        
        // Add to buffer for async publishing after transaction commit
        self.event_buffer.lock().await.push(event);
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn publish_buffered_events(&self) -> Result<(), AppError> {
        let events = {
            let mut buffer = self.event_buffer.lock().await;
            std::mem::take(&mut *buffer)
        };
        
        if events.is_empty() {
            return Ok(());
        }
        
        for event in events {
            let payload = serde_json::to_vec(&event)?;
            self.rabbitmq_client
                .publish(&self.replication_queue, &payload)
                .await?;
        }
        
        Ok(())
    }
    
    // Start a background worker to process replication events
    async fn start_replication_worker(&self) {
        let (tx, mut rx) = mpsc::channel(100);
        
        let rabbitmq_client = self.rabbitmq_client.clone();
        let replication_queue = self.replication_queue.clone();
        let conn_manager = self.conn_manager.clone();
        
        // Spawn consumer thread
        tokio::spawn(async move {
            info!("Starting replication worker");
            
            match rabbitmq_client.consume(&replication_queue, tx).await {
                Ok(_) => info!("Replication consumer started successfully"),
                Err(e) => error!("Failed to start replication consumer: {}", e),
            }
        });
        
        // Spawn processor thread
        let conn_manager_clone = conn_manager.clone();
        tokio::spawn(async move {
            info!("Starting replication processor");
            
            while let Some(delivery) = rx.recv().await {
                match process_replication_event(&conn_manager_clone, &delivery.data).await {
                    Ok(_) => {
                        if let Err(e) = delivery.ack().await {
                            error!("Failed to acknowledge message: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to process replication event: {}", e);
                        // Negative acknowledgment with requeue
                        if let Err(e) = delivery.nack(false, true).await {
                            error!("Failed to negatively acknowledge message: {}", e);
                        }
                    }
                }
            }
        });
    }
}

#[instrument(skip(conn_manager, data))]
async fn process_replication_event(
    conn_manager: &DbConnectionManager,
    data: &[u8],
) -> Result<PgQueryResult, AppError> {
    let event: ReplicationEvent = serde_json::from_slice(data)?;
    
    info!(
        "Processing replication event: type={}, id={}, operation={}",
        event.entity_type, event.entity_id, event.operation
    );
    
    // Get all follower pools (non-leader pools)
    let leader_pools = conn_manager.get_all_leader_pools();
    let all_pools = conn_manager.get_all_pools();
    
    let follower_pools: Vec<_> = all_pools
        .into_iter()
        .filter(|pool| !leader_pools.contains(pool))
        .collect();
    
    // Apply the operation to all follower shards
    for pool in follower_pools {
        let mut conn = pool.acquire().await?;
        
        // Execute the SQL with parameters if available
        let result = match &event.params {
            Some(params) => {
                // This is a simplified approach - in a real system you'd need 
                // to handle parameter binding more robustly
                let query = sqlx::query(&event.sql);
                query.bind(params).execute(&mut *conn).await?
            }
            None => {
                sqlx::query(&event.sql).execute(&mut *conn).await?
            }
        };
        
        info!(
            "Applied replication event: type={}, id={}, rows={}",
            event.entity_type, event.entity_id, result.rows_affected()
        );
    }
    
    // Return the result from the last execution
    Ok(sqlx::query("SELECT 1").execute(&leader_pools[0]).await?)
}
```

## Redis Cache Layer

```rust name=src/cache/redis.rs
use crate::{config::AppConfig, error::AppError};
use deadpool_redis::{Config as RedisConfig, Connection, Pool, Runtime};
use serde::{de::DeserializeOwned, Serialize};
use std::{sync::Arc, time::Duration};
use tracing::{error, info, instrument};

#[derive(Clone)]
pub struct RedisCache {
    pool: Pool,
    ttl: Duration,
}

impl RedisCache {
    pub async fn new(config: AppConfig) -> Result<Self, AppError> {
        let redis_config = RedisConfig::from_url(config.redis.url.clone());
        let pool = redis_config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| AppError::Redis(redis::RedisError::from((redis::ErrorKind::IoError, e.to_string()))))?;
        
        // Verify connection
        let mut conn = pool.get().await.map_err(|e| {
            AppError::Redis(redis::RedisError::from((redis::ErrorKind::IoError, e.to_string())))
        })?;
        
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(AppError::Redis)?;
            
        info!("Successfully connected to Redis");
        
        Ok(Self {
            pool,
            ttl: Duration::from_secs(config.redis.ttl_seconds),
        })
    }
    
    async fn get_connection(&self) -> Result<Connection, AppError> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::Redis(redis::RedisError::from((redis::ErrorKind::IoError, e.to_string()))))
    }
    
    #[instrument(skip(self, value))]
    pub async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl_override: Option<Duration>,
    ) -> Result<(), AppError> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)?;
        
        let ttl = ttl_override.unwrap_or(self.ttl);
        
        redis::cmd("SET")
            .arg(key)
            .arg(serialized)
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(AppError::Redis)
    }
    
    #[instrument(skip(self))]
    pub async fn get<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>, AppError> {
        let mut conn = self.get_connection().await?;
        
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(AppError::Redis)?;
            
        match result {
            Some(data) => {
                let deserialized = serde_json::from_str(&data)?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }
    
    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.get_connection().await?;
        
        redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(AppError::Redis)?;
            
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn invalidate_by_pattern(&self, pattern: &str) -> Result<(), AppError> {
        let mut conn = self.get_connection().await?;
        
        // Get all keys matching pattern
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .map_err(AppError::Redis)?;
            
        if !keys.is_empty() {
            // Delete all matched keys
            redis::cmd("DEL")
                .arg(keys)
                .query_async(&mut conn)
                .await
                .map_err(AppError::Redis)?;
        }
        
        Ok(())
    }
}

// Distributed cache implementation with Redis
#[derive(Clone)]
pub struct DistributedCache {
    redis: Arc<RedisCache>,
}

impl DistributedCache {
    pub fn new(redis: RedisCache) -> Self {
        Self {
            redis: Arc::new(redis),
        }
    }
    
    // Get a cached value or compute and cache it if not found
    #[instrument(skip(self, compute_fn))]
    pub async fn get_or_compute<T, F, Fut>(
        &self,
        key: &str,
        compute_fn: F,
        ttl_override: Option<Duration>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned + Serialize + Send + Sync,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        // Try to get from cache first
        if let Some(cached) = self.redis.get::<T>(key).await? {
            return Ok(cached);
        }
        
        // If not found, compute the value
        let computed = compute_fn().await?;
        
        // Cache the result
        self.redis.set(key, &computed, ttl_override).await?;
        
        Ok(computed)
    }
    
    pub async fn invalidate(&self, key: &str) -> Result<(), AppError> {
        self.redis.delete(key).await
    }
    
    pub async fn invalidate_by_pattern(&self, pattern: &str) -> Result<(), AppError> {
        self.redis.invalidate_by_pattern(pattern).await
    }
}
```

## RabbitMQ Implementation

```rust name=src/queue/rabbitmq.rs
use crate::{config::AppConfig, error::AppError};
use deadpool_lapin::{
    Config as LapinConfig, Manager as LapinManager, Pool as LapinPool, Runtime,
};
use futures::{StreamExt, TryStreamExt};
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions,
        ExchangeDeclareOptions, QueueDeclareOptions,
    },
    publisher_confirm::Confirmation,
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties, ExchangeKind,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument};

pub struct DeliveryHandler {
    pub data: Vec<u8>,
    inner: lapin::message::Delivery,
    channel: lapin::Channel,
}

impl DeliveryHandler {
    pub async fn ack(&self) -> Result<(), lapin::Error> {
        self.inner.ack(BasicAckOptions::default()).await
    }

    pub async fn nack(&self, requeue: bool, multiple: bool) -> Result<(), lapin::Error> {
        self.inner
            .nack(BasicNackOptions {
                multiple,
                requeue,
            })
            .await
    }
}

#[derive(Clone)]
pub struct RabbitMQClient {
    pool: Arc<LapinPool>,
    exchanges: Arc<Vec<String>>,
}

impl RabbitMQClient {
    pub async fn new(config: AppConfig) -> Result<Self, AppError> {
        // Define exchanges
        let exchanges = vec![
            "orders".to_string(),
            "logs".to_string(),
            "events".to_string(),
        ];

        // Create connection pool
        let manager = LapinManager::new(
            config.rabbitmq.url.clone(),
            ConnectionProperties::default(),
        );
        
        let pool = LapinConfig {
            max_size: 10,
            timeouts: None,
        }
        .create_pool(manager, Runtime::Tokio1)
        .map_err(|e| AppError::RabbitMQ(lapin::Error::IOError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))))?;

        let client = Self {
            pool: Arc::new(pool),
            exchanges: Arc::new(exchanges.clone()),
        };

        // Set up exchanges
        client.setup_exchanges().await?;

        Ok(client)
    }

    async fn setup_exchanges(&self) -> Result<(), AppError> {
        let conn = self.get_connection().await?;
        let channel = conn.create_channel().await?;

        // Create exchanges
        for exchange in self.exchanges.iter() {
            channel
                .exchange_declare(
                    exchange,
                    ExchangeKind::Topic,
                    ExchangeDeclareOptions {
                        durable: true,
                        ..ExchangeDeclareOptions::default()
                    },
                    FieldTable::default(),
                )
                .await?;
        }

        info!("RabbitMQ exchanges set up successfully");
        Ok(())
    }

    async fn get_connection(&self) -> Result<Connection, AppError> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| {
                AppError::RabbitMQ(lapin::Error::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )))
            })?;

        Ok(conn.clone())
    }

    #[instrument(skip(self, payload))]
    pub async fn publish(
        &self,
        queue: &str,
        payload: &[u8],
    ) -> Result<Confirmation, AppError> {
        let conn = self.get_connection().await?;
        let channel = conn.create_channel().await?;

        // Declare queue to ensure it exists
        channel
            .queue_declare(
                queue,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await?;

        // Publish message
        let confirmation = channel
            .basic_publish(
                "", // Default exchange
                queue,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default().with_delivery_mode(2), // Persistent delivery
            )
            .await?
            .await?;

        debug!("Published message to queue {}", queue);
        Ok(confirmation)
    }

    #[instrument(skip(self, tx))]
    pub async fn consume(
        &self,
        queue: &str,
        tx: mpsc::Sender<DeliveryHandler>,
    ) -> Result<(), AppError> {
        let conn = self.get_connection().await?;
        let channel = conn.create_channel().await?;

        // Declare queue to ensure it exists
        channel
            .queue_declare(
                queue,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await?;

        // Consume messages
        let mut consumer = channel
            .basic_consume(
                queue,
                &format!("consumer-{}", uuid::Uuid::new_v4()),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // Process messages
        let channel_clone = channel.clone();
        tokio::spawn(async move {
            info!("Started consuming from queue: {}", queue);

            while let Some(delivery_result) = consumer.next().await {
                match delivery_result {
                    Ok(delivery) => {
                        let handler = DeliveryHandler {
                            data: delivery.data.clone(),
                            inner: delivery,
                            channel: channel_clone.clone(),
                        };

                        if let Err(e) = tx.send(handler).await {
                            error!("Failed to send message to handler: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error consuming message: {}", e);
                    }
                }
            }

            info!("Stopped consuming from queue: {}", queue);
        });

        Ok(())
    }
}
```

## Product Model

```rust name=src/models/product.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: i64, // Stored as cents
    pub category: Option<String>,
    pub inventory_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductDto {
    pub name: String,
    pub description: Option<String>,
    pub price: i64, // Stored as cents
    pub category: Option<String>,
    pub inventory_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProductDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub category: Option<String>,
    pub inventory_count: Option<i32>,
}

impl Product {
    pub fn new(dto: CreateProductDto) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: dto.name,
            description: dto.description,
            price: dto.price,
            category: dto.category,
            inventory_count: dto.inventory_count,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_update(&mut self, dto: UpdateProductDto) {
        if let Some(name) = dto.name {
            self.name = name;
        }
        
        if let Some(description) = dto.description {
            self.description = Some(description);
        }
        
        if let Some(price) = dto.price {
            self.price = price;
        }
        
        if let Some(category) = dto.category {
            self.category = Some(category);
        }
        
        if let Some(inventory_count) = dto.inventory_count {
            self.inventory_count = inventory_count;
        }
        
        self.updated_at = Utc::now();
    }
}
```

## Order Model

```rust name=src/models/order.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub status: OrderStatus,
    pub total_amount: i64, // Stored as cents
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub price_at_time: i64, // Stored as cents
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
pub enum OrderStatus {
    Created,
    Processing,
    Shipped,
    Delivered,
    Canceled,
    Returned,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderDto {
    pub customer_id: Uuid,
    pub items: Vec<CreateOrderItemDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderItemDto {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrderStatusDto {
    pub status: OrderStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderWithItems {
    pub order: Order,
    pub items: Vec<OrderItem>,
}

impl Order {
    pub fn new(customer_id: Uuid, total_amount: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            customer_id,
            status: OrderStatus::Created,
            total_amount,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_status(&mut self, status: OrderStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl OrderItem {
    pub fn new(order_id: Uuid, product_id: Uuid, quantity: i32, price: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            order_id,
            product_id,
            quantity,
            price_at_time: price,
            created_at: Utc::now(),
        }
    }
}
```

## Product Service

```rust name=src/services/product_service.rs
use crate::{
    cache::redis::DistributedCache,
    db::{connection::DbConnectionManager, sharding::ShardKey},
    error::AppError,
    models::product::{CreateProductDto, Product, UpdateProductDto},
    utils::circuit_breaker::CircuitBreaker,
};
use sqlx::{query, query_as, PgPool};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

pub struct ProductService {
    db: DbConnectionManager,
    cache: Arc<DistributedCache>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl ProductService {
    pub fn new(
        db: DbConnectionManager,
        cache: Arc<DistributedCache>,
        circuit_breaker: Arc<CircuitBreaker>,
    ) -> Self {
        Self {
            db,
            cache,
            circuit_breaker,
        }
    }

    #[instrument(skip(self, dto))]
    pub async fn create_product(&self, dto: CreateProductDto) -> Result<Product, AppError> {
        let product = Product::new(dto);
        
        // Determine the appropriate shard key based on product category
        let shard_key = match &product.category {
            Some(category) => ShardKey::ProductCategory(category.clone()),
            None => ShardKey::ProductId(product.id),
        };
        
        // Get the appropriate connection pool
        let pool = self.db.get_pool_for_key(&shard_key).await?;
        
        // Use circuit breaker for database operation
        self.circuit_breaker
            .call(
                || Self::insert_product_to_db(pool.clone(), &product),
                "insert_product".to_string(),
            )
            .await?;
        
        // Invalidate category cache if product has a category
        if let Some(category) = &product.category {
            self.cache
                .invalidate_by_pattern(&format!("products:category:{}:*", category))
                .await?;
        }
        
        info!("Created product: {}", product.id);
        Ok(product)
    }

    #[instrument(skip(self))]
    pub async fn get_product(&self, id: Uuid) -> Result<Product, AppError> {
        let cache_key = format!("products:id:{}", id);
        
        // Try to get from cache or compute
        self.cache
            .get_or_compute(&cache_key, || async {
                let shard_key = ShardKey::ProductId(id);
                let pool = self.db.get_pool_for_key(&shard_key).await?;
                
                // Use circuit breaker for database operation
                self.circuit_breaker
                    .call(
                        || async {
                            query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
                                .bind(id)
                                .fetch_optional(&pool)
                                .await
                                .map_err(AppError::Database)
                        },
                        "get_product".to_string(),
                    )
                    .await?
                    .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", id)))
            }, None)
            .await
    }

    #[instrument(skip(self))]
    pub async fn get_products_by_category(
        &self,
        category: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Product>, AppError> {
        let cache_key = format!("products:category:{}:{}:{}", category, limit, offset);
        
        // Try to get from cache or compute
        self.cache
            .get_or_compute(&cache_key, || async {
                let shard_key = ShardKey::ProductCategory(category.to_string());
                let pool = self.db.get_pool_for_key(&shard_key).await?;
                
                // Use circuit breaker for database operation
                self.circuit_breaker
                    .call(
                        || async {
                            query_as::<_, Product>(
                                "SELECT * FROM products WHERE category = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                            )
                            .bind(category)
                            .bind(limit)
                            .bind(offset)
                            .fetch_all(&pool)
                            .await
                            .map_err(AppError::Database)
                        },
                        "get_products_by_category".to_string(),
                    )
                    .await
            }, None)
            .await
    }

    #[instrument(skip(self))]
    pub async fn list_products(&self, limit: i64, offset: i64) -> Result<Vec<Product>, AppError> {
        let cache_key = format!("products:list:{}:{}", limit, offset);
        
        // Try to get from cache or compute
        self.cache
            .get_or_compute(&cache_key, || async {
                let mut all_products = Vec::new();
                
                // Query all leader shards
                for pool in self.db.get_all_leader_pools() {
                    match self.circuit_breaker
                        .call(
                            || async {
                                query_as::<_, Product>(
                                    "SELECT * FROM products ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                                )
                                .bind(limit)
                                .bind(offset)
                                .fetch_all(&pool)
                                .await
                                .map_err(AppError::Database)
                            },
                            "list_products".to_string(),
                        )
                        .await
                    {
                        Ok(products) => all_products.extend(products),
                        Err(e) => {
                            // Log error but continue with other shards
                            tracing::warn!("Error fetching products from a shard: {}", e);
                        }
                    }
                }
                
                // Sort all products by created_at DESC and apply limit/offset
                all_products.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                
                let end = (limit as usize).min(all_products.len());
                Ok(all_products.into_iter().take(end).collect())
            }, None)
            .await
    }

    #[instrument(skip(self, dto))]
    pub async fn update_product(
        &self,
        id: Uuid,
        dto: UpdateProductDto,
    ) -> Result<Product, AppError> {
        // Get existing product to determine correct shard
        let mut product = self.get_product(id).await?;
        
        // Apply updates
        product.apply_update(dto);
        
        // Determine shard key
        let shard_key = match &product.category {
            Some(category) => ShardKey::ProductCategory(category.clone()),
            None => ShardKey::ProductId(id),
        };
        
        let pool = self.db.get_pool_for_key(&shard_key).await?;
        
        // Use circuit breaker for database operation
        self.circuit_breaker
            .call(
                || async {
                    query!(
                        "UPDATE products SET 
                            name = $1, 
                            description = $2, 
                            price = $3, 
                            category = $4, 
                            inventory_count = $5,
                            updated_at = $6
                        WHERE id = $7",
                        product.name,
                        product.description,
                        product.price,
                        product.category,
                        product.inventory_count,
                        product.updated_at,
                        id
                    )
                    .execute(&pool)
                    .await
                    .map_err(AppError::Database)
                },
                "update_product".to_string(),
            )
            .await?;
            
        // Invalidate caches
        self.cache.invalidate(&format!("products:id:{}", id)).await?;
        
        // Invalidate category caches if needed
        if let Some(category) = &product.category {
            self.cache
                .invalidate_by_pattern(&format!("products:category:{}:*", category))
                .await?;
        }
        
        // Invalidate list caches
        self.cache.invalidate_by_pattern("products:list:*").await?;
        
        info!("Updated product: {}", product.id);
        Ok(product)
    }

    #[instrument(skip(self))]
    pub async fn delete_product(&self, id: Uuid) -> Result<(), AppError> {
        // Get existing product to determine correct shard
        let product = self.get_product(id).await?;
        
        // Determine shard key
        let shard_key = match &product.category {
            Some(category) => ShardKey::ProductCategory(category.clone()),
            None => ShardKey::ProductId(id),
        };
        
        let pool = self.db.get_pool_for_key(&shard_key).await?;
        
        // Use circuit breaker for database operation
        self.circuit_breaker
            .call(
                || async {
                    query!("DELETE FROM products WHERE id = $1", id)
                        .execute(&pool)
                        .await
                        .map_err(AppError::Database)
                },
                "delete_product".to_string(),
            )
            .await?;
            
        // Invalidate caches
        self.cache.invalidate(&format!("products:id:{}", id)).await?;
        
        // Invalidate category caches if needed
        if let Some(category) = &product.category {
            self.cache
                .invalidate_by_pattern(&format!("products:category:{}:*", category))
                .await?;
        }
        
        // Invalidate list caches
        self.cache.invalidate_by_pattern("products:list:*").await?;
        
        info!("Deleted product: {}", id);
        Ok(())
    }

    async fn insert_product_to_db(pool: PgPool, product: &Product) -> Result<(), AppError> {
        query!(
            "INSERT INTO products 
            (id, name, description, price, category, inventory_count, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            product.id,
            product.name,
            product.description,
            product.price,
            product.category,
            product.inventory_count,
            product.created_at,
            product.updated_at
        )
        .execute(&pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }
}
```

## Order Service

```rust name=src/services/order_service.rs
use crate::{
    cache::redis::DistributedCache,
    db::{connection::DbConnectionManager, sharding::ShardKey},
    error::AppError,
    models::order::{CreateOrderDto, CreateOrderItemDto, Order, OrderItem, OrderStatus, OrderWithItems, UpdateOrderStatusDto},
    queue::rabbitmq::RabbitMQClient,
    services::product_service::ProductService,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Postgres, Transaction};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderProcessingMessage {
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct OrderService {
    db: DbConnectionManager,
    cache: Arc<DistributedCache>,
    rabbitmq: Arc<RabbitMQClient>,
    product_service: Arc<ProductService>,
}

impl OrderService {
    pub fn new(
        db: DbConnectionManager,
        cache: Arc<DistributedCache>,
        rabbitmq: Arc<RabbitMQClient>,
        product_service: Arc<ProductService>,
    ) -> Self {
        Self {
            db,
            cache,
            rabbitmq,
            product_service,
        }
    }

    #[instrument(skip(self, dto))]
    pub async fn create_order(&self, dto: CreateOrderDto) -> Result<OrderWithItems, AppError> {
        // Validate all products exist and have sufficient inventory
        let mut total_amount = 0;
        let mut order_items = Vec::new();

        for item_dto in &dto.items {
            let product = self.product_service.get_product(item_dto.product_id).await?;
            
            if product.inventory_count < item_dto.quantity {
                return Err(AppError::Validation(format!(
                    "Insufficient inventory for product {}: {} available, {} requested",
                    product.id, product.inventory_count, item_dto.quantity
                )));
            }
            
            let item_price = product.price * item_dto.quantity as i64;
            total_amount += item_price;
            
            order_items.push((item_dto.product_id, item_dto.quantity, product.price));
        }

        // Create order with transaction
        let order = Order::new(dto.customer_id, total_amount);
        
        // Use customer_id as shard key for the order
        let shard_key = ShardKey::UserId(dto.customer_id);
        let pool = self.db.get_pool_for_key(&shard_key).await?;

        // Start transaction
        let mut tx = pool.begin().await?;
        
        // Insert order
        query!(
            "INSERT INTO orders (id, customer_id, status, total_amount, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)",
            order.id,
            order.customer_id,
            order.status as _,
            order.total_amount,
            order.created_at,
            order.updated_at
        )
        .execute(&mut tx)
        .await?;

        // Insert order items and update inventory
        let mut items = Vec::new();
        for (product_id, quantity, price) in order_items {
            let item = OrderItem::new(order.id, product_id, quantity, price);
            
            query!(
                "INSERT INTO order_items (id, order_id, product_id, quantity, price_at_time, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)",
                item.id,
                item.order_id,
                item.product_id,
                item.quantity,
                item.price_at_time,
                item.created_at
            )
            .execute(&mut tx)
            .await?;

            // Update product inventory
            query!(
                "UPDATE products SET inventory_count = inventory_count - $1 WHERE id = $2",
                quantity,
                product_id
            )
            .execute(&mut tx)
            .await?;

            items.push(item);
        }

        // Commit transaction
        tx.commit().await?;

        // Send message to order processing queue
        let message = OrderProcessingMessage {
            order_id: order.id,
            customer_id: order.customer_id,
            status: format!("{:?}", order.status),
            timestamp: chrono::Utc::now(),
        };
        
        let payload = serde_json::to_vec(&message)?;
        self.rabbitmq.publish("order_processing", &payload).await?;
        
        // Invalidate relevant caches
        self.cache.invalidate_by_pattern(&format!("orders:customer:{}:*", dto.customer_id)).await?;

        info!("Created order: {} for customer: {}", order.id, order.customer_id);
        
        Ok(OrderWithItems { order, items })
    }

    #[instrument(skip(self))]
    pub async fn get_order(&self, id: Uuid) -> Result<OrderWithItems, AppError> {
        let cache_key = format!("orders:id:{}", id);
        
        self.cache
            .get_or_compute(&cache_key, || async {
                // Get order first to determine shard
                let shard_key = ShardKey::OrderId(id);
                let pool = self.db.get_pool_for_key(&shard_key).await?;
                
                // Get order
                let order = query_as::<_, Order>("SELECT * FROM orders WHERE id = $1")
                    .bind(id)
                    .fetch_optional(&pool)
                    .await?
                    .ok_or_else(|| AppError::NotFound(format!("Order not found: {}", id)))?;
                
                // Get order items
                let items = query_as::<_, OrderItem>("SELECT * FROM order_items WHERE order_id = $1")
                    .bind(id)
                    .fetch_all(&pool)
                    .await?;
                
                Ok(OrderWithItems { order, items })
            }, None)
            .await
    }

    #[instrument(skip(self))]
    pub async fn get_orders_by_customer(
        &self,
        customer_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Order>, AppError> {
        let cache_key = format!("orders:customer:{}:{}:{}", customer_id, limit, offset);
        
        self.cache
            .get_or_compute(&cache_key, || async {
                let shard_key = ShardKey::UserId(customer_id);
                let pool = self.db.get_pool_for_key(&shard_key).await?;
                
                let orders = query_as::<_, Order>(
                    "SELECT * FROM orders WHERE customer_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
                )
                .bind(customer_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&pool)
                .await?;
                
                Ok(orders)
            }, None)
            .await
    }

    #[instrument(skip(self, dto))]
    pub async fn update_order_status(
        &self,
        id: Uuid,
        dto: UpdateOrderStatusDto,
    ) -> Result<Order, AppError> {
        // Get existing order to determine correct shard
        let order_data = self.get_order(id).await?;
        let mut order = order_data.order;
        
        // Update status
        order.update_status(dto.status);
        
        // Get pool for this order
        let shard_key = ShardKey::UserId(order.customer_id);
        let pool = self.db.get_pool_for_key(&shard_key).await?;
        
        // Update in database
        query!(
            "UPDATE orders SET status = $1, updated_at = $2 WHERE id = $3",
            order.status as _,
            order.updated_at,
            order.id
        )
        .execute(&pool)
        .await?;
        
        // Send message to order processing queue
        let message = OrderProcessingMessage {
            order_id: order.id,
            customer_id: order.customer_id,
            status: format!("{:?}", order.status),
            timestamp: chrono::Utc::now(),
        };
        
        let payload = serde_json::to_vec(&message)?;
        self.rabbitmq.publish("order_status_updates", &payload).await?;
        
        // Invalidate caches
        self.cache.invalidate(&format!("orders:id:{}", id)).await?;
        self.cache.invalidate_by_pattern(&format!("orders:customer:{}:*", order.customer_id)).await?;
        
        info!(
            "Updated order status: {} for customer: {} to: {:?}", 
            order.id, order.customer_id, order.status
        );
        
        Ok(order)
    }

    #[instrument(skip(self, tx))]
    async fn insert_order_tx<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        order: &Order,
    ) -> Result<(), AppError> {
        query!(
            "INSERT INTO orders (id, customer_id, status, total_amount, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)",
            order.id,
            order.customer_id,
            order.status as _,
            order.total_amount,
            order.created_at,
            order.updated_at
        )
        .execute(tx)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cancel_order(&self, id: Uuid) -> Result<Order, AppError> {
        // Get existing order
        let order_data = self.get_order(id).await?;
        let order = order_data.order;
        
        // Check if order can be canceled (only certain statuses)
        match order.status {
            OrderStatus::Created | OrderStatus::Processing => {
                // Continue with cancellation
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Cannot cancel order with status: {:?}", 
                    order.status
                )));
            }
        }
        
        // Update status
        let dto = UpdateOrderStatusDto {
            status: OrderStatus::Canceled,
        };
        
        self.update_order_status(id, dto).await
    }
}
```

## Circuit Breaker Implementation

```rust name=src/utils/circuit_breaker.rs
use crate::config::AppConfig;
use crate::error::AppError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

pub enum CircuitState {
    Closed,
    Open(Instant), // When the circuit was opened
    HalfOpen,
}

struct CircuitMetrics {
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    last_success: Option<Instant>,
}

impl CircuitMetrics {
    fn new() -> Self {
        Self {
            failure_count: 0,
            success_count: 0,
            last_failure: None,
            last_success: None,
        }
    }

    fn record_success(&mut self) {
        self.success_count += 1;
        self.failure_count = 0;
        self.last_success = Some(Instant::now());
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.success_count = 0;
        self.last_failure = Some(Instant::now());
    }

    fn reset(&mut self) {
        self.failure_count = 0;
        self.success_count = 0;
    }
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout_duration: Duration,
    reset_timeout: Duration,
    states: Mutex<HashMap<String, CircuitState>>,
    metrics: Mutex<HashMap<String, CircuitMetrics>>,
}

impl CircuitBreaker {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            failure_threshold: config.circuit_breaker.failure_threshold,
            success_threshold: config.circuit_breaker.success_threshold,
            timeout_duration: Duration::from_millis(config.circuit_breaker.timeout_ms),
            reset_timeout: Duration::from_secs(30), // Time before circuit transitions from Open to HalfOpen
            states: Mutex::new(HashMap::new()),
            metrics: Mutex::new(HashMap::new()),
        }
    }

    pub async fn call<F, Fut, T>(&self, operation: F, operation_name: String) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        // Check if circuit is open
        match self.get_circuit_state(&operation_name) {
            CircuitState::Open(opened_at) => {
                // Check if reset timeout has elapsed
                if opened_at.elapsed() > self.reset_timeout {
                    debug!(
                        "Circuit for {} transitioning from Open to HalfOpen",
                        operation_name
                    );
                    self.set_circuit_state(&operation_name, CircuitState::HalfOpen);
                } else {
                    // Circuit is still open, fail fast
                    return Err(AppError::CircuitBreakerOpen(format!(
                        "Circuit breaker open for operation: {}",
                        operation_name
                    )));
                }
            }
            _ => {} // Continue with the call for Closed or HalfOpen
        }

        // Execute the operation with timeout
        match timeout(self.timeout_duration, operation()).await {
            Ok(Ok(result)) => {
                // Operation succeeded
                self.record_success(&operation_name);
                Ok(result)
            }
            Ok(Err(error)) => {
                // Operation failed
                self.record_failure(&operation_name);
                Err(error)
            }
            Err(_) => {
                // Operation timed out
                self.record_failure(&operation_name);
                Err(AppError::Timeout(format!(
                    "Operation timed out: {}",
                    operation_name
                )))
            }
        }
    }

    fn get_circuit_state(&self, operation: &str) -> CircuitState {
        let states = self.states.lock().unwrap();
        match states.get(operation) {
            Some(state) => match state {
                CircuitState::Open(instant) => CircuitState::Open(*instant),
                CircuitState::HalfOpen => CircuitState::HalfOpen,
                CircuitState::Closed => CircuitState::Closed,
            },
            None => CircuitState::Closed, // Default state
        }
    }

    fn set_circuit_state(&self, operation: &str, state: CircuitState) {
        let mut states = self.states.lock().unwrap();
        states.insert(operation.to_string(), state);
    }

    fn record_success(&self, operation: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let entry = metrics.entry(operation.to_string()).or_insert_with(CircuitMetrics::new);
        entry.record_success();

        // Check if we should close the circuit in HalfOpen state
        if let CircuitState::HalfOpen = self.get_circuit_state(operation) {
            if entry.success_count >= self.success_threshold {
                info!("Circuit for {} closed after successful tests", operation);
                self.set_circuit_state(operation, CircuitState::Closed);
                entry.reset();
            }
        }
    }

    fn record_failure(&self, operation: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let entry = metrics.entry(operation.to_string()).or_insert_with(CircuitMetrics::new);
        entry.record_failure();

        // Check if we should open the circuit
        match self.get_circuit_state(operation) {
            CircuitState::Closed => {
                if entry.failure_count >= self.failure_threshold {
                    warn!("Circuit for {} opened due to failures", operation);
                    self.set_circuit_state(operation, CircuitState::Open(Instant::now()));
                    entry.reset();
                }
            }
            CircuitState::HalfOpen => {
                // In half-open state, a single failure returns to open
                warn!("Circuit for {} reopened after test failure", operation);
                self.set_circuit_state(operation, CircuitState::Open(Instant::now()));
                entry.reset();
            }
            _ => {}
        }
    }
}
```

## Health Check Utility

```rust name=src/utils/health.rs
use crate::{
    cache::redis::RedisCache,
    db::connection::DbConnectionManager,
    error::AppError,
    queue::rabbitmq::RabbitMQClient,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub database: ComponentHealth,
    pub redis: ComponentHealth,
    pub rabbitmq: ComponentHealth,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

pub struct HealthChecker {
    db: DbConnectionManager,
    redis: Arc<RedisCache>,
    rabbitmq: Arc<RabbitMQClient>,
    version: String,
}

impl HealthChecker {
    pub fn new(
        db: DbConnectionManager,
        redis: Arc<RedisCache>,
        rabbitmq: Arc<RabbitMQClient>,
        version: String,
    ) -> Self {
        Self {
            db,
            redis,
            rabbitmq,
            version,
        }
    }

    pub async fn check_health(&self) -> HealthCheck {
        let now = chrono::Utc::now();
        
        // Check database
        let db_health = self.check_database().await;
        
        // Check Redis
        let redis_health = self.check_redis().await;
        
        // Check RabbitMQ
        let rabbitmq_health = self.check_rabbitmq().await;
        
        // Determine overall status
        let status = match (&db_health.status, &redis_health.status, &rabbitmq_health.status) {
            (HealthStatus::Healthy, HealthStatus::Healthy, HealthStatus::Healthy) => {
                HealthStatus::Healthy
            }
            (HealthStatus::Unhealthy, _, _) | (_, HealthStatus::Unhealthy, _) | (_, _, HealthStatus::Unhealthy) => {
                HealthStatus::Unhealthy
            }
            _ => HealthStatus::Degraded,
        };
        
        HealthCheck {
            status,
            database: db_health,
            redis: redis_health,
            rabbitmq: rabbitmq_health,
            version: self.version.clone(),
            timestamp: now,
        }
    }
    
    async fn check_database(&self) -> ComponentHealth {
        let now = chrono::Utc::now();
        
        // Try to get a connection to at least one database shard
        let pools = self.db.get_all_leader_pools();
        if pools.is_empty() {
            return ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some("No database pools available".to_string()),
                last_check: now,
            };
        }
        
        let mut healthy_count = 0;
        let mut errors = Vec::new();
        
        for (i, pool) in pools.iter().enumerate() {
            match pool.acquire().await {
                Ok(mut conn) => {
                    match sqlx::query("SELECT 1").execute(&mut *conn).await {
                        Ok(_) => {
                            healthy_count += 1;
                        }
                        Err(e) => {
                            errors.push(format!("Shard {}: Query failed: {}", i, e));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Shard {}: Connection failed: {}", i, e));
                }
            }
        }
        
        // Determine status based on how many shards are healthy
        if healthy_count == pools.len() {
            ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                last_check: now,
            }
        } else if healthy_count > 0 {
            ComponentHealth {
                status: HealthStatus::Degraded,
                message: Some(format!(
                    "{}/{} database shards healthy. Errors: {}",
                    healthy_count,
                    pools.len(),
                    errors.join("; ")
                )),
                last_check: now,
            }
        } else {
            ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("All database shards unhealthy: {}", errors.join("; "))),
                last_check: now,
            }
        }
    }
    
    async fn check_redis(&self) -> ComponentHealth {
        let now = chrono::Utc::now();
        
        let mut conn = match self.redis.get_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                return ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Redis connection failed: {}", e)),
                    last_check: now,
                };
            }
        };
        
        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
            Ok(response) => {
                if response == "PONG" {
                    ComponentHealth {
                        status: HealthStatus::Healthy,
                        message: None,
                        last_check: now,
                    }
                } else {
                    ComponentHealth {
                        status: HealthStatus::Degraded,
                        message: Some(format!("Redis PING returned unexpected response: {}", response)),
                        last_check: now,
                    }
                }
            }
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Redis PING failed: {}", e)),
                last_check: now,
            },
        }
    }
    
    async fn check_rabbitmq(&self) -> ComponentHealth {
        let now = chrono::Utc::now();
        
        match self.rabbitmq.get_connection().await {
            Ok(conn) => {
                match conn.create_channel().await {
                    Ok(_) => ComponentHealth {
                        status: HealthStatus::Healthy,
                        message: None,
                        last_check: now,
                    },
                    Err(e) => ComponentHealth {
                        status: HealthStatus::Degraded,
                        message: Some(format!("RabbitMQ channel creation failed: {}", e)),
                        last_check: now,
                    },
                }
            }
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("RabbitMQ connection failed: {}", e)),
                last_check: now,
            },
        }
    }
}
```

## API Endpoints - Product Controller

```rust name=src/api/product.rs
use crate::{
    error::AppError,
    models::product::{CreateProductDto, UpdateProductDto},
    services::product_service::ProductService,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ProductListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    category: Option<String>,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    data: serde_json::Value,
}

pub async fn create_product(
    State(product_service): State<Arc<ProductService>>,
    Json(dto): Json<CreateProductDto>,
) -> Result<(StatusCode, Json<ProductResponse>), AppError> {
    let product = product_service.create_product(dto).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(ProductResponse {
            data: serde_json::to_value(product)?,
        }),
    ))
}

pub async fn get_product(
    State(product_service): State<Arc<ProductService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>, AppError> {
    let product = product_service.get_product(id).await?;
    
    Ok(Json(ProductResponse {
        data: serde_json::to_value(product)?,
    }))
}

pub async fn list_products(
    State(product_service): State<Arc<ProductService>>,
    Query(query): Query<ProductListQuery>,
) -> Result<Json<ProductResponse>, AppError> {
    let products = match &query.category {
        Some(category) => {
            product_service
                .get_products_by_category(category, query.limit, query.offset)
                .await?
        }
        None => {
            product_service
                .list_products(query.limit, query.offset)
                .await?
        }
    };
    
    Ok(Json(ProductResponse {
        data: serde_json::to_value(products)?,
    }))
}

pub async fn update_product(
    State(product_service): State<Arc<ProductService>>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateProductDto>,
) -> Result<Json<ProductResponse>, AppError> {
    let product = product_service.update_product(id, dto).await?;
    
    Ok(Json(ProductResponse {
        data: serde_json::to_value(product)?,
    }))
}

pub async fn delete_product(
    State(product_service): State<Arc<ProductService>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    product_service.delete_product(id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}
```

## API Endpoints - Order Controller

```rust name=src/api/order.rs
use crate::{
    error::AppError,
    models::order::{CreateOrderDto, UpdateOrderStatusDto},
    services::order_service::OrderService,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct OrderListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    customer_id: Option<Uuid>,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    data: serde_json::Value,
}

pub async fn create_order(
    State(order_service): State<Arc<OrderService>>,
    Json(dto): Json<CreateOrderDto>,
) -> Result<(StatusCode, Json<OrderResponse>), AppError> {
    let order = order_service.create_order(dto).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(OrderResponse {
            data: serde_json::to_value(order)?,
        }),
    ))
}

pub async fn get_order(
    State(order_service): State<Arc<OrderService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrderResponse>, AppError> {
    let order = order_service.get_order(id).await?;
    
    Ok(Json(OrderResponse {
        data: serde_json::to_value(order)?,
    }))
}

pub async fn list_orders(
    State(order_service): State<Arc<OrderService>>,
    Query(query): Query<OrderListQuery>,
) -> Result<Json<OrderResponse>, AppError> {
    match query.customer_id {
        Some(customer_id) => {
            let orders = order_service
                .get_orders_by_customer(customer_id, query.limit, query.offset)
                .await?;
                
            Ok(Json(OrderResponse {
                data: serde_json::to_value(orders)?,
            }))
        }
        None => {
            return Err(AppError::Validation(
                "customer_id is required for listing orders".to_string(),
            ));
        }
    }
}

pub async fn update_order_status(
    State(order_service): State<Arc<OrderService>>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateOrderStatusDto>,
) -> Result<Json<OrderResponse>, AppError> {
    let order = order_service.update_order_status(id, dto).await?;
    
    Ok(Json(OrderResponse {
        data: serde_json::to_value(order)?,
    }))
}

pub async fn cancel_order(
    State(order_service): State<Arc<OrderService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrderResponse>, AppError> {
    let order = order_service.cancel_order(id).await?;
    
    Ok(Json(OrderResponse {
        data: serde_json::to_value(order)?,
    }))
}
```

## API Module

```rust name=src/api/mod.rs
use crate::{
    error::AppError,
    utils::health::{HealthChecker, HealthCheck, HealthStatus},
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post, delete, put},
    Router,
};
use std::sync::Arc;
use tower_http::{
    trace::{TraceLayer},
    timeout::TimeoutLayer,
    cors::CorsLayer,
};
use std::time::Duration;

// Import controllers
pub mod product;
pub mod order;

pub fn create_router(
    product_service: Arc<crate::services::product_service::ProductService>,
    order_service: Arc<crate::services::order_service::OrderService>,
    health_checker: Arc<HealthChecker>,
) -> Router {
    // Shared middleware
    let middleware_stack = tower::ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CorsLayer::permissive());

    // Create routers
    let product_routes = Router::new()
        .route("/", post(product::create_product))
        .route("/", get(product::list_products))
        .route("/:id", get(product::get_product))
        .route("/:id", put(product::update_product))
        .route("/:id", delete(product::delete_product))
        .with_state(product_service);

    let order_routes = Router::new()
        .route("/", post(order::create_order))
        .route("/", get(order::list_orders))
        .route("/:id", get(order::get_order))
        .route("/:id/status", put(order::update_order_status))
        .route("/:id/cancel", post(order::cancel_order))
        .with_state(order_service);

    // Define health check endpoints
    let health_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/liveness", get(liveness_handler))
        .with_state(health_checker);

    // Combine all routes
    Router::new()
        .nest("/api/products", product_routes)
        .nest("/api/orders", order_routes)
        .merge(health_routes)
        .layer(middleware_stack)
}

async fn health_handler(
    State(health_checker): State<Arc<HealthChecker>>,
) -> Result<(StatusCode, axum::Json<HealthCheck>), AppError> {
    let health = health_checker.check_health().await;
    
    let status_code = match health.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still return 200 but with degraded status in body
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };
    
    Ok((status_code, axum::Json(health)))
}

async fn liveness_handler() -> StatusCode {
    StatusCode::OK
}
```

## Telemetry - Metrics

```rust name=src/telemetry/metrics.rs
use crate::config::AppConfig;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use opentelemetry::{
    global, metrics::Counter, metrics::ValueRecorder, sdk::Resource, KeyValue
};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::info;

pub struct MetricsState {
    registry: Registry,
    request_counter: Counter<u64>,
    response_time: ValueRecorder<f64>,
}

impl MetricsState {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        // Create a new PrometheusExporter
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()
            .expect("Failed to create Prometheus exporter");
        
        // Configure the meter provider with the exporter
        let meter_provider = opentelemetry_sdk::metrics::MeterProvider::builder()
            .with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "distributed-ecommerce",
            )]))
            .with_reader(exporter)
            .build();
        
        // Get a meter from the provider
        let meter = meter_provider.meter("distributed-ecommerce");
        
        // Create instruments
        let request_counter = meter
            .u64_counter("http_requests_total")
            .with_description("Total number of HTTP requests")
            .init();
        
        let response_time = meter
            .f64_value_recorder("http_response_time_seconds")
            .with_description("HTTP response time in seconds")
            .init();
        
        Self {
            registry,
            request_counter,
            response_time,
        }
    }
    
    pub fn increment_request_counter(&self, method: &str, path: &str, status_code: u16) {
        self.request_counter.add(
            1,
            &[
                KeyValue::new("method", method.to_string()),
                KeyValue::new("path", path.to_string()),
                KeyValue::new("status", status_code.to_string()),
            ],
        );
    }
    
    pub fn record_response_time(&self, method: &str, path: &str, duration_secs: f64) {
        self.response_time.record(
            duration_secs,
            &[
                KeyValue::new("method", method.to_string()),
                KeyValue::new("path", path.to_string()),
            ],
        );
    }
}

async fn metrics_handler(State(state): State<Arc<MetricsState>>) -> Response {
    let encoder = TextEncoder::new();
    let metric_families = state.registry.gather();
    
    let mut buffer = Vec::new();
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => match String::from_utf8(buffer) {
            Ok(metrics_text) => (
                StatusCode::OK,
                [(
                    axum::http::header::CONTENT_TYPE,
                    "text/plain; charset=utf-8",
                )],
                metrics_text,
            )
                .into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn init_metrics() -> (Arc<MetricsState>, Router) {
    let metrics_state = Arc::new(MetricsState::new());
    
    let metrics_router = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(metrics_state.clone());
    
    (metrics_state, metrics_router)
}

pub async fn start_metrics_server(config: AppConfig) -> JoinHandle<()> {
    let metrics_port = config.telemetry.metrics_port;
    let (_metrics_state, metrics_router) = init_metrics();
    
    info!("Starting metrics server on port {}", metrics_port);
    
    tokio::spawn(async move {
        let addr = format!("0.0.0.0:{}", metrics_port).parse().unwrap();
        let server = axum::Server::bind(&addr)
            .serve(metrics_router.into_make_service());
        
        if let Err(e) = server.await {
            tracing::error!("Metrics server error: {}", e);
        }
    })
}
```

## Telemetry - Tracing

```rust name=src/telemetry/tracing.rs
use crate::config::AppConfig;
use opentelemetry::{global, sdk::Resource, KeyValue};
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, registry::Registry, util::SubscriberInitExt,
};

pub fn init_tracing(config: &AppConfig) {
    // Parse trace level from config
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.telemetry.tracing_level));

    // Create a resource to identify this service
    let resource = Resource::new(vec![KeyValue::new(
        "service.name",
        "distributed-ecommerce",
    )]);

    // Configure the OpenTelemetry tracer
    let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_resource(resource)
        .build()
        .tracer("distributed-ecommerce");

    // Create a tracing layer with the configured tracer
    let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create a registry with the layers
    Registry::default()
        .with(filter_layer)
        .with(fmt::layer().json().with_target(true))
        .with(opentelemetry_layer)
        .init();
}

pub fn shutdown_tracing() {
    // Shut down the tracer provider
    global::shutdown_tracer_provider();
}
```

## Main Application

```rust name=src/main.rs
mod api;
mod cache;
mod config;
mod db;
mod error;
mod models;
mod queue;
mod services;
mod telemetry;
mod utils;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

use crate::api::create_router;
use crate::cache::redis::RedisCache;
use crate::cache::redis::DistributedCache;
use crate::config::load_config;
use crate::db::connection::DbConnectionManager;
use crate::queue::rabbitmq::RabbitMQClient;
use crate::services::order_service::OrderService;
use crate::services::product_service::ProductService;
use crate::telemetry::metrics::start_metrics_server;
use crate::telemetry::tracing::{init_tracing, shutdown_tracing};
use crate::utils::circuit_breaker::CircuitBreaker;
use crate::utils::health::HealthChecker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config();
    
    // Initialize tracing
    init_tracing(&config);
    
    info!("Starting distributed e-commerce application");
    info!("Environment: {}", config.environment);
    
    // Initialize database connection manager
    info!("Initializing database connections");
    let db_manager = DbConnectionManager::new(config.clone()).await?;
    
    // Initialize Redis cache
    info!("Initializing Redis cache");
    let redis_cache = RedisCache::new(config.clone()).await?;
    let distributed_cache = Arc::new(DistributedCache::new(redis_cache.clone()));
    
    // Initialize RabbitMQ client
    info!("Initializing RabbitMQ client");
    let rabbitmq_client = Arc::new(RabbitMQClient::new(config.clone()).await?);
    
    // Initialize circuit breaker
    let circuit_breaker = Arc::new(CircuitBreaker::new(&config));
    
    // Initialize services
    let product_service = Arc::new(ProductService::new(
        db_manager.clone(),
        distributed_cache.clone(),
        circuit_breaker.clone(),
    ));
    
    let order_service = Arc::new(OrderService::new(
        db_manager.clone(),
        distributed_cache.clone(),
        rabbitmq_client.clone(),
        product_service.clone(),
    ));
    
    // Initialize health checker
    let health_checker = Arc::new(HealthChecker::new(
        db_manager.clone(),
        Arc::new(redis_cache),
        rabbitmq_client,
        env!("CARGO_PKG_VERSION").to_string(),
    ));
    
    // Start metrics server
    let _metrics_handle = start_metrics_server(config.clone()).await;
    
    // Create the application router
    let app = create_router(product_service, order_service, health_checker);
    
    // Start the web server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Listening on {}", addr);
    
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal());
    
    if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
    }
    
    // Shutdown tracing
    shutdown_tracing();
    
    info!("Server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM, starting graceful shutdown");
        },
    }
}
```

## Docker Configuration

```dockerfile name=docker/api/Dockerfile
FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .

# Build the application in release mode
RUN cargo build --release

# Create a minimal runtime image
FROM debian:bookworm-slim

# Install necessary dependencies
RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/distributed-ecommerce /app/distributed-ecommerce
COPY --from=builder /usr/src/app/config /app/config

# Set environment variables
ENV RUN_MODE=production
ENV APP__SERVER__HOST=0.0.0.0
ENV APP__SERVER__PORT=8080

# Expose ports
EXPOSE 8080 9090

# Run the binary
CMD ["./distributed-ecommerce"]
```

```bash name=docker/db/init-shards.sh
#!/bin/bash
set -e

# This script initializes multiple Postgres databases for sharding
# and sets up replication between them

# Create the replication_log table in each database
function create_replication_log {
    local db_name=$1
    echo "Creating replication_log table in $db_name"
    
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db_name" <<-EOSQL
        CREATE TABLE IF NOT EXISTS replication_log (
            id UUID PRIMARY KEY,
            timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
            sql_statement TEXT NOT NULL,
            params JSONB,
            source_shard INTEGER NOT NULL,
            entity_type VARCHAR(255) NOT NULL,
            entity_id VARCHAR(255) NOT NULL,
            operation VARCHAR(50) NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
        
        CREATE INDEX IF NOT EXISTS replication_log_entity_idx ON replication_log(entity_type, entity_id);
        CREATE INDEX IF NOT EXISTS replication_log_timestamp_idx ON replication_log(timestamp);
EOSQL
}

# Create shard databases
for i in {0..3}; do
    db_name="ecommerce_shard_$i"
    
    echo "Creating database $db_name"
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
        CREATE DATABASE $db_name;
EOSQL

    # Create extension uuid-ossp
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db_name" <<-EOSQL
        CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
EOSQL

    # Create common tables in each shard
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db_name" <<-EOSQL
        -- Products table
        CREATE TABLE IF NOT EXISTS products (
            id UUID PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            price BIGINT NOT NULL,
            category VARCHAR(100),
            inventory_count INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS products_category_idx ON products(category);
        
        -- Create order status enum type
        DO $$ 
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'order_status') THEN
                CREATE TYPE order_status AS ENUM (
                    'created',
                    'processing',
                    'shipped',
                    'delivered',
                    'canceled',
                    'returned'
                );
            END IF;
        END
        $$;
        
        -- Orders table
        CREATE TABLE IF NOT EXISTS orders (
            id UUID PRIMARY KEY,
            customer_id UUID NOT NULL,
            status order_status NOT NULL,
            total_amount BIGINT NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS orders_customer_idx ON orders(customer_id);
        CREATE INDEX IF NOT EXISTS orders_status_idx ON orders(status);
        
        -- Order items table
        CREATE TABLE IF NOT EXISTS order_items (
            id UUID PRIMARY KEY,
            order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
            product_id UUID NOT NULL,
            quantity INTEGER NOT NULL,
            price_at_time BIGINT NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS order_items_order_idx ON order_items(order_id);
        CREATE INDEX IF NOT EXISTS order_items_product_idx ON order_items(product_id);
EOSQL

    # Create replication log table
    create_replication_log "$db_name"
done

echo "Database initialization completed"
```

## Docker Compose Setup

```yaml name=docker-compose.yml
version: '3.8'

services:
  # API service with multiple instances for load balancing
  api1:
    build:
      context: .
      dockerfile: docker/api/Dockerfile
    container_name: ecommerce-api1
    ports:
      - "8081:8080"
      - "9091:9090"
    depends_on:
      - db-shard1
      - db-shard2
      - db-shard3
      - db-shard4
      - redis
      - rabbitmq
    environment:
      - APP__SERVER__HOST=0.0.0.0
      - APP__SERVER__PORT=8080
      - APP__SERVER__SHUTDOWN_TIMEOUT_SECS=10
      - APP__DATABASE__SHARD_COUNT=4
      - APP__DATABASE__SHARDS__0__HOST=db-shard1
      - APP__DATABASE__SHARDS__0__PORT=5432
      - APP__DATABASE__SHARDS__0__USERNAME=postgres
      - APP__DATABASE__SHARDS__0__PASSWORD=postgres
      - APP__DATABASE__SHARDS__0__DATABASE=ecommerce_shard_0
      - APP__DATABASE__SHARDS__0__POOL_SIZE=10
      - APP__DATABASE__SHARDS__0__IS_LEADER=true
      - APP__DATABASE__SHARDS__1__HOST=db-shard2
      - APP__DATABASE__SHARDS__1__PORT=5432
      - APP__DATABASE__SHARDS__1__USERNAME=postgres
      - APP__DATABASE__SHARDS__1__PASSWORD=postgres
      - APP__DATABASE__SHARDS__1__DATABASE=ecommerce_shard_1
      - APP__DATABASE__SHARDS__1__POOL_SIZE=10
      - APP__DATABASE__SHARDS__1__IS_LEADER=true
      - APP__DATABASE__SHARDS__2__HOST=db-shard3
      - APP__DATABASE__SHARDS__2__PORT=5432
      - APP__DATABASE__SHARDS__2__USERNAME=postgres
      - APP__DATABASE__SHARDS__2__PASSWORD=postgres
      - APP__DATABASE__SHARDS__2__DATABASE=ecommerce_shard_2
      - APP__DATABASE__SHARDS__2__POOL_SIZE=10
      - APP__DATABASE__SHARDS__2__IS_LEADER=false
      - APP__DATABASE__SHARDS__3__HOST=db-shard4
      - APP__DATABASE__SHARDS__3__PORT=5432
      - APP__DATABASE__SHARDS__3__USERNAME=postgres
      - APP__DATABASE__SHARDS__3__PASSWORD=postgres
      - APP__DATABASE__SHARDS__3__DATABASE=ecommerce_shard_3
      - APP__DATABASE__SHARDS__3__POOL_SIZE=10
      - APP__DATABASE__SHARDS__3__IS_LEADER=false
      - APP__REDIS__URL=redis://redis:6379
      - APP__REDIS__POOL_SIZE=20
      - APP__REDIS__TTL_SECONDS=3600
      - APP__RABBITMQ__URL=amqp://guest:guest@rabbitmq:5672/%2F
      - APP__RABBITMQ__ORDER_QUEUE=order_processing
      - APP__RABBITMQ__LOGGING_QUEUE=application_logs
      - APP__TELEMETRY__TRACING_LEVEL=info
      - APP__TELEMETRY__METRICS_PORT=9090
      - APP__CIRCUIT_BREAKER__FAILURE_THRESHOLD=5
      - APP__CIRCUIT_BREAKER__SUCCESS_THRESHOLD=3
      - APP__CIRCUIT_BREAKER__TIMEOUT_MS=2000
      - APP__ENVIRONMENT=production
    networks:
      - ecommerce-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 15s

  api2:
    build:
      context: .
      dockerfile: docker/api/Dockerfile
    container_name: ecommerce-api2
    ports:
      - "8082:8080"
      - "9092:9090"
    depends_on:
      - api1
    environment:
      - APP__SERVER__HOST=0.0.0.0
      - APP__SERVER__PORT=8080
      - APP__SERVER__SHUTDOWN_TIMEOUT_SECS=10
      - APP__DATABASE__SHARD_COUNT=4
      - APP__DATABASE__SHARDS__0__HOST=db-shard1
      - APP__DATABASE__SHARDS__0__PORT=5432
      - APP__DATABASE__SHARDS__0__USERNAME=postgres
      - APP__DATABASE__SHARDS__0__PASSWORD=postgres
      - APP__DATABASE__SHARDS__0__DATABASE=ecommerce_shard_0
      - APP__DATABASE__SHARDS__0__POOL_SIZE=10
      - APP__DATABASE__SHARDS__0__IS_LEADER=true
      - APP__DATABASE__SHARDS__1__HOST=db-shard2
      - APP__DATABASE__SHARDS__1__PORT=5432
      - APP__DATABASE__SHARDS__1__USERNAME=postgres
      - APP__DATABASE__SHARDS__1__PASSWORD=postgres
      - APP__DATABASE__SHARDS__1__DATABASE=ecommerce_shard_1
      - APP__DATABASE__SHARDS__1__POOL_SIZE=10
      - APP__DATABASE__SHARDS__1__IS_LEADER=true
      - APP__DATABASE__SHARDS__2__HOST=db-shard3
      - APP__DATABASE__SHARDS__2__PORT=5432
      - APP__DATABASE__SHARDS__2__USERNAME=postgres
      - APP__DATABASE__SHARDS__2__PASSWORD=postgres
      - APP__DATABASE__SHARDS__2__DATABASE=ecommerce_shard_2
      - APP__DATABASE__SHARDS__2__POOL_SIZE=10
      - APP__DATABASE__SHARDS__2__IS_LEADER=false
      - APP__DATABASE__SHARDS__3__HOST=db-shard4
      - APP__DATABASE__SHARDS__3__PORT=5432
      - APP__DATABASE__SHARDS__3__USERNAME=postgres
      - APP__DATABASE__SHARDS__3__PASSWORD=postgres
      - APP__DATABASE__SHARDS__3__DATABASE=ecommerce_shard_3
      - APP__DATABASE__SHARDS__3__POOL_SIZE=10
      - APP__DATABASE__SHARDS__3__IS_LEADER=false
      - APP__REDIS__URL=redis://redis:6379
      - APP__REDIS__POOL_SIZE=20
      - APP__REDIS__TTL_SECONDS=3600
      - APP__RABBITMQ__URL=amqp://guest:guest@rabbitmq:5672/%2F
      - APP__RABBITMQ__ORDER_QUEUE=order_processing
      - APP__RABBITMQ__LOGGING_QUEUE=application_logs
      - APP__TELEMETRY__TRACING_LEVEL=info
      - APP__TELEMETRY__METRICS_PORT=9090
      - APP__CIRCUIT_BREAKER__FAILURE_THRESHOLD=5
      - APP__CIRCUIT_BREAKER__SUCCESS_THRESHOLD=3
      - APP__CIRCUIT_BREAKER__TIMEOUT_MS=2000
      - APP__ENVIRONMENT=production
    networks:
      - ecommerce-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 15s

  # Load Balancer
  nginx:
    image: nginx:latest
    container_name: ecommerce-nginx
    ports:
      - "8080:80"
    volumes:
      - ./docker/nginx/nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - api1
      - api2
    networks:
      - ecommerce-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:80/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

  # Database shards
  db-shard1:
    image: postgres:16
    container_name: ecommerce-db-shard1
    ports:
      - "5433:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=postgres
    volumes:
      - db-shard1-data:/var/lib/postgresql/data
      - ./docker/db/init-shards.sh:/docker-entrypoint-initdb.d/init-shards.sh
      - ./seed/seed-data.sql:/docker-entrypoint-initdb.d/seed-data.sql
    networks:
      - ecommerce-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

  db-shard2:
    image: postgres:16
    container_name: ecommerce-db-shard2
    ports:
      - "5434:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=postgres
    volumes:
      - db-shard2-data:/var/lib/postgresql/data
      - ./docker/db/init-shards.sh:/docker-entrypoint-initdb.d/init-shards.sh
    networks:
      - ecommerce-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

  db-shard3:
    image: postgres:16
    container_name: ecommerce-db-shard3
    ports:
      - "5435:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=postgres
    volumes:
      - db-shard3-data:/var/lib/postgresql/data
      - ./docker/db/init-shards.sh:/docker-entrypoint-initdb.d/init-shards.sh
    networks:
      - ecommerce-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

  db-shard4:
    image: postgres:16
    container_name: ecommerce-db-shard4
    ports:
      - "5436:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=postgres
    volumes:
      - db-shard4-data:/var/lib/postgresql/data
      - ./docker/db/init-shards.sh:/docker-entrypoint-initdb.d/init-shards.sh
    networks:
      - ecommerce-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

  # Redis for caching
  redis:
    image: redis:7-alpine
    container_name: ecommerce-redis
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis-data:/data
    networks:
      - ecommerce-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  # RabbitMQ for message queuing
  rabbitmq:
    image: rabbitmq:3-management
    container_name: ecommerce-rabbitmq
    ports:
      - "5672:5672"
      - "15672:15672"
    volumes:
      - rabbitmq-data:/var/lib/rabbitmq
    networks:
      - ecommerce-network
    healthcheck:
      test: ["CMD", "rabbitmqctl", "status"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 20s

volumes:
  db-shard1-data:
  db-shard2-data:
  db-shard3-data:
  db-shard4-data:
  redis-data:
  rabbitmq-data:

networks:
  ecommerce-network:
    driver: bridge
```

## NGINX Configuration for Load Balancer

```conf name=docker/nginx/nginx.conf
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 4096;
    multi_accept on;
    use epoll;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for" '
                    '$request_time $upstream_response_time $pipe';

    access_log /var/log/nginx/access.log main;

    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    server_tokens off;

    # Gzip Settings
    gzip on;
    gzip_disable "msie6";
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_buffers 16 8k;
    gzip_http_version 1.1;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;

    # Upstream servers
    upstream api_servers {
        server api1:8080 max_fails=3 fail_timeout=30s;
        server api2:8080 max_fails=3 fail_timeout=30s;
        
        # Comment out below for round robin (default)
        # least_conn; # Use least connections algorithm
    }

    # HTTP server
    server {
        listen 80;
        server_name localhost;

        # Health check
        location /health {
            proxy_pass http://api_servers/health;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection 'upgrade';
            proxy_set_header Host $host;
            proxy_cache_bypass $http_upgrade;
            
            # Health check specific settings
            proxy_connect_timeout 5s;
            proxy_read_timeout 5s;
            proxy_send_timeout 5s;
        }

        location /liveness {
            proxy_pass http://api_servers/liveness;
            proxy_http_version 1.1;
            proxy_set_header Host $host;
            proxy_connect_timeout 2s;
            proxy_read_timeout 2s;
        }

        # API endpoints
        location /api/ {
            proxy_pass http://api_servers;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection 'upgrade';
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_cache_bypass $http_upgrade;
            
            # Timeouts
            proxy_connect_timeout 10s;
            proxy_send_timeout 30s;
            proxy_read_timeout 30s;
        }

        # Default root location
        location / {
            return 404;
        }
    }
}
```

## Seed Data

```sql name=seed/seed-data.sql
-- SEED DATA for ecommerce_shard_0 database
\c ecommerce_shard_0

-- Products seed data for shard 0 (Electronics category)
INSERT INTO products (id, name, description, price, category, inventory_count, created_at, updated_at)
VALUES
    ('2d3b3c9a-d0b2-4a1a-b1a1-5c4e42f39d76', 'Smartphone X Pro', 'Latest flagship smartphone with 6.7" display and 5G', 99900, 'Electronics', 75, '2025-03-20 10:00:00+00', '2025-03-20 10:00:00+00'),
    ('a8f7e6d5-c4b3-4a2b-9d8e-7f6a5b4c3d2e', 'Laptop Ultra', 'Lightweight laptop with 16GB RAM and 512GB SSD', 129900, 'Electronics', 50, '2025-03-20 10:05:00+00', '2025-03-20 10:05:00+00'),
    ('c1d2e3f4-g5h6-7i8j-9k0l-m1n2o3p4q5r6', 'Wireless Earbuds', 'Noise cancelling earbuds with 24hr battery life', 14900, 'Electronics', 150, '2025-03-20 10:10:00+00', '2025-03-20 10:10:00+00'),
    ('f9e8d7c6-b5a4-3210-9876-f5e4d3c2b1a0', 'Smart Watch', 'Fitness tracker with heart rate monitor', 19900, 'Electronics', 100, '2025-03-20 10:15:00+00', '2025-03-20 10:15:00+00'),
    ('1a2b3c4d-5e6f-7g8h-9i0j-1k2l3m4n5o6p', 'Bluetooth Speaker', 'Waterproof portable speaker with 20hr battery', 7999, 'Electronics', 200, '2025-03-20 10:20:00+00', '2025-03-20 10:20:00+00');

-- Insert a sample order in shard 0
INSERT INTO orders (id, customer_id, status, total_amount, created_at, updated_at)
VALUES 
    ('fd7e9abc-123a-4567-89ab-cdef01234567', 'a1b2c3d4-e5f6-4789-ghi0-jkl543210abc', 'processing', 114800, '2025-03-23 14:30:00+00', '2025-03-23 14:30:00+00');

-- Insert order items
INSERT INTO order_items (id, order_id, product_id, quantity, price_at_time, created_at)
VALUES
    ('aab98765-4321-6789-fedc-ba0987654321', 'fd7e9abc-123a-4567-89ab-cdef01234567', '2d3b3c9a-d0b2-4a1a-b1a1-5c4e42f39d76', 1, 99900, '2025-03-23 14:30:00+00'),
    ('bbf12356-7890-1234-5678-9012def34567', 'fd7e9abc-123a-4567-89ab-cdef01234567', '1a2b3c4d-5e6f-7g8h-9i0j-1k2l3m4n5o6p', 2, 7450, '2025-03-23 14:30:00+00');

-- Switch to shard 1
\c ecommerce_shard_1

-- Products seed data for shard 1 (Home & Kitchen category)
INSERT INTO products (id, name, description, price, category, inventory_count, created_at, updated_at)
VALUES
    ('b1c2d3e4-f5g6-7h8i-9j0k-l1m2n3o4p5q6', 'Coffee Maker', 'Programmable drip coffee maker with thermal carafe', 8999, 'Home & Kitchen', 60, '2025-03-20 11:00:00+00', '2025-03-20 11:00:00+00'),
    ('c2d3e4f5-g6h7-8i9j-0k1l-m2n3o4p5q6r7', 'Stand Mixer', 'Professional 5-quart stand mixer', 34999, 'Home & Kitchen', 25, '2025-03-20 11:05:00+00', '2025-03-20 11:05:00+00'),
    ('d3e4f5g6-h7i8-9j0k-1l2m-n3o4p5q6r7s8', 'Air Fryer', '5-quart digital air fryer', 12999, 'Home & Kitchen', 80, '2025-03-20 11:10:00+00', '2025-03-20 11:10:00+00'),
    ('e4f5g6h7-i8j9-0k1l-2m3n-o4p5q6r7s8t9', 'Robot Vacuum', 'Smart robot vacuum with mapping technology', 29999, 'Home & Kitchen', 40, '2025-03-20 11:15:00+00', '2025-03-20 11:15:00+00'),
    ('f5g6h7i8-j9k0-1l2m-3n4o-p5q6r7s8t9u0', 'Blender', 'High-speed blender for smoothies and soups', 9999, 'Home & Kitchen', 70, '2025-03-20 11:20:00+00', '2025-03-20 11:20:00+00');

-- Insert a sample order in shard 1
INSERT INTO orders (id, customer_id, status, total_amount, created_at, updated_at)
VALUES 
    ('ed8f9abc-234b-5678-90cd-ef01235678ab', 'b2c3d4e5-f6g7-5890-hij1-klm654321bcd', 'created', 12999, '2025-03-23 15:45:00+00', '2025-03-23 15:45:00+00');

-- Insert order items
INSERT INTO order_items (id, order_id, product_id, quantity, price_at_time, created_at)
VALUES
    ('ccg23456-7890-2345-6789-0123efg45678', 'ed8f9abc-234b-5678-90cd-ef01235678ab', 'd3e4f5g6-h7i8-9j0k-1l2m-n3o4p5q6r7s8', 1, 12999, '2025-03-23 15:45:00+00');

-- Switch to shard 2
\c ecommerce_shard_2

-- Products seed data for shard 2 (Clothing category)
INSERT INTO products (id, name, description, price, category, inventory_count, created_at, updated_at)
VALUES
    ('g6h7i8j9-k0l1-2m3n-4o5p-q6r7s8t9u0v1', 'Winter Jacket', 'Water-resistant insulated jacket for cold weather', 13999, 'Clothing', 100, '2025-03-20 12:00:00+00', '2025-03-20 12:00:00+00'),
    ('h7i8j9k0-l1m2-3n4o-5p6q-r7s8t9u0v1w2', 'Running Shoes', 'Lightweight cushioned running shoes', 9999, 'Clothing', 120, '2025-03-20 12:05:00+00', '2025-03-20 12:05:00+00'),
    ('i8j9k0l1-m2n3-4o5p-6q7r-s8t9u0v1w2x3', 'Denim Jeans', 'Classic straight fit jeans', 5999, 'Clothing', 150, '2025-03-20 12:10:00+00', '2025-03-20 12:10:00+00'),
    ('j9k0l1m2-n3o4-5p6q-7r8s-t9u0v1w2x3y4', 'Cotton T-Shirt', 'Pack of 3 premium cotton t-shirts', 2499, 'Clothing', 200, '2025-03-20 12:15:00+00', '2025-03-20 12:15:00+00'),
    ('k0l1m2n3-o4p5-6q7r-8s9t-u0v1w2x3y4z5', 'Wool Sweater', 'Merino wool sweater for winter', 7999, 'Clothing', 80, '2025-03-20 12:20:00+00', '2025-03-20 12:20:00+00');

-- Insert a sample order in shard 2
INSERT INTO orders (id, customer_id, status, total_amount, created_at, updated_at)
VALUES 
    ('dc9e0bcd-345c-6789-01de-f0123456789c', 'c3d4e5f6-g7h8-6901-ijk2-lmn765432cde', 'delivered', 23998, '2025-03-22 09:15:00+00', '2025-03-23 11:30:00+00');

-- Insert order items
INSERT INTO order_items (id, order_id, product_id, quantity, price_at_time, created_at)
VALUES
    ('ddh34567-8901-3456-7890-1234fgh56789', 'dc9e0bcd-345c-6789-01de-f0123456789c', 'i8j9k0l1-m2n3-4o5p-6q7r-s8t9u0v1w2x3', 4, 5999, '2025-03-22 09:15:00+00');

-- Switch to shard 3
\c ecommerce_shard_3

-- Products seed data for shard 3 (Books & Media category)
INSERT INTO products (id, name, description, price, category, inventory_count, created_at, updated_at)
VALUES
    ('l1m2n3o4-p5q6-7r8s-9t0u-v1w2x3y4z5a6', 'Bestseller Novel', 'Award-winning fiction bestseller', 1899, 'Books & Media', 200, '2025-03-20 13:00:00+00', '2025-03-20 13:00:00+00'),
    ('m2n3o4p5-q6r7-8s9t-0u1v-w2x3y4z5a6b7', 'Cookbook', 'International recipes from renowned chef', 2499, 'Books & Media', 100, '2025-03-20 13:05:00+00', '2025-03-20 13:05:00+00'),
    ('n3o4p5q6-r7s8-9t0u-1v2w-x3y4z5a6b7c8', 'Vinyl Record', 'Classic rock album on 180g vinyl', 2999, 'Books & Media', 50, '2025-03-20 13:10:00+00', '2025-03-20 13:10:00+00'),
    ('o4p5q6r7-s8t9-0u1v-2w3x-y4z5a6b7c8d9', 'Video Game', 'Open-world RPG for console', 5999, 'Books & Media', 80, '2025-03-20 13:15:00+00', '2025-03-20 13:15:00+00'),
    ('p5q6r7s8-t9u0-1v2w-3x4y-z5a6b7c8d9e0', 'Blu-ray Movie', '4K Ultra HD movie with bonus features', 2499, 'Books & Media', 120, '2025-03-20 13:20:00+00', '2025-03-20 13:20:00+00');

-- Insert a sample order in shard 3
INSERT INTO orders (id, customer_id, status, total_amount, created_at, updated_at)
VALUES 
    ('cb0f1cde-456d-7890-12ef-g1234567890d', 'd4e5f6g7-h8i9-7012-jkl3-mno876543def', 'shipped', 10497, '2025-03-21 16:20:00+00', '2025-03-22 08:45:00+00');

-- Insert order items
INSERT INTO order_items (id, order_id, product_id, quantity, price_at_time, created_at)
VALUES
    ('eei45678-9012-4567-8901-2345ghi67890', 'cb0f1cde-456d-7890-12ef-g1234567890d', 'l1m2n3o4-p5q6-7r8s-9t0u-v1w2x3y4z5a6', 1, 1899, '2025-03-21 16:20:00+00'),
    ('ffj56789-0123-5678-9012-3456hij78901', 'cb0f1cde-456d-7890-12ef-g1234567890d', 'o4p5q6r7-s8t9-0u1v-2w3x-y4z5a6b7c8d9', 1, 5999, '2025-03-21 16:20:00+00'),
    ('ggk67890-1234-6789-0123-4567ijk89012', 'cb0f1cde-456d-7890-12ef-g1234567890d', 'p5q6r7s8-t9u0-1v2w-3x4y-z5a6b7c8d9e0', 1, 2599, '2025-03-21 16:20:00+00');
```

## Default Configuration

```toml name=config/default.toml
[server]
host = "127.0.0.1"
port = 8080
shutdown_timeout_secs = 10

[database]
shard_count = 4

# Shard configurations
[[database.shards]]
host = "localhost"
port = 5433
username = "postgres"
password = "postgres"
database = "ecommerce_shard_0"
pool_size = 10
is_leader = true

[[database.shards]]
host = "localhost"
port = 5434
username = "postgres"
password = "postgres"
database = "ecommerce_shard_1"
pool_size = 10
is_leader = true

[[database.shards]]
host = "localhost"
port = 5435
username = "postgres"
password = "postgres"
database = "ecommerce_shard_2"
pool_size = 10
is_leader = false

[[database.shards]]
host = "localhost"
port = 5436
username = "postgres"
password = "postgres"
database = "ecommerce_shard_3"
pool_size = 10
is_leader = false

[redis]
url = "redis://localhost:6379"
pool_size = 20
ttl_seconds = 3600

[rabbitmq]
url = "amqp://guest:guest@localhost:5672/%2F"
order_queue = "order_processing"
logging_queue = "application_logs"

[telemetry]
tracing_level = "debug"
metrics_port = 9090

[circuit_breaker]
failure_threshold = 5
success_threshold = 3
timeout_ms = 2000

environment = "development"
```

## README File

````markdown name=README.md
# Distributed E-Commerce System

A production-ready distributed e-commerce system built with Rust, demonstrating key distributed systems concepts.

## Features

- **Database Sharding** with consistent hashing
- **Leader-Follower Replication**
- **Distributed Caching** with Redis
- **Message Queuing** with RabbitMQ for async processing
- **High Availability and Fault Tolerance**
- **Advanced Error Handling** with circuit breakers
- **Metrics and Telemetry**
- **Load Balancing** with multiple API instances

## Technology Stack

- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL with sharding
- **Caching**: Redis
- **Message Queue**: RabbitMQ
- **Containerization**: Docker and Docker Compose
- **Load Balancing**: NGINX

## Architecture

The system uses a shard-based architecture with consistent hashing to distribute data across multiple database instances. Each shard has a leader-follower setup for replication and high availability.

### Key Components:

1. **API Layer**: Multiple Axum instances behind NGINX load balancer
2. **Service Layer**: Business logic and data access
3. **Database Layer**: Sharded PostgreSQL with leader-follower pattern
4. **Cache Layer**: Redis for high-performance reads
5. **Queue Layer**: RabbitMQ for asynchronous processing

## Getting Started

### Prerequisites

- Docker and Docker Compose
- Rust (for development)

### Running the System

1. Clone the repository
2. Start the system using Docker Compose:

```bash
docker-compose up -d
```

3. Access the API at http://localhost:8080

### Development Setup

1. Install Rust and dependencies:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install sqlx-cli
```

2. Set up environment:

```bash
cp .env.example .env
# Edit .env with your local configuration
```

3. Run the application locally:

```bash
cargo run
```

## API Endpoints

### Products

- `GET /api/products` - List all products
- `GET /api/products?category={category}` - List products by category
- `GET /api/products/{id}` - Get a specific product
- `POST /api/products` - Create a new product
- `PUT /api/products/{id}` - Update a product
- `DELETE /api/products/{id}` - Delete a product

### Orders

- `POST /api/orders` - Create a new order
- `GET /api/orders?customer_id={customer_id}` - List orders by customer
- `GET /api/orders/{id}` - Get a specific order
- `PUT /api/orders/{id}/status` - Update order status
- `POST /api/orders/{id}/cancel` - Cancel an order

### Health

- `GET /health` - System health check
- `GET /liveness` - Liveness probe for Kubernetes

## Monitoring

- Metrics are available at http://localhost:9090/metrics
- RabbitMQ management console is available at http://localhost:15672

## License

MIT
````

## Project Organization and Implementation Summary

This distributed e-commerce system demonstrates core distributed system concepts through a practical implementation. Here's a breakdown of the key components:

1. **Database Sharding with Consistent Hashing**:
   - The system distributes data across multiple PostgreSQL instances using consistent hashing
   - Products are sharded by category, while orders are sharded by customer_id
   - The `ConsistentHashSharding` implementation ensures even distribution and fault tolerance

2. **Leader-Follower Replication**:
   - The database layer implements a leader-follower pattern for replication
   - Write operations go to the leader database and are replicated to followers
   - The `ReplicationManager` handles event propagation using RabbitMQ

3. **Distributed Caching**:
   - Redis is used for distributed caching with TTL support
   - The `DistributedCache` provides high-performance read operations
   - Cache invalidation strategies maintain consistency across the system

4. **Message Queue for Async Processing**:
   - RabbitMQ manages order processing and system events
   - Async workflows handle tasks like order status updates
   - Message queues provide buffering for high-load scenarios

5. **High Availability and Fault Tolerance**:
   - Multiple API instances behind NGINX provide redundancy
   - Circuit breakers prevent cascading failures
   - Health checks detect system issues for recovery

6. **Production-Grade Features**:
   - Comprehensive error handling with proper HTTP responses
   - Metrics and telemetry using OpenTelemetry
   - Graceful shutdowns to preserve in-flight operations

The system is containerized using Docker and orchestrated with Docker Compose, making it easy to deploy and scale. The provided seed data demonstrates the system's functionality across different shards.

# Distributed Systems Concepts & Implementation in the E-commerce System

## 1. Data Sharding with Consistent Hashing

### Concept Overview
Data sharding is a horizontal partitioning technique that distributes data across multiple database instances to improve scalability and performance. Consistent hashing is an algorithm that determines which shard should store particular data, ensuring balanced distribution even when shards are added or removed.

### Implementation in the Code

The implementation revolves around the `ConsistentHashSharding` class in `src/utils/consistent_hash.rs`:

```rust
pub struct ConsistentHashSharding {
    ring: ConsistentHashRing<DbShard>,
    shard_count: usize,
}
```

The system uses a hash ring where database shards are distributed. Each entity (product, order) has a shard key:

```rust
// From src/db/sharding.rs
pub enum ShardKey {
    UserId(Uuid),
    ProductId(Uuid),
    OrderId(Uuid),
    ProductCategory(String),
    Raw(String),
}
```

When storing or retrieving data, we first determine the appropriate shard:

```rust
// From src/utils/consistent_hash.rs
pub fn get_shard_for_key<K: Hash>(&self, key: K) -> Option<&DbShard> {
    // Calculate hash of the key
    let mut hasher = XxHash64::default();
    key.hash(&mut hasher);
    let hash_value = hasher.finish();
    
    // Find the appropriate shard
    self.ring.get_node(&hash_value.to_string())
}
```

For products, we shard by category (or product ID if category is empty):

```rust
// From src/db/sharding.rs
impl Shardable for Product {
    fn get_shard_key(&self) -> ShardKey {
        match &self.category {
            Some(category) => ShardKey::ProductCategory(category.clone()),
            None => ShardKey::ProductId(self.id),
        }
    }
}
```

For orders, we shard by customer ID to keep a customer's orders together:

```rust
impl Shardable for Order {
    fn get_shard_key(&self) -> ShardKey {
        // Orders are sharded by customer_id for efficiency in customer-specific queries
        ShardKey::UserId(self.customer_id)
    }
}
```

### Database Setup for Sharding

The system uses four PostgreSQL instances as shards in the Docker Compose setup:

```yaml
# From docker-compose.yml
db-shard1:
    image: postgres:16
    container_name: ecommerce-db-shard1
    # Configuration...

db-shard2:
    image: postgres:16
    container_name: ecommerce-db-shard2
    # Configuration...

# And similar for db-shard3 and db-shard4
```

Each shard has the same schema but stores different data based on the sharding algorithm. The init script (`docker/db/init-shards.sh`) ensures all shards have the same structure:

```bash
# Create common tables in each shard
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db_name" <<-EOSQL
    -- Products table
    CREATE TABLE IF NOT EXISTS products (
        id UUID PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        # Additional fields...
    );
    
    # Other tables...
EOSQL
```

The `DbConnectionManager` manages connections to these shards:

```rust
// From src/db/connection.rs
pub struct DbConnectionManager {
    sharding: Arc<ConsistentHashSharding>,
    shard_pools: Arc<HashMap<u32, Pool>>,
    config: AppConfig,
}
```

## 2. Leader-Follower Replication

### Concept Overview
Leader-follower replication (also called primary-replica) is a database pattern where one database instance (leader) accepts write operations and replicates those changes to follower instances. This improves read scalability and provides redundancy.

### Implementation in the Code

The `ReplicationManager` in `src/db/replication.rs` handles this process:

```rust
pub struct ReplicationManager {
    conn_manager: DbConnectionManager,
    rabbitmq_client: Arc<RabbitMQClient>,
    replication_queue: String,
    event_buffer: Arc<Mutex<Vec<ReplicationEvent>>>,
}
```

When data is written to a leader database, a replication event is created:

```rust
// From src/db/replication.rs
pub async fn record_write_operation<'a>(
    &self,
    transaction: &mut Transaction<'a, Postgres>,
    sql: String,
    params: Option<serde_json::Value>,
    source_shard: u32,
    entity_type: String,
    entity_id: String,
    operation: String,
) -> Result<(), AppError> {
    // Create replication event
    let event = ReplicationEvent::new(
        sql, params, source_shard, entity_type, entity_id, operation
    );
    
    // Insert event to replication log table within the same transaction
    // ...
    
    // Add to buffer for async publishing after transaction commit
    self.event_buffer.lock().await.push(event);
    
    Ok(())
}
```

These events are published to RabbitMQ and consumed by a background worker:

```rust
async fn process_replication_event(
    conn_manager: &DbConnectionManager,
    data: &[u8],
) -> Result<PgQueryResult, AppError> {
    let event: ReplicationEvent = serde_json::from_slice(data)?;
    
    // Get all follower pools (non-leader pools)
    let leader_pools = conn_manager.get_all_leader_pools();
    let all_pools = conn_manager.get_all_pools();
    
    let follower_pools: Vec<_> = all_pools
        .into_iter()
        .filter(|pool| !leader_pools.contains(pool))
        .collect();
    
    // Apply the operation to all follower shards
    for pool in follower_pools {
        // Execute the SQL with parameters on the follower
        // ...
    }
    
    // Return the result
    // ...
}
```

### Database Setup for Replication

In the Docker Compose configuration, we designate which shards are leaders:

```yaml
# From docker-compose.yml (environment variables for API service)
- APP__DATABASE__SHARDS__0__IS_LEADER=true
- APP__DATABASE__SHARDS__1__IS_LEADER=true
- APP__DATABASE__SHARDS__2__IS_LEADER=false
- APP__DATABASE__SHARDS__3__IS_LEADER=false
```

The database also has a replication log table for tracking operations:

```sql
CREATE TABLE IF NOT EXISTS replication_log (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    sql_statement TEXT NOT NULL,
    params JSONB,
    source_shard INTEGER NOT NULL,
    entity_type VARCHAR(255) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    operation VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

## 3. Distributed Caching

### Concept Overview
Distributed caching stores frequently accessed data in memory to reduce database load and improve response times. In distributed systems, the cache is shared across multiple application instances.

### Implementation in the Code

The system uses Redis for distributed caching:

```rust
// From src/cache/redis.rs
pub struct RedisCache {
    pool: Pool,
    ttl: Duration,
}

pub struct DistributedCache {
    redis: Arc<RedisCache>,
}
```

The implementation provides cache operations with TTL (Time-To-Live) support:

```rust
// From src/cache/redis.rs
pub async fn set<T: Serialize + Send + Sync>(
    &self,
    key: &str,
    value: &T,
    ttl_override: Option<Duration>,
) -> Result<(), AppError> {
    let mut conn = self.get_connection().await?;
    let serialized = serde_json::to_string(value)?;
    
    let ttl = ttl_override.unwrap_or(self.ttl);
    
    redis::cmd("SET")
        .arg(key)
        .arg(serialized)
        .arg("EX")
        .arg(ttl.as_secs())
        .query_async(&mut conn)
        .await
        .map_err(AppError::Redis)
}
```

The `get_or_compute` method is particularly important for the cache-aside pattern:

```rust
// From src/cache/redis.rs
pub async fn get_or_compute<T, F, Fut>(
    &self,
    key: &str,
    compute_fn: F,
    ttl_override: Option<Duration>,
) -> Result<T, AppError>
where
    T: DeserializeOwned + Serialize + Send + Sync,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    // Try to get from cache first
    if let Some(cached) = self.redis.get::<T>(key).await? {
        return Ok(cached);
    }
    
    // If not found, compute the value
    let computed = compute_fn().await?;
    
    // Cache the result
    self.redis.set(key, &computed, ttl_override).await?;
    
    Ok(computed)
}
```

Cache invalidation is also implemented:

```rust
pub async fn invalidate_by_pattern(&self, pattern: &str) -> Result<(), AppError> {
    // ...
}
```

### Cache Service Setup

The Redis cache service is configured in the Docker Compose file:

```yaml
# From docker-compose.yml
redis:
    image: redis:7-alpine
    container_name: ecommerce-redis
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis-data:/data
    networks:
      - ecommerce-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      # Additional healthcheck configuration
```

Cache configuration is loaded from the app settings:

```rust
// From src/cache/redis.rs
pub async fn new(config: AppConfig) -> Result<Self, AppError> {
    let redis_config = RedisConfig::from_url(config.redis.url.clone());
    let pool = redis_config
        .create_pool(Some(Runtime::Tokio1))
        // ...
    
    Ok(Self {
        pool,
        ttl: Duration::from_secs(config.redis.ttl_seconds),
    })
}
```

Cache strategies are implemented in services like `ProductService`:

```rust
// From src/services/product_service.rs
pub async fn get_product(&self, id: Uuid) -> Result<Product, AppError> {
    let cache_key = format!("products:id:{}", id);
    
    // Try to get from cache or compute
    self.cache
        .get_or_compute(&cache_key, || async {
            let shard_key = ShardKey::ProductId(id);
            let pool = self.db.get_pool_for_key(&shard_key).await?;
            
            // Query from database if not in cache
            // ...
        }, None)
        .await
}
```

## 4. Message Queues for Async Processing

### Concept Overview
Message queues enable asynchronous communication between services, improving system resilience and scalability. They decouple components, allowing them to process messages at their own pace.

### Implementation in the Code

The system uses RabbitMQ for message queuing through the `RabbitMQClient` in `src/queue/rabbitmq.rs`:

```rust
pub struct RabbitMQClient {
    pool: Arc<LapinPool>,
    exchanges: Arc<Vec<String>>,
}
```

Messages are published to queues:

```rust
// From src/queue/rabbitmq.rs
pub async fn publish(
    &self,
    queue: &str,
    payload: &[u8],
) -> Result<Confirmation, AppError> {
    let conn = self.get_connection().await?;
    let channel = conn.create_channel().await?;

    // Declare queue to ensure it exists
    // ...
    
    // Publish message
    let confirmation = channel
        .basic_publish(
            "", // Default exchange
            queue,
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default().with_delivery_mode(2), // Persistent delivery
        )
        .await?
        .await?;
    
    // ...
    Ok(confirmation)
}
```

And consumed by asynchronous handlers:

```rust
pub async fn consume(
    &self,
    queue: &str,
    tx: mpsc::Sender<DeliveryHandler>,
) -> Result<(), AppError> {
    // Setup consumer
    // ...
    
    // Process messages
    tokio::spawn(async move {
        info!("Started consuming from queue: {}", queue);

        while let Some(delivery_result) = consumer.next().await {
            // Handle messages
            // ...
        }
    });
    
    Ok(())
}
```

The `OrderService` uses queues for order processing:

```rust
// From src/services/order_service.rs
// Send message to order processing queue
let message = OrderProcessingMessage {
    order_id: order.id,
    customer_id: order.customer_id,
    status: format!("{:?}", order.status),
    timestamp: chrono::Utc::now(),
};

let payload = serde_json::to_vec(&message)?;
self.rabbitmq.publish("order_processing", &payload).await?;
```

### Message Queue Setup

RabbitMQ is configured in the Docker Compose file:

```yaml
# From docker-compose.yml
rabbitmq:
    image: rabbitmq:3-management
    container_name: ecommerce-rabbitmq
    ports:
      - "5672:5672"
      - "15672:15672"
    volumes:
      - rabbitmq-data:/var/lib/rabbitmq
    networks:
      - ecommerce-network
    healthcheck:
      # Healthcheck configuration
```

The client setup initializes exchanges and connections:

```rust
// From src/queue/rabbitmq.rs
pub async fn new(config: AppConfig) -> Result<Self, AppError> {
    // Define exchanges
    let exchanges = vec![
        "orders".to_string(),
        "logs".to_string(),
        "events".to_string(),
    ];

    // Create connection pool
    // ...
    
    // Set up exchanges
    client.setup_exchanges().await?;

    Ok(client)
}
```

## 5. Additional Distributed System Concepts

### 5.1. Circuit Breakers

Circuit breakers prevent cascading failures by detecting and stopping repeated calls to failing components:

```rust
// From src/utils/circuit_breaker.rs
pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout_duration: Duration,
    reset_timeout: Duration,
    states: Mutex<HashMap<String, CircuitState>>,
    metrics: Mutex<HashMap<String, CircuitMetrics>>,
}
```

Implementation:

```rust
pub async fn call<F, Fut, T>(&self, operation: F, operation_name: String) -> Result<T, AppError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    // Check if circuit is open
    match self.get_circuit_state(&operation_name) {
        CircuitState::Open(opened_at) => {
            // Check if reset timeout has elapsed
            if opened_at.elapsed() > self.reset_timeout {
                // Transition to half-open
                // ...
            } else {
                // Circuit is still open, fail fast
                return Err(AppError::CircuitBreakerOpen(format!(
                    "Circuit breaker open for operation: {}",
                    operation_name
                )));
            }
        }
        _ => {} // Continue with the call for Closed or HalfOpen
    }
    
    // Execute with timeout
    // ...
}
```

### 5.2. Load Balancing

Multiple API instances are run behind an NGINX load balancer:

```yaml
# From docker-compose.yml
nginx:
    image: nginx:latest
    container_name: ecommerce-nginx
    ports:
      - "8080:80"
    volumes:
      - ./docker/nginx/nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - api1
      - api2
    networks:
      - ecommerce-network
```

The NGINX configuration distributes requests:

```conf
# From docker/nginx/nginx.conf
upstream api_servers {
    server api1:8080 max_fails=3 fail_timeout=30s;
    server api2:8080 max_fails=3 fail_timeout=30s;
    
    # Comment out below for round robin (default)
    # least_conn; # Use least connections algorithm
}
```

### 5.3. Health Checking and Monitoring

Health checks ensure system reliability:

```rust
// From src/utils/health.rs
pub struct HealthChecker {
    db: DbConnectionManager,
    redis: Arc<RedisCache>,
    rabbitmq: Arc<RabbitMQClient>,
    version: String,
}
```

Implementation:

```rust
pub async fn check_health(&self) -> HealthCheck {
    // Check database
    let db_health = self.check_database().await;
    
    // Check Redis
    let redis_health = self.check_redis().await;
    
    // Check RabbitMQ
    let rabbitmq_health = self.check_rabbitmq().await;
    
    // Determine overall status
    // ...
}
```

### 5.4. Metrics and Telemetry

OpenTelemetry provides metrics and tracing:

```rust
// From src/telemetry/metrics.rs
pub struct MetricsState {
    registry: Registry,
    request_counter: Counter<u64>,
    response_time: ValueRecorder<f64>,
}
```

Methods record metrics:

```rust
pub fn increment_request_counter(&self, method: &str, path: &str, status_code: u16) {
    self.request_counter.add(
        1,
        &[
            KeyValue::new("method", method.to_string()),
            KeyValue::new("path", path.to_string()),
            KeyValue::new("status", status_code.to_string()),
        ],
    );
}
```

## Putting It All Together

The distributed e-commerce system combines these concepts to achieve:

1. **Horizontal Scalability**: Through database sharding, multiple API instances, and load balancing
2. **High Availability**: Through leader-follower replication and health checking
3. **Resilience**: Through circuit breakers and message queues
4. **Performance**: Through distributed caching and optimized data access patterns

The entire system is orchestrated through Docker Compose, with each service properly configured to work together in a distributed manner. The system's modular design allows individual components to scale independently as needed.

This implementation demonstrates how modern distributed systems handle high traffic, maintain data consistency, and recover from failures while delivering a reliable user experience.
