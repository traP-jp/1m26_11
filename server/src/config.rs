use std::{env, net::SocketAddr, num::ParseIntError, str::FromStr};

use sqlx::mysql::MySqlConnectOptions;
use thiserror::Error;

const DEFAULT_APP_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_DEMO_COOKIE_SECURE: bool = true;
const DEFAULT_DB_USER: &str = "root";
const DEFAULT_DB_PASS: &str = "pass";
const DEFAULT_DB_HOST: &str = "localhost";
const DEFAULT_DB_PORT: u16 = 3306;
const DEFAULT_DB_NAME: &str = "app";

#[derive(Clone)]
pub struct Config {
    pub app_addr: SocketAddr,
    pub auth_mode: AuthMode,
    pub demo_cookie_secure: bool,
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
        let demo_cookie_secure = env_bool("DEMO_COOKIE_SECURE", DEFAULT_DEMO_COOKIE_SECURE)?;
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
            demo_cookie_secure,
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
    #[error("{key} must be `true` or `false`, got `{value}`")]
    InvalidBooleanEnvironmentValue { key: &'static str, value: String },
    #[error("DB_PORT is not a valid port number")]
    InvalidDbPort(#[source] ParseIntError),
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn env_bool(key: &'static str, default: bool) -> Result<bool, ConfigError> {
    match env::var(key) {
        Ok(value) => {
            parse_bool(&value).ok_or(ConfigError::InvalidBooleanEnvironmentValue { key, value })
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidBooleanEnvironmentValue {
            key,
            value: "<non-Unicode value>".to_owned(),
        }),
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
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

    use super::{AuthMode, parse_bool};

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

    #[test]
    fn parses_boolean_environment_values() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("TRUE"), None);
        assert_eq!(parse_bool("1"), None);
        assert_eq!(parse_bool(""), None);
    }
}
