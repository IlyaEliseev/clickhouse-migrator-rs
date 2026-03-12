use crate::models::TableType;
use env_logger::Env;
use std::io::Write;

pub fn configure_logger() {
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

pub fn build_tables_query(database: &str, table_type: &TableType, table_name: &str) -> String {
    let mut table_filter = String::new();
    if !table_name.is_empty() {
        table_filter.push_str(&format!("AND name IN ('{}')", table_name));
    }

    match table_type {
        TableType::Table =>{format!("SELECT database, name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = '{}' AND engine {}  AND engine != 'Distributed' {}", database, "NOT LIKE '%View%'", &table_filter).to_string()},
        TableType::View => {format!("SELECT database, name, create_table_query FROM system.tables WHERE database = '{}' AND engine {}  AND engine != 'Distributed' {}", database, "LIKE '%View%'", &table_filter).to_string()},
    }
}

pub fn table_name_with_schema(database:&str, table_name:&str) -> String {
    format!("{}.`{}`", database, table_name).to_string()
}
