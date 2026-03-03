mod client;
mod config;
mod models;
mod traits;

use crate::traits::Migrator;
use clap::Parser;
use client::client_migrator::ClientMigrator;
use config::MigratorConfig;
use models::TableInfo;
use anyhow::{Context, Result, anyhow};

#[tokio::main]
async fn main() -> Result<()> {
    let config = MigratorConfig::parse();
    simple_logger::init().unwrap();

    log::info!("Мигратор запущен");
    let client = ClientMigrator::new(config);

    // let remote = Client::default()
    //     .with_url("http://127.0.0.1:8123")
    //     .with_user("admin")
    //     .with_password("pwd")
    //     .with_compression(clickhouse::Compression::Lz4);

    // let locale = Client::default()
    //     .with_url("http://127.0.0.1:8333")
    //     .with_user("admin")
    //     .with_password("pwd")
    //     .with_compression(clickhouse::Compression::Lz4);

    let tables = client.fetch::<TableInfo>("SELECT name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = 'sp' AND engine NOT LIKE '%View%' AND engine != 'Distributed' AND name = 'events'").await.context("Fetch data error")?;

    // let tables = remote
    //     .query("SELECT name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = 'sp' AND engine NOT LIKE '%View%' AND engine != 'Distributed' AND name = 'events'")
    //     .fetch_all::<TableInfo>()
    //     .await?;

    for row in tables {

        client.create_table(row).await?;
        // let table_name = row.name;
        // let ddl = row.create_table_query;
        // let size = row.size;

        // let table_with_schema = ddl
        //     .split_ascii_whitespace()
        //     .find(|ch| ch.starts_with("sp."))
        //     .unwrap_or(&table_name);

        // log::info!("Таблица: {}", table_with_schema);
        // locale.query(&ddl).execute().await?;
        // log::info!("Таблица {} создана", table_name);
        // info!("  Создана");

        // fetch data
        // docker exec -i clickhouse-test clickhouse-client --host host.docker.internal --port 9000 --user default --password "" --query "select * from sp.events limit 10 format native" | docker exec -i clickhouse-test clickhouse-client --user user --password  "pwd" --query "insert into sp.events format native"
        // docker exec -i clickhouse-server-db clickhouse-client --host 127.0.0.1 --port 9000 --user admin --password "pwd" --query "select * from sp.events limit 10 format native" | docker exec -i clickhouse-server-l clickhouse-client --user admin --password  "pwd" --query "insert into sp.events format native"
        // log::info!(
        //     "Старт переноса данных в таблицу {} размер данных {}",
        //     table_name,
        //     size
        // );
    }

    Ok(())
}
