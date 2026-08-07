use std::{env, net::SocketAddr, num::ParseIntError, str::FromStr};

use sqlx::mysql::MySqlConnectOptions;
use thiserror::Error;

use crate::service::DEFAULT_PHOTO_API_URL;

const DEFAULT_APP_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_DB_USER: &str = "root";
const DEFAULT_DB_PASS: &str = "pass";
const DEFAULT_DB_HOST: &str = "localhost";
const DEFAULT_DB_PORT: u16 = 3306;
const DEFAULT_DB_NAME: &str = "app";

#[derive(Clone)]
pub struct Config {
    pub app_addr: SocketAddr,
    pub photo_api_url: String,
    database_url: Option<String>,
    db_user: String,
    db_pass: String,
    db_host: String,
    db_port: u16,
    db_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let app_addr = normalize_app_addr(&env_or("APP_ADDR", DEFAULT_APP_ADDR))
            .parse()
            .map_err(ConfigError::InvalidAppAddr)?;
        let db_port = match env::var("DB_PORT") {
            Ok(value) => value.parse().map_err(ConfigError::InvalidDbPort)?,
            Err(_) => DEFAULT_DB_PORT,
        };

        Ok(Self {
            app_addr,
            photo_api_url: env_or("PHOTO_API_URL", DEFAULT_PHOTO_API_URL),
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
