use std::process::{Command, Stdio};

use crate::{
    config::{MigrationType, MigratorConfig},
    models::TableInfo,
    traits::Migrator,
};
use anyhow::{Context, Result, anyhow};
use clickhouse::{Client, Row, RowOwned};
use serde::de::DeserializeOwned;


pub struct ClientMigrator {
    src_client: Client,
    dst_client: Client,
    config: MigratorConfig,
}

impl ClientMigrator {
    pub fn new(config: MigratorConfig) -> Self {
        let src_client = Client::default()
            .with_url(format!("http://{}:{}", &config.src_host, &config.src_port))
            .with_user(&config.src_user)
            .with_password(config.src_password.as_deref().unwrap_or(""))
            .with_compression(clickhouse::Compression::Lz4);

        let dst_client = Client::default()
            .with_url(format!("http://{}:{}", &config.dst_host, &config.dst_port))
            .with_user(&config.dst_user)
            .with_password(config.dst_password.as_deref().unwrap_or(""))
            .with_compression(clickhouse::Compression::Lz4);

        Self {
            src_client,
            dst_client,
            config,
        }
    }

    fn src_transfer_args(
        migr_type: MigrationType,
        container_name: &str,
        host: &str,
        port: &str,
        user: &str,
        password: &str,
    ) -> Vec<String> {
        match migr_type {
            MigrationType::InternalHostToDocker => vec![
                "exec".to_string(),
                "-i".to_string(),
                container_name.to_string(),
                "clickhouse-client".to_string(),
                "--host".to_string(),
                "host.docker.internal".to_string(),
                "--port".to_string(),
                port.to_string(),
                "--user".to_string(),
                user.to_string(),
                "--password".to_string(),
                password.to_string(),
                "--query".to_string(),
                "select * from sp.events limit 10 format native".to_string(),
            ],

            MigrationType::DockerToDoker => vec![
                "exec".to_string(),
                "-i".to_string(),
                container_name.to_string(),
                "clickhouse-client".to_string(),
                "--host".to_string(),
                host.to_string(),
                "--port".to_string(),
                port.to_string(),
                "--user".to_string(),
                user.to_string(),
                "--password".to_string(),
                password.to_string(),
                "--query".to_string(),
                "select * from sp.events limit 10 format native".to_string(),
            ],
        }
    }

    fn dest_transfer_args(
        migr_type: &MigrationType,
        container_name: &str,
        host: &str,
        port: &str,
        user: &str,
        password: &str,
    ) -> Vec<String> {
        match migr_type {
            MigrationType::InternalHostToDocker => vec![
                "exec".to_string(),
                "-i".to_string(),
                container_name.to_string(),
                "clickhouse-client".to_string(),
                "--user".to_string(),
                user.to_string(),
                "--password".to_string(),
                password.to_string(),
                "--query".to_string(),
                "insert into sp.events format native".to_string(),
            ],

            MigrationType::DockerToDoker => vec![
                "exec".to_string(),
                "-i".to_string(),
                container_name.to_string(),
                "clickhouse-client".to_string(),
                "--user".to_string(),
                user.to_string(),
                "--password".to_string(),
                password.to_string(),
                "--query".to_string(),
                "insert into sp.events format native".to_string(),
            ],
        }
    }
}

impl Migrator for ClientMigrator {
    async fn create_database(&self, db_name: &str) -> Result<()> {
        self.dst_client.query(&format!("CREATE DATABASE IF NOT EXISTS {}", db_name)).execute().await?;
        Ok(())
    }

    async fn transfer_data(
        &self,
        table: &str,
        size_table_label: &str,
        config: &MigratorConfig,
    ) -> Result<()> {
        let src_args = ClientMigrator::src_transfer_args(
            config.migr_type,
            &config.src_container,
            &config.src_host,
            &config.src_port,
            &config.src_user,
            &config.src_password.as_deref().unwrap_or(""),
        );

        let dst_args = ClientMigrator::src_transfer_args(
            config.migr_type,
            &config.dst_container,
            &config.dst_host,
            &config.dst_port,
            &config.dst_user,
            &config.dst_password.as_deref().unwrap_or(""),
        );

        let mut source_proc = Command::new("docker")
            .args(&src_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Export error")?;

        let source_stdout = source_proc
            .stdout
            .take()
            .ok_or_else(|| anyhow!("failed to open stdout"))?;

        let mut dest_proc = Command::new("docker")
            .args(&dst_args)
            .stdin(source_stdout)
            .stderr(Stdio::inherit())
            .spawn()
            .context("Import error")?;

        let _ = source_proc.wait()?;
        let status_dest = dest_proc.wait()?;

        if status_dest.success() {
            println!("Данные перенесены");
            Ok(())
        } else {
            Err(anyhow!("Ошибка при переносе данных {}", table))
        }
    }

    async fn fetch<T>(&self, ddl: &str) -> Result<Vec<T>>
    where
        T: Row + RowOwned + DeserializeOwned + Send + Sync,
    {
        let res = self.src_client.query(ddl).fetch_all::<T>().await?;

        Ok(res)
    }

    async fn create_table(&self, table_info: TableInfo) -> Result<()> {
        let table_name = table_info.name;
        let ddl = table_info.create_table_query;

        let table_with_schema = ddl
            .split_ascii_whitespace()
            .find(|ch| ch.starts_with("sp."))
            .unwrap_or(&table_name);

        log::info!("Таблица: {}", table_with_schema);

        match self.dst_client.query(&ddl).execute().await {
            Ok(_) => {
                log::info!("Таблица {} создана", table_name);
                Ok(())
            }
            Err(e) => {
                if e.to_string().contains("57") {
                    self.dst_client
                        .query(&format!("drop table {}", table_with_schema))
                        .execute()
                        .await?;
                    self.dst_client.query(&ddl).execute().await?;
                    log::info!("Таблица {} пересоздана", table_with_schema);

                    Ok(())
                } else {
                    Err(anyhow::Error::new(e))
                }
            }
        }
    }
}
