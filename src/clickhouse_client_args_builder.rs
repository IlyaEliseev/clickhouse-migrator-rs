use anyhow::{Context, Result};

#[derive(Debug)]
pub struct ClickhouseArgs {
    client_command: String,
    port: String,
    host: String,
    user: String,
    password: String,
    query: String,
}

pub struct ClickhouseArgsBuilder {
    client_command: String,
    port: Option<String>,
    host: Option<String>,
    user: Option<String>,
    password: Option<String>,
    query: Option<String>,
}

impl ClickhouseArgs {
    pub fn create() -> ClickhouseArgsBuilder {
        let client_command = String::from("clickhouse-client");
        ClickhouseArgsBuilder {
            client_command,
            port: None,
            host: None,
            user: None,
            password: None,
            query: None,
        }
    }

    pub fn to_array_args(self) -> Vec<String> {
        vec![
            self.client_command,
            String::from("--host"),
            self.host,
            String::from("--port"),
            self.port,
            String::from("--user"),
            self.user,
            String::from("--password"),
            self.password,
            String::from("--query"),
            self.query,
        ]
    }
}

impl ClickhouseArgsBuilder {
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn with_port(mut self, port: impl Into<String>) -> Self {
        self.port = Some(port.into());
        self
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn build(self) -> Result<ClickhouseArgs> {
        let port = self
            .port
            .context("Поле 'port' обязательно для заполнения!")?;
        let host = self
            .host
            .context("Поле 'host' обязательно для заполнения!")?;
        let user = self
            .user
            .context("Поле 'user' обязательно для заполнения!")?;
        let query = self
            .query
            .context("Поле 'query' обязательно для заполнения!")?;
        let password = self
            .password
            .context("Поле 'password' обязательно для заполнения!")?;

        let client_command = self.client_command;

        Ok(ClickhouseArgs {
            client_command,
            port,
            host,
            user,
            password,
            query,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_build_with_all_args() {
        let client = ClickhouseArgs::create()
            .with_host("host")
            .with_password("password")
            .with_port("port")
            .with_query("query")
            .with_user("user")
            .build();

        assert!(client.is_ok());

        let result = client.unwrap();
        assert_eq!(result.port, "port");
        assert_eq!(result.host, "host");
        assert_eq!(result.user, "user");
        assert_eq!(result.password, "password");
        assert_eq!(result.query, "query");
    }

    #[test]
    fn not_successful_build_without_host() {
        let client = ClickhouseArgs::create()
            .with_password("password")
            .with_port("port")
            .with_query("query")
            .with_user("user")
            .build();

        assert!(client.is_err());

        assert_eq!(
            client.unwrap_err().to_string(),
            "Поле 'host' обязательно для заполнения!"
        );
    }

    #[test]
    fn not_successful_build_without_user() {
        let client = ClickhouseArgs::create()
            .with_host("localhost")
            .with_password("password")
            .with_port("port")
            .with_query("query")
            .build();

        assert!(client.is_err());
        assert_eq!(
            client.unwrap_err().to_string(),
            "Поле 'user' обязательно для заполнения!"
        );
    }

    #[test]
    fn not_successful_build_without_password() {
        let client = ClickhouseArgs::create()
            .with_host("localhost")
            .with_port("port")
            .with_query("query")
            .with_user("user")
            .build();

        assert!(client.is_err());
        assert_eq!(
            client.unwrap_err().to_string(),
            "Поле 'password' обязательно для заполнения!"
        );
    }

    #[test]
    fn not_successful_build_without_port() {
        let client = ClickhouseArgs::create()
            .with_host("localhost")
            .with_password("password")
            .with_query("query")
            .with_user("user")
            .build();

        assert!(client.is_err());
        assert_eq!(
            client.unwrap_err().to_string(),
            "Поле 'port' обязательно для заполнения!"
        );
    }

    #[test]
    fn not_successful_build_without_query() {
        let client = ClickhouseArgs::create()
            .with_host("localhost")
            .with_password("password")
            .with_port("port")
            .with_user("user")
            .build();

        assert!(client.is_err());
        assert_eq!(
            client.unwrap_err().to_string(),
            "Поле 'query' обязательно для заполнения!"
        );
    }
}
