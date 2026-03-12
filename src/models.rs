use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ClientSettings {
    pub port: String,
    pub host: String,
    pub container_name: String,
    pub user: String,
    pub password: String,
}

#[derive(Row, Deserialize)]
pub struct TableInfo {
    pub database: String,
    pub name: String,
    pub create_table_query: String,
    pub size: Option<String>,
}

pub enum TableType {
    Table,
    View,
}
