mod connect;
mod init;

pub use connect::connect;

use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PgConfig {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    database: String,
    application_name: Option<String>,
    ssl_mode: Option<String>, // disable, allow, prefer, require, verify-ca, verify-full
    ssl_root_cert: Option<PathBuf>,
    ssl_client_cert: Option<PathBuf>,
    ssl_client_key: Option<PathBuf>,
    tune: Option<PgConfigTune>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PgConfigTune {
    statement_cache_capacity: Option<usize>,
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    max_lifetime: Option<u64>,    // seconds
    idle_timeout: Option<u64>,    // seconds
    acquire_timeout: Option<u64>, // seconds
}
