use anyhow::{Context, Result, anyhow};
use clickhouse::{Client, Row, RowOwned};
use serde::de::DeserializeOwned;
use std::process::{Command, Stdio};

use crate::config::{MigrationType, MigratorConfig};
use crate::models::TableInfo;
use crate::traits::Migrator;
use crate::utils::table_name_with_schema;

pub struct DockerMigrator {
    src_client: Client,
    dst_client: Client,
    config: MigratorConfig,
}

impl DockerMigrator {
    pub fn new(config: MigratorConfig) -> Self {
        let src_client = Client::default()
            .with_url(format!(
                "http://{}:{}",
                &config.src_host, &config.src_http_port
            ))
            .with_user(&config.src_user)
            .with_password(config.src_password.as_deref().unwrap_or(""))
            .with_compression(clickhouse::Compression::Lz4);

        let dst_client = Client::default()
            .with_url(format!(
                "http://{}:{}",
                &config.dst_host, &config.dst_http_port
            ))
            .with_user(&config.dst_user)
            .with_password(config.dst_password.as_deref().unwrap_or(""))
            .with_compression(clickhouse::Compression::Lz4);

        Self {
            src_client,
            dst_client,
            config,
        }
    }

    fn src_transfer_args(&self, table_info: &TableInfo) -> Vec<String> {
        let config = &self.config;

        let mut args = vec!["exec", "-i"];

        match config.migr_type {
            MigrationType::InternalHostToDocker => args.push(&config.dst_container),

            MigrationType::DockerToDocker => {
                args.push(config.src_container.as_deref().unwrap_or(""))
            }
        };

        let table_name = table_name_with_schema(&table_info.database, &table_info.name);
        let select_script = format!("select * from {} format native", table_name);

        args.extend([
            "clickhouse-client",
            "--host",
            &config.src_host,
            "--port",
            &config.src_tcp_port,
            "--user",
            &config.src_user,
            "--password",
            config.src_password.as_deref().unwrap_or(""),
            "--query",
            &select_script,
        ]);

        args.into_iter().map(String::from).collect()
    }

    fn dest_transfer_args(&self, table_info: &TableInfo) -> Vec<String> {
        let config = &self.config;
        let table_name = table_name_with_schema(&table_info.database, &table_info.name);
        let insert_script = format!("insert into {} format native", table_name);

        vec![
            "exec",
            "-i",
            &config.dst_container,
            "clickhouse-client",
            "--user",
            &config.dst_user,
            "--password",
            &config.dst_password.as_deref().unwrap_or(""),
            "--query",
            &insert_script,
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

impl Migrator for DockerMigrator {
    async fn create_database(&self, db_name: &str) -> Result<()> {
        self.dst_client
            .query(&format!("CREATE DATABASE IF NOT EXISTS {}", db_name))
            .execute()
            .await?;
        Ok(())
    }

    async fn transfer_data(&self, table_info: &TableInfo) -> Result<()> {
        let src_args = &self.src_transfer_args(&table_info);
        let dst_args = &self.dest_transfer_args(&table_info);

        let mut source_proc = Command::new("docker")
            .args(src_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Export error")?;

        let source_stdout = source_proc
            .stdout
            .take()
            .ok_or_else(|| anyhow!("failed to open stdout"))?;

        let mut dest_proc = Command::new("docker")
            .args(dst_args)
            .stdin(source_stdout)
            .stderr(Stdio::inherit())
            .spawn()
            .context("Import error")?;

        let _ = source_proc.wait()?;
        let status_dest = dest_proc.wait()?;

        if status_dest.success() {
            log::info!("Данные перенесены");
            Ok(())
        } else {
            Err(anyhow!("Ошибка при переносе данных {}", "events"))
        }
    }

    async fn fetch<T>(&self, ddl: &str) -> Result<Vec<T>>
    where
        T: Row + RowOwned + DeserializeOwned + Send + Sync,
    {
        let res = self.src_client.query(ddl).fetch_all::<T>().await?;

        Ok(res)
    }

    async fn create_table(&self, table_info: &TableInfo) -> Result<()> {
        let database = &table_info.database;
        let table_name = &table_info.name;
        let ddl = &table_info.create_table_query;
        let table_with_schema = table_name_with_schema(database, table_name);

        match self.dst_client.query(&ddl).execute().await {
            Ok(_) => {
                log::info!("Таблица {} создана", table_name);
                Ok(())
            }
            Err(e) => {
                if e.to_string().contains("57") {
                    log::info!(
                        "Таблица: {} уже сушествует и будет пересоздана",
                        table_with_schema
                    );
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
