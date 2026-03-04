mod client;
mod config;
mod models;
mod traits;
use crate::{config::MigrationType, traits::Migrator};
use anyhow::{Context, Result, anyhow};
use clap::{CommandFactory, Parser};
use client::client_migrator::ClientMigrator;
use config::MigratorConfig;
use env_logger::Env;
use models::TableInfo;
use std::io::Write;

pub enum TableType {
    Table,
    View,
}

fn build_tables_query(database: &str, table_type: &TableType, table_name: &str) -> String {
    let mut table_filter = String::new();
    if !table_name.is_empty() {
        table_filter.push_str(&format!("AND name IN ('{}')", table_name));
    }

    match table_type {
        TableType::Table =>{format!("SELECT name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = '{}' AND engine {}  AND engine != 'Distributed' {}", database, "NOT LIKE '%View%'", &table_filter).to_string()},
        TableType::View => {format!("SELECT name, create_table_query FROM system.tables WHERE database = '{}' AND engine {}  AND engine != 'Distributed' {}", database, "LIKE '%View%'", &table_filter).to_string()},
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = MigratorConfig::parse();
    if config.migr_type == MigrationType::DockerToDocker && config.src_container.is_none() {
        let mut cmd = MigratorConfig::command();
        cmd.error(
            clap::error::ErrorKind::MissingRequiredArgument,
            "--src-container обязатален, если --migr-type docker-to-docker",
        )
        .exit();
    }

    // let config = MigratorConfig
    // {
    //     migr_type: config::MigrationType::InternalHostToDocker,
    //     src_host: "host.docker.internal".to_string(),
    //     src_tcp_port: "9000".to_string(),
    //     src_http_port: "8123".to_string(),
    //     src_user: "default".to_string(),
    //     src_password: Some("".to_string()),
    //     src_container: "clickhouse-test".to_string(),
    //     dst_host: "127.0.0.1".to_string(),
    //     dst_tcp_port: "9998".to_string(),
    //     dst_http_port: "8334".to_string(),
    //     dst_user: "user".to_string(),
    //     dst_password: Some("pwd".to_string()),
    //     dst_container: "clickhouse-test".to_string(),
    //     database: "sp".to_string()
    // };

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                record.args()
            )
        })
        .init();

    //simple_logger::init().unwrap();
    let database = config.database.clone();

    log::info!("Мигратор запущен");
    let client = ClientMigrator::new(config.clone());

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

    let tables = client
        .fetch::<TableInfo>(&build_tables_query(
            &config.database,
            &TableType::Table,
            &config.table_name.as_deref().unwrap_or(""),
        ))
        .await
        .context("Fetch data error")?;

    // let tables = remote
    //     .query("SELECT name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = 'sp' AND engine NOT LIKE '%View%' AND engine != 'Distributed' AND name = 'events'")
    //     .fetch_all::<TableInfo>()
    //     .await?;

    client.create_database(&database).await?;
    for row in tables {
        client.create_table(row).await?;
        client.transfer_data().await?;
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

    let view_up = false;

    if view_up {
        let views = client
            .fetch::<TableInfo>(&build_tables_query(
                &config.database,
                &TableType::View,
                &config.table_name.as_deref().unwrap_or(""),
            ))
            .await
            .context("Fetch data error")?;
        for v in views {
            client.create_table(v).await?;
        }
    }

    Ok(())
}
