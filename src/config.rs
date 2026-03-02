use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, ValueEnum, Debug, PartialEq, Eq)]
pub enum MigrationType {
    DockerToDoker,
    InternalHostToDocker,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct MigratorConfig {
    /// Тип миграции
    #[arg(long, value_enum)]
    pub migr_type: MigrationType,

    /// Источник: HOST (Env: SRC_HOST)
    #[arg(long, env = "SRC_URL", default_value = "127.0.0.1")]
    pub src_url: String,

    /// Источник: PORT (Env: SRC_PORT)
    #[arg(long, env = "SRC_URL", default_value = "9000")]
    pub src_port: String,

    /// Источник: User (Env: SRC_USER)
    #[arg(long, env = "SRC_USER", default_value = "admin")]
    pub src_user: String,

    /// Источник: Password (Env: SRC_PASSWORD)
    #[arg(long, env = "SRC_PASSWORD")]
    pub src_password: Option<String>,

    /// Источник: Container (Env: SRC_CONTAINER)
    #[arg(long, env = "SRC_CONTAINER", default_value = "clickhouse-server-db")]
    pub src_container: String,

    /// Цель: URL (Env: DST_URL)
    #[arg(long, env = "DST_URL", default_value = "http://127.0.0.1:8333")]
    pub dst_url: String,

    /// Цель: User (Env: DST_USER)
    #[arg(long, env = "DST_USER", default_value = "admin")]
    pub dst_user: String,

    /// Цель: Password (Env: DST_PASSWORD)
    #[arg(long, env = "DST_PASSWORD")]
    pub dst_password: Option<String>,

    /// Цель: Container (Env: DST_CONTAINER)
    #[arg(long, env = "DST_CONTAINER", default_value = "clickhouse-server-l")]
    pub dst_container: String,

    /// База данных (Env: DB_NAME)
    #[arg(short, long, env = "DB_NAME", default_value = "sp")]
    pub database: String,
}
