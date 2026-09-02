use std::{env, net::SocketAddr, num::ParseIntError, str::FromStr};

use sqlx::mysql::MySqlConnectOptions;
use thiserror::Error;

const DEFAULT_APP_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_DEMO_COOKIE_SECURE: bool = true;
const DEFAULT_DB_USER: &str = "root";
const DEFAULT_DB_PASS: &str = "pass";
const DEFAULT_DB_HOST: &str = "localhost";
const DEFAULT_DB_PORT: u16 = 3306;
const DEFAULT_DB_NAME: &str = "app";
const DEFAULT_IMAGE_UPLOAD_ENABLED: bool = false;

#[derive(Clone)]
pub struct StorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub force_path_style: bool,
    pub public_base_url: String,
}

#[derive(Clone)]
pub struct Config {
    pub app_addr: SocketAddr,
    pub auth_mode: AuthMode,
    pub demo_cookie_secure: bool,
    pub image_upload_enabled: bool,
    pub storage: Option<StorageConfig>,
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
        let image_upload_enabled = env_bool("IMAGE_UPLOAD_ENABLED", DEFAULT_IMAGE_UPLOAD_ENABLED)?;

        let storage = load_storage_config(auth_mode, image_upload_enabled, required_env)?;
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
            image_upload_enabled,
            storage,
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
    #[error("IMAGE_UPLOAD_ENABLED=true requires AUTH_MODE=demo")]
    ImageUploadRequiresDemoAuth,
    #[error("{key} must be set when IMAGE_UPLOAD_ENABLED=true")]
    MissingImageUploadSetting { key: &'static str },
    #[error("APP_ADDR is not a valid socket address")]
    InvalidAppAddr(#[source] std::net::AddrParseError),
    #[error("{key} must be `true` or `false`, got `{value}`")]
    InvalidBooleanEnvironmentValue { key: &'static str, value: String },
    #[error("DB_PORT is not a valid port number")]
    InvalidDbPort(#[source] ParseIntError),
}

fn load_storage_config<F>(
    auth_mode: AuthMode,
    image_upload_enabled: bool,
    lookup: F,
) -> Result<Option<StorageConfig>, ConfigError>
where
    F: Fn(&'static str) -> Result<String, ConfigError>,
{
    if !image_upload_enabled {
        return Ok(None);
    }

    if auth_mode != AuthMode::Demo {
        return Err(ConfigError::ImageUploadRequiresDemoAuth);
    }

    let force_path_style_value = lookup("S3_FORCE_PATH_STYLE")?;
    let force_path_style =
        parse_bool(&force_path_style_value).ok_or(ConfigError::InvalidBooleanEnvironmentValue {
            key: "S3_FORCE_PATH_STYLE",
            value: force_path_style_value,
        })?;

    Ok(Some(StorageConfig {
        endpoint: lookup("S3_ENDPOINT")?,
        bucket: lookup("S3_BUCKET")?,
        access_key_id: lookup("AWS_ACCESS_KEY_ID")?,
        secret_access_key: lookup("AWS_SECRET_ACCESS_KEY")?,
        region: lookup("AWS_REGION")?,
        force_path_style,
        public_base_url: lookup("ASSET_PUBLIC_BASE_URL")?,
    }))
}

fn required_env(key: &'static str) -> Result<String, ConfigError> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        Ok(_) | Err(env::VarError::NotPresent) | Err(env::VarError::NotUnicode(_)) => {
            Err(ConfigError::MissingImageUploadSetting { key })
        }
    }
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
        format!("127.0.0.1{value}")
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

    use super::{
        AuthMode, ConfigError, DEFAULT_APP_ADDR, load_storage_config, normalize_app_addr,
        parse_bool,
    };

    fn storage_value(key: &'static str) -> Result<String, ConfigError> {
        let value = match key {
            "S3_ENDPOINT" => "https://s3.example.invalid",
            "S3_BUCKET" => "test-bucket",
            "AWS_ACCESS_KEY_ID" => "test-access-key",
            "AWS_SECRET_ACCESS_KEY" => "test-secret-key",
            "AWS_REGION" => "test-region",
            "S3_FORCE_PATH_STYLE" => "true",
            "ASSET_PUBLIC_BASE_URL" => "https://assets.example.invalid/",
            _ => return Err(ConfigError::MissingImageUploadSetting { key }),
        };

        Ok(value.to_owned())
    }

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

    #[test]
    fn disabled_image_upload_does_not_require_storage_settings() {
        let storage = load_storage_config(AuthMode::Demo, false, |_| {
            panic!("storage settings must not be read when upload is disabled")
        })
        .expect("disabled upload should be valid");

        assert!(storage.is_none());
    }

    #[test]
    fn demo_enabled_image_upload_loads_storage_settings() {
        let storage = load_storage_config(AuthMode::Demo, true, storage_value)
            .expect("demo upload configuration should be valid")
            .expect("enabled upload should contain storage settings");

        assert_eq!(storage.endpoint, "https://s3.example.invalid");
        assert_eq!(storage.bucket, "test-bucket");
        assert_eq!(storage.region, "test-region");
        assert!(storage.force_path_style);
        assert_eq!(storage.public_base_url, "https://assets.example.invalid/");
    }

    #[test]
    fn neoshowcase_enabled_image_upload_is_rejected() {
        let result = load_storage_config(AuthMode::NeoShowcase, true, storage_value);

        assert!(matches!(
            result,
            Err(ConfigError::ImageUploadRequiresDemoAuth)
        ));
    }

    #[test]
    fn enabled_image_upload_rejects_missing_storage_setting() {
        let result = load_storage_config(AuthMode::Demo, true, |key| {
            if key == "AWS_SECRET_ACCESS_KEY" {
                Err(ConfigError::MissingImageUploadSetting { key })
            } else {
                storage_value(key)
            }
        });

        assert!(matches!(
            result,
            Err(ConfigError::MissingImageUploadSetting {
                key: "AWS_SECRET_ACCESS_KEY"
            })
        ));
    }

    #[test]
    fn app_addr_defaults_and_shorthand_use_loopback() {
        assert_eq!(DEFAULT_APP_ADDR, "127.0.0.1:8080");
        assert_eq!(normalize_app_addr(":8080"), "127.0.0.1:8080");
        assert_eq!(normalize_app_addr("0.0.0.0:8080"), "0.0.0.0:8080");
    }
}
