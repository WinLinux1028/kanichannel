#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod endpoints;
mod postgres;

use crate::postgres::PgConfig;

use rand::Rng;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpListener,
};

use sqlx::PgPool;

type Error = Box<dyn std::error::Error>;
type Router = axum::Router<Arc<Server>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut config = String::new();
    File::open("./config.json")
        .await?
        .read_to_string(&mut config)
        .await?;

    let mut salt_f = File::open("./salt.bin").await;
    if salt_f.is_err() {
        let mut new_salt = BufWriter::new(File::create("./salt.bin").await?);
        for _ in 0..28 {
            new_salt.write_all(&[rand::thread_rng().gen()]).await?;
        }
        new_salt.flush().await?;
        drop(new_salt);

        salt_f = File::open("./salt.bin").await;
    }
    let mut salt = Vec::new();
    salt_f?.read_to_end(&mut salt).await?;

    let config: Config = serde_json::from_str(&config)?;
    let db = postgres::connect(&config.postgres).await?;

    let state = Arc::new(Server { config, db, salt });

    let app = endpoints::create_router().with_state(Arc::clone(&state));

    let listener = TcpListener::bind(&state.config.listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

struct Server {
    config: Config,
    db: PgPool,
    salt: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    listen: SocketAddr,
    postgres: PgConfig,
}
