use anyhow::{Context, Result, anyhow};
use clickhouse::{Row, RowOwned};
use serde::de::DeserializeOwned;

use crate::config::MigratorConfig;

pub trait Migrator {
    async fn create_database(&self, db_name: &str) -> Result<()>;
    async fn execute_ddl(&self, ddl: &str) -> Result<()>;
    async fn transfer_data(&self, table: &str, size_table_label: &str, config:&MigratorConfig) -> Result<()>;

    async fn fetch<T>(&self, ddl: &str) -> Result<Vec<T>>
    where
        T: Row + RowOwned + DeserializeOwned + Send + Sync;
}
