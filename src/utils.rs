use crate::models::TableType;
use env_logger::Env;
use std::io::Write;

pub fn configure_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.args()
            )
        })
        .init();
}

pub fn build_tables_query(
    database: &str,
    table_type: &TableType,
    table_name: &str,
    fetch_all: bool,
) -> String {
    let mut table_filter = String::new();
    if !table_name.is_empty() && !fetch_all {
        table_filter.push_str(&format!("AND name IN ('{}')", table_name));
    }

    match table_type {
        TableType::Table => format!(
            "SELECT database, name, create_table_query, formatReadableSize(total_bytes) size \
             FROM system.tables \
             WHERE database = '{}' AND engine {}  AND engine != 'Distributed' {}",
            database, "NOT LIKE '%View%'", table_filter
        )
        .trim_end()
        .to_string(),
        TableType::View => format!(
            "SELECT database, name, create_table_query, formatReadableSize(total_bytes) size \
             FROM system.tables \
             WHERE database = '{}' AND engine {} AND engine != 'Distributed' {}",
            database, "LIKE '%View%'", table_filter
        )
        .trim_end()
        .to_string(),
        TableType::MaterializedView => format!(
            "SELECT database, name, create_table_query, formatReadableSize(total_bytes) size \
            FROM system.tables \
            WHERE database = '{}' AND engine != 'Distributed'",
            database
        )
        .trim_end()
        .to_string(),
    }
}

pub fn table_name_with_schema(database: &str, table_name: &str) -> String {
    format!("{}.`{}`", database, table_name).to_string()
}

pub fn check_mv_tables(tables: &[&str]) -> bool {
    tables.iter().any(|t| t.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_contains_materialize_view_table() {
        let tables = vec!["normal_table", ".mv_table"];
        let result = check_mv_tables(&tables);

        assert_eq!(result, true);
    }

    #[test]
    fn build_tables_query_for_table_type_and_not_featch_all() {
        let database = "sp";
        let table_type = TableType::Table;
        let featch_all = false;
        let table_name = "test_table";

        let expected_query = format!(
            "SELECT database, name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = '{}' AND engine NOT LIKE '%View%'  AND engine != 'Distributed' AND name IN ('{}')",
            database, table_name
        );

        let result = build_tables_query(database, &table_type, table_name, featch_all);

        assert_eq!(result, expected_query)
    }

    #[test]
    fn build_tables_query_for_table_type_and_featch_all() {
        let database = "sp";
        let table_type = TableType::Table;
        let featch_all = true;
        let table_name = "test_table";

        let expected_query = format!(
            "SELECT database, name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = '{}' AND engine NOT LIKE '%View%'  AND engine != 'Distributed'",
            database
        );

        let result = build_tables_query(database, &table_type, table_name, featch_all);

        assert_eq!(result, expected_query)
    }

    #[test]
    fn build_tables_query_for_view_type_and_not_featch_all() {
        let database = "sp";
        let table_type = TableType::View;
        let featch_all = false;
        let table_name = "test_table";

        let expected_query = format!(
            "SELECT database, name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = '{}' AND engine LIKE '%View%' AND engine != 'Distributed' AND name IN ('{}')",
            database, table_name
        );

        let result = build_tables_query(database, &table_type, table_name, featch_all);

        assert_eq!(result, expected_query)
    }

    #[test]
    fn build_tables_query_for_view_type_featch_all() {
        let database = "sp";
        let table_type = TableType::View;
        let featch_all = true;
        let table_name = "test_table";

        let expected_query = format!(
            "SELECT database, name, create_table_query, formatReadableSize(total_bytes) size FROM system.tables WHERE database = '{}' AND engine LIKE '%View%' AND engine != 'Distributed'",
            database
        );

        let result = build_tables_query(database, &table_type, table_name, featch_all);

        assert_eq!(result, expected_query)
    }
}
