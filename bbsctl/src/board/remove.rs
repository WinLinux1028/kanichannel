use crate::STATE;

use entity::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};

use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn run() {
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = std::io::stdout();

    print!("board id> ");
    stdout.flush().unwrap();
    let mut board_id = String::new();
    stdin.read_line(&mut board_id).await.unwrap();
    let board_id = board_id.trim();

    print!("ARE YOU REALLY? [y/N]> ");
    stdout.flush().unwrap();
    let mut confirm = String::new();
    stdin.read_line(&mut confirm).await.unwrap();
    let confirm = confirm.trim();

    if confirm != "y" {
        println!("Aborted");
        return;
    }

    let trx = STATE.get().unwrap().db.begin().await.unwrap();

    post::Entity::delete_many()
        .filter(post::Column::BoardId.eq(board_id))
        .exec(&trx)
        .await
        .unwrap();
    thread::Entity::delete_many()
        .filter(thread::Column::BoardId.eq(board_id))
        .exec(&trx)
        .await
        .unwrap();
    board::Entity::delete_many()
        .filter(board::Column::Id.eq(board_id))
        .exec(&trx)
        .await
        .unwrap();

    trx.commit().await.unwrap();
    println!("OK");
}
