use super::PgConfig;
use crate::Error;

use std::{str::FromStr, time::Duration};

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};

pub async fn connect(config: &PgConfig) -> Result<PgPool, Error> {
    let mut connect_option = PgConnectOptions::new();
    let mut pool_option = PgPoolOptions::new();

    connect_option = connect_option
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .database(&config.database);

    if let Some(password) = &config.password {
        connect_option = connect_option.password(password);
    }
    if let Some(application_name) = &config.application_name {
        connect_option = connect_option.application_name(application_name);
    }
    if let Some(ssl_mode) = &config.ssl_mode {
        connect_option = connect_option.ssl_mode(PgSslMode::from_str(ssl_mode)?);
    }
    if let Some(ssl_root_cert) = &config.ssl_root_cert {
        connect_option = connect_option.ssl_root_cert(ssl_root_cert);
    }
    if let Some(ssl_client_cert) = &config.ssl_client_cert {
        connect_option = connect_option.ssl_client_cert(ssl_client_cert);
    }
    if let Some(ssl_client_key) = &config.ssl_client_key {
        connect_option = connect_option.ssl_client_key(ssl_client_key);
    }

    if let Some(tune) = &config.tune {
        if let Some(statement_cache_capacity) = tune.statement_cache_capacity {
            connect_option = connect_option.statement_cache_capacity(statement_cache_capacity);
        }

        if let Some(max_connections) = tune.max_connections {
            pool_option = pool_option.max_connections(max_connections);
        }
        if let Some(min_connections) = tune.min_connections {
            pool_option = pool_option.min_connections(min_connections);
        }
        if let Some(max_lifetime) = tune.max_lifetime {
            pool_option = pool_option.max_lifetime(Duration::from_secs(max_lifetime));
        }
        if let Some(idle_timeout) = tune.idle_timeout {
            pool_option = pool_option.max_lifetime(Duration::from_secs(idle_timeout));
        }
        if let Some(acquire_timeout) = tune.acquire_timeout {
            pool_option = pool_option.acquire_timeout(Duration::from_secs(acquire_timeout));
        }
    }

    let mut connection;
    loop {
        match pool_option
            .clone()
            .connect_with(connect_option.clone())
            .await
        {
            Ok(o) => {
                connection = o;
            }
            Err(e) => {
                eprintln!("DB connection: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        match super::init::init(&connection).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("DB initialization: {}", e);
                continue;
            }
        }
    }

    Ok(connection)
}
