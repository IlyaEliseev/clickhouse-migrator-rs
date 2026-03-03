use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, ValueEnum, Debug, PartialEq, Eq)]
pub enum MigrationType {
    DockerToDoker,
    InternalHostToDocker,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct MigratorConfig {
    /// Тип миграции
    #[arg(long, value_enum)]
    pub migr_type: MigrationType,

    /// Источник: HOST (Env: SRC_HOST)
    #[arg(long, env = "SRC_URL", default_value = "host.docker.internal")]
    pub src_host: String,

    /// Источник: PORT (Env: SRC_PORT)
    #[arg(long, env = "SRC_URL", default_value = "9000")]
    pub src_port: String,

    /// Источник: User (Env: SRC_USER)
    #[arg(long, env = "SRC_USER", default_value = "default")]
    pub src_user: String,

    /// Источник: Password (Env: SRC_PASSWORD)
    #[arg(long, env = "SRC_PASSWORD")]
    pub src_password: Option<String>,

    /// Источник: Container (Env: SRC_CONTAINER)
    #[arg(long, env = "SRC_CONTAINER")]
    pub src_container: String,

    /// Цель: HOST (Env: DST_URL)
    #[arg(long, env = "DST_HOST", default_value = "127.0.0.1")]
    pub dst_host: String,

    /// Источник: PORT (Env: DST_PORT)
    #[arg(long, env = "DST_URL")]
    pub dst_port: String,

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
}
