use anyhow::Result;
use clickhouse_migrator::config::MigratorConfig;
use clickhouse_migrator::engine::run;
use clickhouse_migrator::utils;

#[tokio::main]
async fn main() -> Result<()> {
    utils::configure_logger();
    let config = MigratorConfig::parse_config();
    println!("{:?}", &config);
    run(config).await?;

    Ok(())
}
