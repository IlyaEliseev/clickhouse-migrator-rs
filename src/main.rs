use clickhouse_migrator::config::MigratorConfig;
use clickhouse_migrator::engine::run;
use clickhouse_migrator::utils;
use anyhow::{Result};

#[tokio::main]
async fn main() -> Result<()> {
    utils::configure_logger();
    let config = MigratorConfig::parse_config();
    println!("Конфигурация:");
    println!("{:?}", config);

    run(config).await?;

    Ok(())
}
