use crate::client::docker_migrator::DockerMigrator;
use crate::config::MigratorConfig;
use crate::models::{TableInfo, TableType};
use crate::traits::Migrator;
use crate::utils;
use anyhow::{Context, Result};

pub async fn run(config: MigratorConfig) -> Result<()> {
    let database = config.database.clone();

    log::info!("Мигратор запущен");
    let client = DockerMigrator::new(config.clone());

    let tables = client
        .fetch::<TableInfo>(&utils::build_tables_query(
            &config.database,
            &TableType::Table,
            config.table_name.as_deref().unwrap_or(""),
            config.fetch_all,
        ))
        .await
        .context("Fetch data error")?;

    client.create_database(&database).await?;

    for row in tables {
        client.create_table(&row).await?;
        client.transfer_data(&row).await?;
    }

    let views = client
        .fetch::<TableInfo>(&utils::build_tables_query(
            &config.database,
            &TableType::View,
            config.table_name.as_deref().unwrap_or(""),
            config.fetch_all,
        ))
        .await
        .context("Fetch data error")?;

    for v in views {
        client.create_table(&v).await?;
    }

    Ok(())
}
