mod client;
mod config;
mod models;
mod traits;
use crate::{config::MigrationType, traits::Migrator};
use anyhow::{Context, Result, anyhow};
use clap::{CommandFactory, Parser};
use client::client_migrator::ClientMigrator;
use config::MigratorConfig;
use dotenvy::from_path;
use env_logger::Env;
use models::TableInfo;
use std::{
    env::{Args, args},
    io::{self, Write},
    process::exit,
};

pub enum TableType {
    Table,
    View,
}

fn configure_logger() {
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
}

fn parse_config() -> MigratorConfig {
    let mut config_argument = args()
    .skip_while(|arg| arg != "--config" && arg != "-c");

    if config_argument.next().is_some() {
        if let Some(path) = config_argument.next() {
            from_path(&path).expect("Файл конфигурации не найден");
        } else {
            eprintln!("Путь не указан");            
            exit(1);
        }
    }

    let config = MigratorConfig::parse();
    if config.migr_type == MigrationType::DockerToDocker && config.src_container.is_none() {
        let mut cmd = MigratorConfig::command();
        cmd.error(
            clap::error::ErrorKind::MissingRequiredArgument,
            "--src-container обязатален, если --migr-type docker-to-docker",
        )
        .exit();
    }

    config
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
    configure_logger();
    let config = parse_config();

    println!("Конфигурация:");
    println!("{:?}", config);

    let t = false;

    if t {
        let database = config.database.clone();

        log::info!("Мигратор запущен");
        let client = ClientMigrator::new(config.clone());

        let tables = client
            .fetch::<TableInfo>(&build_tables_query(
                &config.database,
                &TableType::Table,
                &config.table_name.as_deref().unwrap_or(""),
            ))
            .await
            .context("Fetch data error")?;

        client.create_database(&database).await?;
        for row in tables {
            client.create_table(row).await?;
            client.transfer_data().await?;
        }
    }

    let view_up = false;

    // if view_up {
    //     let views = client
    //         .fetch::<TableInfo>(&build_tables_query(
    //             &config.database,
    //             &TableType::View,
    //             &config.table_name.as_deref().unwrap_or(""),
    //         ))
    //         .await
    //         .context("Fetch data error")?;
    //     for v in views {
    //         client.create_table(v).await?;
    //     }
    // }

    Ok(())
}
