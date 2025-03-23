use config::AppConfig;

mod config;

#[tokio::main]
async fn main() {
    let app_config = AppConfig::load();
    // TODO: Initialize the application configuration here.
    if let Err(e) = distributed_catalog_system::start_service().await {}
}
