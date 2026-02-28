use std::thread;
use std::time::Duration;

use clickhouse::error::Result;
use clickhouse::{Client, Row};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use log::{Level, debug, error, info, log_enabled};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, Receiver, error::TryRecvError},
    time::timeout,
};

use std::process::{Command, Stdio};  

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

    // let pb = ProgressBar::new(100);
    // for _ in (0..100).progress() {
    //     thread::sleep(Duration::from_secs(1));
    // }

    let remote = Client::default()
        .with_url("http://127.0.0.1:8123")
        .with_user("admin")
        .with_password("pwd")
        .with_compression(clickhouse::Compression::Lz4);

    let locale = Client::default()
        .with_url("http://127.0.0.1:8333")
        .with_user("admin")
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

        // info!("Таблица: {}", table_with_schema);
        // locale.query(&ddl).execute().await?;
        println!("Таблица {} создана", table_name);
        // info!("  Создана");

        // fetch data
        // docker exec -i clickhouse-test clickhouse-client --host host.docker.internal --port 9000 --user default --password "" --query "select * from sp.events limit 10 format native" | docker exec -i clickhouse-test clickhouse-client --user user --password  "pwd" --query "insert into sp.events format native"
        // docker exec -i clickhouse-server-db clickhouse-client --host 127.0.0.1 --port 9000 --user admin --password "pwd" --query "select * from sp.events limit 10 format native" | docker exec -i clickhouse-server-l clickhouse-client --user admin --password  "pwd" --query "insert into sp.events format native"
        println!("Старт переноса данных в таблицу {}", table_name);
        
        let mut source_proc = Command::new("docker")
            .args([
                "exec", 
                "-i", 
                "clickhouse-server-db",
                "clickhouse-client",
                "--host", "127.0.0.1",
                "--port", "9000",
                "--user", "admin",
                "--password", "pwd",
                "--query", "select * from sp.events limit 10 format native"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let source_stdout = source_proc.stdout.take().ok_or("failed to open stdout")?;

        let mut dest_proc = Command::new("docker")
            .args([
                "exec", 
                "-i", 
                "clickhouse-server-l", 
                "clickhouse-client",
                "--user", "admin",
                "--password", "pwd",
                "--query", "insert into sp.events format native"
            ])
            .stdin(source_stdout)
            .stderr(Stdio::inherit())
            .spawn()?;

        let _ = source_proc.wait()?;
        let status_dest = dest_proc.wait()?;

        if status_dest.success() {
            println!("Данные перенесены");
        }
        else {
            println!("Ошибка при переносе данных");
        }
    }

    Ok(())
}
