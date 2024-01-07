use crate::STATE;

use entity::*;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, TransactionTrait};

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

    println!("You can specify a range like 100-200, 100-, -200 or a number like 100.");
    println!("If specifying multiple, separate them with commas.");
    print!("thread id> ");
    stdout.flush().unwrap();
    let mut thread_ids = String::new();
    stdin.read_line(&mut thread_ids).await.unwrap();

    let mut condition_post = Condition::any();
    let mut condition_thread = Condition::any();
    for thread_id in thread_ids.split(',') {
        let mut condition_post_ = Condition::all();
        let mut condition_thread_ = Condition::all();
        if let Some((id1, id2)) = thread_id.split_once('-') {
            let id1 = id1.trim();
            let id2 = id2.trim();
            if id1.is_empty() && id2.is_empty() {
                panic!("Invalid range");
            }
            if !id1.is_empty() {
                let id1 = id1.parse::<u64>().unwrap();
                condition_post_ = condition_post_.add(post::Column::ThreadId.gte(id1));
                condition_thread_ = condition_thread_.add(thread::Column::Id.gte(id1));
            }
            if !id2.is_empty() {
                let id2 = id2.parse::<u64>().unwrap();
                condition_post_ = condition_post_.add(post::Column::ThreadId.lte(id2));
                condition_thread_ = condition_thread_.add(thread::Column::Id.lte(id2));
            }
        } else {
            let thread_id = thread_id.trim().parse::<u64>().unwrap();
            condition_post_ = condition_post_.add(post::Column::ThreadId.eq(thread_id));
            condition_thread_ = condition_thread_.add(thread::Column::Id.eq(thread_id));
        }

        condition_post = condition_post.add(condition_post_);
        condition_thread = condition_thread.add(condition_thread_);
    }

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
        .filter(condition_post)
        .filter(post::Column::Id.ne(65536000000000_u64))
        .exec(&trx)
        .await
        .unwrap();
    thread::Entity::delete_many()
        .filter(thread::Column::BoardId.eq(board_id))
        .filter(condition_thread)
        .filter(thread::Column::Id.ne(1000000000_u64))
        .exec(&trx)
        .await
        .unwrap();

    trx.commit().await.unwrap();
    println!("OK");
}
