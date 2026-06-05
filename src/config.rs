use clap::{CommandFactory, Parser, ValueEnum};
use dotenvy::from_path;
use std::{env::args, process::exit};

#[derive(Clone, Copy, ValueEnum, Debug, PartialEq, Eq)]
pub enum MigrationType {
    DockerToDocker,
    InternalHostToDocker,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct MigratorConfig {
    /// Тип миграции
    #[arg(long, env = "MIGR_TYPE", value_enum, value_name = "type")]
    pub migr_type: MigrationType,

    /// HOST бд откуда копируются данные
    #[arg(
        long,
        env = "SRC_HOST",
        default_value = "host.docker.internal",
        value_name = "host"
    )]
    pub src_host: String,

    /// TCP PORT бд откуда копируются данные
    #[arg(
        long,
        env = "SRC_TCP_PORT",
        default_value = "9000",
        value_name = "port"
    )]
    pub src_tcp_port: String,

    /// HTTP PORT бд откуда копируются данные
    #[arg(
        long,
        env = "SRC_HTTP_PORT",
        default_value = "8123",
        value_name = "port"
    )]
    pub src_http_port: String,

    /// Username бд откуда копируются данные
    #[arg(long, env = "SRC_USER", default_value = "default", value_name = "user")]
    pub src_user: String,

    /// Password бд откуда копируются данные
    #[arg(
        long,
        env = "SRC_PASSWORD",
        default_value = "",
        value_name = "password"
    )]
    pub src_password: Option<String>,

    /// Имя контейнера бд откуда копируются данные (только для типа миграции: docker-to-docker)
    #[arg(long, env = "SRC_CONTAINER", value_name = "container")]
    pub src_container: Option<String>,

    /// HOST бд куда копирования данных
    #[arg(
        long,
        env = "DST_HOST",
        default_value = "127.0.0.1",
        value_name = "host"
    )]
    pub dst_host: String,

    /// TCP PORT бд куда копирования данных
    #[arg(long, env = "DST_TCP_PORT", value_name = "port")]
    pub dst_tcp_port: String,

    /// HTTP PORT бд куда копирования данных
    #[arg(long, env = "DST_HTTP_PORT", value_name = "port")]
    pub dst_http_port: String,

    /// Username бд куда копирования данных
    #[arg(long, env = "DST_USER", value_name = "user")]
    pub dst_user: String,

    /// Password бд куда копирования данных
    #[arg(long, env = "DST_PASSWORD", value_name = "password")]
    pub dst_password: Option<String>,

    /// Имя docker контейнера бд куда копируются данные
    #[arg(long, env = "DST_CONTAINER", value_name = "container")]
    pub dst_container: String,

    /// Имя базы данных
    #[arg(
        short,
        long,
        env = "DB_NAME",
        default_value = "sp",
        value_name = "database"
    )]
    pub database: String,

    /// Имя таблицы
    #[arg(short, long, env = "TABLE_NAME", value_name = "table")]
    pub table_name: Option<String>,

    /// Путь до файла конфига
    #[arg(short, long, value_name = "config")]
    pub config: Option<String>,

    /// Установить если нужно скопировать всю базу данных
    #[arg(
        long = "all",
        env = "FETCH_ALL",
        default_value_t = false,
        value_name = "fetch_all"
    )]
    pub fetch_all: bool,
}

impl MigratorConfig {
    pub fn parse_config() -> MigratorConfig {
        let mut config_argument = args().skip_while(|arg| arg != "--config" && arg != "-c");

        if config_argument.next().is_some() {
            if let Some(path) = config_argument.next() {
                from_path(&path).expect("Файл конфигурации не найден");
            } else {
                eprintln!("Путь не указан");
                exit(1);
            }
        }

        let config = MigratorConfig::parse();
        if config.migr_type == MigrationType::DockerToDocker && config.src_container.is_none() {
            let mut cmd = MigratorConfig::command();
            cmd.error(
                clap::error::ErrorKind::MissingRequiredArgument,
                "--src-container обязатален, если --migr-type docker-to-docker",
            )
            .exit();
        }

        config
    }
}
