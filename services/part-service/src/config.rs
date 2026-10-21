use serde::Deserialize;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            server_address: env::var("PART_SERVICE_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:3002".to_string()),
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
        })
    }
}
