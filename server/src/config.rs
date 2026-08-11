use std::{env, net::SocketAddr, num::ParseIntError, str::FromStr};

use sqlx::mysql::MySqlConnectOptions;
use thiserror::Error;

const DEFAULT_APP_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_DB_USER: &str = "root";
const DEFAULT_DB_PASS: &str = "pass";
const DEFAULT_DB_HOST: &str = "localhost";
const DEFAULT_DB_PORT: u16 = 3306;
const DEFAULT_DB_NAME: &str = "app";

#[derive(Clone)]
pub struct Config {
    pub app_addr: SocketAddr,
    pub auth_mode: AuthMode,
    database_url: Option<String>,
    db_user: String,
    db_pass: String,
    db_host: String,
    db_port: u16,
    db_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let auth_mode = match env::var("AUTH_MODE") {
            Ok(value) => value.parse().map_err(ConfigError::InvalidAuthMode)?,
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::MissingAuthMode);
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidAuthMode(
                    "<non-Unicode value>".to_owned(),
                ));
            }
        };
        let app_addr = normalize_app_addr(&env_or("APP_ADDR", DEFAULT_APP_ADDR))
            .parse()
            .map_err(ConfigError::InvalidAppAddr)?;
        let db_port = match env::var("DB_PORT") {
            Ok(value) => value.parse().map_err(ConfigError::InvalidDbPort)?,
            Err(_) => DEFAULT_DB_PORT,
        };

        Ok(Self {
            app_addr,
            auth_mode,
            database_url: env::var("DATABASE_URL")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            db_user: env_or("DB_USER", DEFAULT_DB_USER),
            db_pass: env_or("DB_PASS", DEFAULT_DB_PASS),
            db_host: env_or("DB_HOST", DEFAULT_DB_HOST),
            db_port,
            db_name: env_or("DB_NAME", DEFAULT_DB_NAME),
        })
    }

    pub fn database_options(&self) -> Result<MySqlConnectOptions, sqlx::Error> {
        if let Some(database_url) = &self.database_url {
            return MySqlConnectOptions::from_str(database_url);
        }

        Ok(MySqlConnectOptions::new()
            .host(&self.db_host)
            .port(self.db_port)
            .username(&self.db_user)
            .password(&self.db_pass)
            .database(&self.db_name))
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("AUTH_MODE is not set")]
    MissingAuthMode,
    #[error("AUTH_MODE must be `demo` or `neoshowcase`, got `{0}`")]
    InvalidAuthMode(String),
    #[error("APP_ADDR is not a valid socket address")]
    InvalidAppAddr(#[source] std::net::AddrParseError),
    #[error("DB_PORT is not a valid port number")]
    InvalidDbPort(#[source] ParseIntError),
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn normalize_app_addr(value: &str) -> String {
    if value.starts_with(':') {
        format!("0.0.0.0{value}")
    } else {
        value.to_owned()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthMode {
    Demo,
    NeoShowcase,
}

impl FromStr for AuthMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "demo" => Ok(Self::Demo),
            "neoshowcase" => Ok(Self::NeoShowcase),
            other => Err(other.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::AuthMode;

    #[test]
    fn parses_auth_mode() {
        assert_eq!(AuthMode::from_str("demo"), Ok(AuthMode::Demo));
        assert_eq!(AuthMode::from_str("neoshowcase"), Ok(AuthMode::NeoShowcase));
    }

    #[test]
    fn rejects_invalid_auth_mode() {
        assert_eq!(AuthMode::from_str("local"), Err("local".to_owned()));
        assert!(AuthMode::from_str("Demo").is_err());
        assert!(AuthMode::from_str("").is_err());
    }
}
