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
    #[arg(long, env = "MIGR_TYPE", value_enum)]
    pub migr_type: MigrationType,

    /// Источник: HOST (Env: SRC_HOST)
    #[arg(long, env = "SRC_HOST", default_value = "host.docker.internal")]
    pub src_host: String,

    /// Источник: TCP PORT (Env: SRC_TCP_PORT)
    #[arg(long, env = "SRC_TCP_PORT", default_value = "9000")]
    pub src_tcp_port: String,

    /// Источник: HTTP PORT (Env: SRC_HTTP_PORT)
    #[arg(long, env = "SRC_HTTP_PORT", default_value = "8123")]
    pub src_http_port: String,

    /// Источник: User (Env: SRC_USER)
    #[arg(long, env = "SRC_USER", default_value = "default")]
    pub src_user: String,

    /// Источник: Password (Env: SRC_PASSWORD)
    #[arg(long, env = "SRC_PASSWORD", default_value = "")]
    pub src_password: Option<String>,

    /// Источник: Container (Env: SRC_CONTAINER)
    #[arg(long, env = "SRC_CONTAINER")]
    pub src_container: Option<String>,

    /// Цель: HOST (Env: DST_URL)
    #[arg(long, env = "DST_HOST", default_value = "127.0.0.1")]
    pub dst_host: String,

    /// Источник: PORT (Env: DST_TCP_PORT)
    #[arg(long, env = "DST_TCP_PORT")]
    pub dst_tcp_port: String,

    /// Источник: PORT (Env: DST_HTTP_PORT)
    #[arg(long, env = "DST_HTTP_PORT")]
    pub dst_http_port: String,

    /// Цель: User (Env: DST_USER)
    #[arg(long, env = "DST_USER")]
    pub dst_user: String,

    /// Цель: Password (Env: DST_PASSWORD)
    #[arg(long, env = "DST_PASSWORD")]
    pub dst_password: Option<String>,

    /// Цель: Container (Env: DST_CONTAINER)
    #[arg(long, env = "DST_CONTAINER")]
    pub dst_container: String,

    /// База данных (Env: DB_NAME)
    #[arg(short, long, env = "DB_NAME", default_value = "sp")]
    pub database: String,

    /// Имя таблицы (Env: TABLE_NAME)
    #[arg(short, long, env = "TABLE_NAME")]
    pub table_name: Option<String>,

    /// Путь до файла конфига (записть в формате key=value)
    #[arg(short, long, env = "CONFIG")]
    pub config: Option<String>,
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
