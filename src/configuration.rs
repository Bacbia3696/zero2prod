use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};
#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: AppSettings,
}

#[derive(Deserialize, Debug)]
pub struct AppSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment = std::env::var("APP_ENVIRONMENT").unwrap_or("local".into());

    config::Config::builder()
        .add_source(config::File::with_name("configuration/base"))
        .add_source(config::File::with_name(&format!(
            "configuration/{environment}"
        )))
        .add_source(
            config::Environment::with_prefix("app")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?
        .try_deserialize()
}

impl DatabaseSettings {
    pub fn withdb(&self) -> PgConnectOptions {
        let DatabaseSettings { database_name, .. } = self;
        let mut options = self.without_db().database(database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let DatabaseSettings {
            username,
            password,
            port,
            host,
            require_ssl,
            ..
        } = self;
        let ssl_mode = if *require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .port(port.to_owned())
            .username(username)
            .host(host)
            .ssl_mode(ssl_mode)
            .password(password.expose_secret())
    }
}
