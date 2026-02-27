use std::thread;
use std::time::Duration;

use clickhouse::{Client, Row};
use clickhouse::{error::Result, sql::Identifier};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle, ProgressIterator};
use log::{Level, debug, error, info, log_enabled};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, Receiver, error::TryRecvError},
    time::timeout,
};
use tokio_util::io::StreamReader;

#[derive(Row, Deserialize)]
struct TableInfo {
    name: String,
    create_table_query: String,
}

#[derive(Row, Deserialize)]
struct ColumnInfo {
    col: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Мигратор запущен");

    let pb = ProgressBar::new(100);
    for _ in (0..100).progress(){ thread::sleep(Duration::from_secs(1));};

    
    let remote = Client::default()
        .with_url("http://127.0.0.1:8123")
        .with_user("default")
        .with_password("")
        .with_compression(clickhouse::Compression::Lz4);

    let locale = Client::default()
        .with_url("http://127.0.0.1:8334")
        .with_user("user")
        .with_password("pwd")
        .with_compression(clickhouse::Compression::Lz4);

    let tables = remote
        .query("SELECT name, create_table_query FROM system.tables WHERE database = 'sp' AND engine NOT LIKE '%View%' AND engine != 'Distributed' AND name = 'events'")
        .fetch_all::<TableInfo>()
        .await?;

    for row in tables {
        let table_name = row.name;
        let ddl = row.create_table_query;

        let table_with_schema = ddl
            .split_ascii_whitespace()
            .find(|ch| ch.starts_with("sp."))
            .unwrap_or(&table_name);

        // let columns_info = remote
        //     .query("SELECT if(type LIKE 'LowCardinality%', concat('CAST(', name, ' AS String) AS ', name), name) as col
        //             FROM system.columns WHERE database = 'sp' AND table = ? ORDER BY position")
        //     .bind(&table_name)
        //     .fetch_all::<ColumnInfo>()
        //     .await?;

        info!("Таблица: {}", table_with_schema);
        //locale.query(&ddl).execute().await?;
        info!("  Создана");

        // let columns_str = columns_info.into_iter().map(|c| c.col).collect::<Vec<_>>().join(", ");
        // let mut cursor = remote.query(&format!("SELECT {} FROM {} SETTINGS max_block_size = 10000", columns_str, table_with_schema)).fetch_bytes("RowBinary").unwrap();

        // while let Some(c) = cursor.next().await? {
        //     locale.insert_formatted_with(table_with_schema).send(c);
        // }

        // // 3. Перенос данных
        // // В официальном клиенте для прямого копирования блоков "как есть"
        // // через generic-интерфейс лучше использовать RowBinary.
        // let mut cursor = remote
        //     .query(&format!("SELECT {} FROM {} SETTINGS max_block_size = 10000", columns_str, table_with_schema))
        //     .fetch::<serde_json::Value>()?; // Используем Value для динамических данных, если структура заранее неизвестна

        // let mut insert = locale.insert(table_with_schema)?;
        // while let Some(row) = cursor.next().await? {
        //     insert.write(&row).await?;
        //     println!("  Строка записана");
        // }
        // insert.end().await?;

        // locale.query(&format!("OPTIMIZE TABLE {} FINAL", table_with_schema)).execute().await?;
    
    // fetch data
    // docker exec -i clickhouse-test clickhouse-client --host host.docker.internal --port 9000 --user default --password "" --query "select * from sp.events limit 10 format native" | docker exec -i clickhouse-test clickhouse-client --user user --password  "pwd" --query "insert into sp.events format native"
    }

    Ok(())
}
