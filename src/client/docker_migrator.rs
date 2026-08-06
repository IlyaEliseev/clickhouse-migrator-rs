use anyhow::{Context, Result, anyhow};
use clickhouse::{Client, Row, RowOwned};
use serde::de::DeserializeOwned;
use std::process::{Command, Stdio};

use crate::clickhouse_client_args_builder::ClickhouseArgs;
use crate::config::{MigrationType, MigratorConfig};
use crate::constants;
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
                constants::LOCALHOST,
                config.src_http_port
            ))
            .with_user(&config.src_user)
            .with_password(config.src_password.as_deref().unwrap_or(""))
            .with_compression(clickhouse::Compression::Lz4);

        let dst_client = Client::default()
            .with_url(format!(
                "http://{}:{}",
                constants::LOCALHOST,
                config.dst_http_port
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

    fn src_transfer_args(&self, table_info: &TableInfo) -> Result<Vec<String>> {
        let config = &self.config;

        let mut args = vec!["exec".to_string(), "-i".to_string()];

        match config.migr_type {
            MigrationType::InternalHostToDocker => args.push(config.dst_container.clone()),

            MigrationType::DockerToDocker => args.push(
                config
                    .src_container
                    .clone()
                    .as_deref()
                    .unwrap_or("")
                    .to_string(),
            ),
        };

        let table_name = table_name_with_schema(&table_info.database, &table_info.name);
        let select_script = format!("select * from {} format native", table_name);

        let client_args = ClickhouseArgs::create()
            .with_host(&config.src_host)
            .with_port(&config.src_tcp_port)
            .with_user(&config.src_user)
            .with_password(config.src_password.as_deref().unwrap_or(""))
            .with_query(select_script)
            .build()?;

        args.extend(client_args.to_array_args());

        Ok(args)
    }

    fn dest_transfer_args(&self, table_info: &TableInfo) -> Result<Vec<String>> {
        let config = &self.config;
        let table_name = table_name_with_schema(&table_info.database, &table_info.name);
        let insert_script = format!("insert into {} format native", table_name);

        let client_args = ClickhouseArgs::create()
            .with_host(&config.src_host)
            .with_port(&config.src_tcp_port)
            .with_user(&config.src_user)
            .with_password(config.src_password.as_deref().unwrap_or(""))
            .with_query(insert_script)
            .build()?;

        let mut args = vec![
            "exec".to_string(),
            "-i".to_string(),
            config.dst_container.clone(),
        ];

        args.extend(client_args.to_array_args());

        Ok(args)
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
        let src_args = &self.src_transfer_args(table_info)?;
        let dst_args = &self.dest_transfer_args(table_info)?;

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

        let status_dest = dest_proc.wait()?;
        let status_src = source_proc.wait()?;

        if !status_src.success() {
            return Err(anyhow!("Ошибка при переносе данных"));
        }

        if status_dest.success() {
            log::info!("Данные перенесены");
            Ok(())
        } else {
            Err(anyhow!("Ошибка при переносе данных {}", table_info.name))
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

        match self.dst_client.query(ddl).execute().await {
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
                    self.dst_client.query(ddl).execute().await?;
                    log::info!("Таблица {} пересоздана", table_with_schema);

                    Ok(())
                } else {
                    Err(anyhow::Error::new(e))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_docker_migrator(migr_type: MigrationType) -> DockerMigrator {
        DockerMigrator {
            src_client: Client::default(),
            dst_client: Client::default(),
            config: MigratorConfig {
                migr_type,
                src_host: "localhost".to_string(),
                src_tcp_port: "9000".to_string(),
                src_http_port: "8123".to_string(),
                src_user: "default".to_string(),
                src_password: Some("".to_string()),
                src_container: Some("src_container".to_string()),
                dst_host: "localhost".to_string(),
                dst_tcp_port: "9001".to_string(),
                dst_http_port: "8124".to_string(),
                dst_user: "default".to_string(),
                dst_password: Some("".to_string()),
                dst_container: "dst_container".to_string(),
                database: "db".to_string(),
                table_name: Some("tb_name".to_string()),
                config: Some("".to_string()),
                fetch_all: false,
            },
        }
    }

    fn create_test_table_info() -> TableInfo {
        TableInfo {
            database: "db".to_string(),
            name: "table".to_string(),
            create_table_query: "".to_string(),
            size: Some("".to_string()),
        }
    }

    #[test]
    fn internal_host_to_docker_src_transfer_args() {
        let docker_migrator = create_test_docker_migrator(MigrationType::InternalHostToDocker);

        let table_info = create_test_table_info();

        let args = docker_migrator.src_transfer_args(&table_info).unwrap();

        assert_eq!(args[0], "exec");
        assert_eq!(args[1], "-i");
        assert_eq!(args[2], docker_migrator.config.dst_container);
    }

    #[test]
    fn docker_to_docker_src_transfer_args() {
        let docker_migrator = create_test_docker_migrator(MigrationType::DockerToDocker);
        let table_info = create_test_table_info();

        let args = docker_migrator.src_transfer_args(&table_info).unwrap();

        assert_eq!(args[0], "exec");
        assert_eq!(args[1], "-i");
        assert_eq!(args[2], docker_migrator.config.src_container.unwrap());
    }

    #[test]
    fn dest_transfer_args() {
        let docker_migrator = create_test_docker_migrator(MigrationType::DockerToDocker);
        let table_info = create_test_table_info();

        let args = docker_migrator.dest_transfer_args(&table_info).unwrap();

        assert_eq!(args[0], "exec");
        assert_eq!(args[1], "-i");
        assert_eq!(args[2], docker_migrator.config.dst_container);
    }
}
