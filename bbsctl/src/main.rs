mod board;
mod post;
mod thread;

use once_cell::sync::OnceCell;
use sea_orm::{Database, DatabaseConnection};
use tokio::{fs::File, io::AsyncReadExt};

static STATE: OnceCell<State> = OnceCell::new();

#[tokio::main]
async fn main() {
    let mut config = String::new();
    File::open("./config.json5")
        .await
        .unwrap()
        .read_to_string(&mut config)
        .await
        .unwrap();
    let config: Config = json5::from_str(&config).unwrap();
    let db = Database::connect(&config.db.connect).await.unwrap();
    let _ = STATE.set(State { db });

    let mut args = std::env::args();
    args.next();
    match args.next().as_deref() {
        Some("board") => board::run(args).await,
        Some("thread") => thread::run(args).await,
        Some("post") => post::run(args).await,
        _ => help(),
    }
}

fn help() {
    println!("bbsctl [sub command] [options...]");
    println!("sub commands: board, thread, post, help");
}

struct State {
    db: DatabaseConnection,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    db: DbConfig,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DbConfig {
    connect: String,
}
