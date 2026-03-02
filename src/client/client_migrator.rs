use std::process::{Command, Stdio};

use crate::{config::{MigrationType, MigratorConfig}, traits::Migrator};
use anyhow::{Context, Result, anyhow};
use clickhouse::{Client, Row, RowOwned, RowRead};
use serde::de::DeserializeOwned;

struct TransferArgs {
    dest: Vec<String>,
    src: Vec<String>
}

pub struct ClientMigrator {
    client: Client,
    migr_type: MigrationType,
}

impl ClientMigrator {
    pub fn new(ulr: &str, password: &str, user: &str, migr_type: MigrationType) -> Self {
        let client = Client::default()
            .with_url(ulr)
            .with_user(user)
            .with_password(password)
            .with_compression(clickhouse::Compression::Lz4);

        Self { client, migr_type }
    }

    // fn transfer_args(migr_type: MigrationType, container_name:&str, host:&str, port:&str, user:&str, password:&str) -> TransferArgs {
    //     let mut src_args = vec!["exec".to_string(), "-i".to_string()];
    //     let mut dest_args = vec!["exec".to_string(), "-i".to_string()];

    //     TransferArgs { dest: (), src: () }
    // }

    fn src_transfer_args(migr_type: MigrationType, container_name:&str, host:&str, port:&str, user:&str, password:&str) -> Vec<String> {
        match migr_type {
            MigrationType::InternalHostToDocker => vec![
                "exec".to_string(), 
                "-i".to_string(), 
                container_name.to_string(), 
                "clickhouse-client".to_string(), 
                "--host".to_string(), "host.docker.internal".to_string(), 
                "--port".to_string(), port.to_string(), 
                "--user".to_string(), user.to_string(), 
                "--password".to_string(), password.to_string(), 
                "--query".to_string(), "select * from sp.events limit 10 format native".to_string()],
            
            MigrationType::DockerToDoker => vec![
                "exec".to_string(),
                "-i".to_string(),
                container_name.to_string(),
                "clickhouse-client".to_string(),
                "--host".to_string(), host.to_string(),
                "--port".to_string(), port.to_string(),
                "--user".to_string(), user.to_string(),
                "--password".to_string(), password.to_string(),
                "--query".to_string(), "select * from sp.events limit 10 format native".to_string()],
        }
    }

    fn dest_transfer_args(migr_type: &MigrationType, container_name:&str, host:&str, port:&str, user:&str, password:&str) -> Vec<String> {
        match migr_type {
            MigrationType::InternalHostToDocker => vec![
                "exec".to_string(),
                "-i".to_string(),
                container_name.to_string(),
                "clickhouse-client".to_string(),
                "--user".to_string(), user.to_string(),
                "--password".to_string(), password.to_string(),
                "--query".to_string(), "insert into sp.events format native".to_string()],

            MigrationType::DockerToDoker => vec![
                "exec".to_string(),
                "-i".to_string(),
                container_name.to_string(),
                "clickhouse-client".to_string(),
                "--user".to_string(), user.to_string(),
                "--password".to_string(), password.to_string(),
                "--query".to_string(), "insert into sp.events format native".to_string()],
        }
    }
}

impl Migrator for ClientMigrator {
    async fn create_database(&self, db_name: &str) -> Result<()> {
        todo!()
    }

    async fn execute_ddl(&self, ddl: &str) -> Result<()> {
        todo!()
    }

    async fn transfer_data(&self, table: &str, size_table_label: &str, config:&MigratorConfig) -> Result<()> {
        let src_args = ClientMigrator::src_transfer_args(
            config.migr_type, 
            &config.src_container, host, port, user, password);

        let mut source_proc = Command::new("docker")
            .args([
                "exec",
                "-i",
                "clickhouse-server-db",
                "clickhouse-client",
                "--host",
                "127.0.0.1",
                "--port",
                "9000",
                "--user",
                "admin",
                "--password",
                "pwd",
                "--query",
                "select * from sp.events limit 10 format native",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Export error")?;

        let source_stdout = source_proc
            .stdout
            .take()
            .ok_or_else(|| anyhow!("failed to open stdout"))?;

        let mut dest_proc = Command::new("docker")
            .args([
                "exec",
                "-i",
                "clickhouse-server-l",
                "clickhouse-client",
                "--user",
                "admin",
                "--password",
                "pwd",
                "--query",
                "insert into sp.events format native",
            ])
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
        let res = self.client.query(ddl).fetch_all::<T>().await?;

        Ok(res)
    }
}
