use crate::models::TableInfo;
use anyhow::Result;
use clickhouse::{Row, RowOwned};
use serde::de::DeserializeOwned;

pub trait Migrator {
    async fn create_database(&self, db_name: &str) -> Result<()>;
    async fn transfer_data(&self, table_info: &TableInfo) -> Result<()>;
    async fn create_table(&self, table_info: &TableInfo) -> Result<()>;
    async fn fetch<T>(&self, ddl: &str) -> Result<Vec<T>>
    where
        T: Row + RowOwned + DeserializeOwned + Send + Sync;
}
