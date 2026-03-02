use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Row, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub create_table_query: String,
    pub size: String,
}
