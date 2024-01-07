use crate::STATE;

use entity::*;
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter};

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

    loop {
        print!("What do you want to edit?> ");
        stdout.flush().unwrap();
        let mut buf = String::new();
        stdin.read_line(&mut buf).await.unwrap();
        let buf = buf.trim();

        match buf {
            "board name" => board_name(board_id).await,
            "category" => category(board_id).await,
            _ => help(),
        }
    }
}

async fn board_name(board_id: &str) {
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = std::io::stdout();

    print!("new name> ");
    stdout.flush().unwrap();
    let mut board_name = String::new();
    stdin.read_line(&mut board_name).await.unwrap();
    let board_name = board_name.trim();

    board::Entity::update_many()
        .col_expr(board::Column::Title, Expr::value(board_name))
        .filter(board::Column::Id.eq(board_id))
        .exec(&STATE.get().unwrap().db)
        .await
        .unwrap();

    println!("OK");
}

async fn category(board_id: &str) {
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = std::io::stdout();

    print!("new category> ");
    stdout.flush().unwrap();
    let mut category = String::new();
    stdin.read_line(&mut category).await.unwrap();
    let category = category.trim();

    board::Entity::update_many()
        .col_expr(board::Column::Category, Expr::value(category))
        .filter(board::Column::Id.eq(board_id))
        .exec(&STATE.get().unwrap().db)
        .await
        .unwrap();

    println!("OK");
}

fn help() {
    println!("board name, category");
    println!("Ctrl+C to exit.");
}
