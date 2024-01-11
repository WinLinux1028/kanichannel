use crate::STATE;

use entity::*;
use sea_orm::{ActiveModelTrait, IntoActiveModel, TransactionTrait};

use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn run() {
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = std::io::stdout();

    println!("Note: ASCII only, underscore is not usable");
    print!("board id> ");
    stdout.flush().unwrap();
    let mut board_id = String::new();
    stdin.read_line(&mut board_id).await.unwrap();
    let board_id = board_id.trim();
    if board_id.contains('_') || !board_id.is_ascii() {
        panic!("The input does not meet the conditions.");
    }
    if board_id.is_empty() {
        panic!("Empty board id is not allowed");
    }

    print!("board name> ");
    stdout.flush().unwrap();
    let mut board_name = String::new();
    stdin.read_line(&mut board_name).await.unwrap();
    let board_name = board_name.trim();
    if board_name.is_empty() {
        panic!("Empty board name is not allowed");
    }

    print!("category> ");
    stdout.flush().unwrap();
    let mut category = String::new();
    stdin.read_line(&mut category).await.unwrap();
    let category = category.trim();
    if category.is_empty() {
        panic!("Empty category is not allowed");
    }

    let trx = STATE.get().unwrap().db.begin().await.unwrap();

    board::Model {
        id: board_id.to_string(),
        title: board_name.to_string(),
        category: category.to_string(),
    }
    .into_active_model()
    .insert(&trx)
    .await
    .unwrap();

    thread::Model {
        board_id: board_id.to_string(),
        id: 1000000000,
        name: "Hello World!".to_string(),
    }
    .into_active_model()
    .insert(&trx)
    .await
    .unwrap();

    post::Model {
        board_id: board_id.to_string(),
        thread_id: 1000000000,
        id: 65536000000000,
        name: "</b>System<b>".to_string(),
        mail: "".to_string(),
        poster_id: "System".to_string(),
        body: "Congratulations!<br>New board has been created.".to_string(),
    }
    .into_active_model()
    .insert(&trx)
    .await
    .unwrap();

    trx.commit().await.unwrap();
    println!("OK");
}
