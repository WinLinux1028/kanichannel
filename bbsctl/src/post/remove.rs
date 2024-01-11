use crate::STATE;

use entity::*;
use sea_orm::{
    sea_query::{Expr, Query},
    ColumnTrait, Condition, EntityTrait, QueryFilter,
};

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

    print!("thread id> ");
    stdout.flush().unwrap();
    let mut thread_id = String::new();
    stdin.read_line(&mut thread_id).await.unwrap();
    let thread_id: u64 = thread_id.trim().parse().unwrap();

    println!("You can specify a range like 100-200, 100-, -200 or a number like 100.");
    println!("If specifying multiple, separate them with commas.");
    print!("post time(unix time)> ");
    stdout.flush().unwrap();
    let mut post_times = String::new();
    stdin.read_line(&mut post_times).await.unwrap();

    let mut condition = Condition::any();
    for post_time in post_times.split(',') {
        let mut condition_ = Condition::all();
        if let Some((time1, time2)) = post_time.split_once('-') {
            let time1 = time1.trim();
            let time2 = time2.trim();
            if time1.is_empty() && time2.is_empty() {
                panic!("Invalid range");
            }
            if !time1.is_empty() {
                let id1 = time1.parse::<u64>().unwrap() << 16;
                condition_ = condition_.add(post::Column::Id.gte(id1));
            }
            if !time2.is_empty() {
                let id2 = time2.parse::<u64>().unwrap() << 16;
                condition_ = condition_.add(post::Column::Id.lte(id2));
            }
        } else {
            let post_id1 = post_time.trim().parse::<u64>().unwrap() << 16;
            let post_id2 = post_id1 | 0xFFFF;
            condition_ = condition_.add(post::Column::Id.gte(post_id1));
            condition_ = condition_.add(post::Column::Id.lte(post_id2));
        }

        condition = condition.add(condition_);
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

    post::Entity::update_many()
        .col_expr(post::Column::Name, Expr::value("</b>System<b>"))
        .col_expr(post::Column::Mail, Expr::value("sage"))
        .col_expr(post::Column::Body, Expr::value("Deleted by administrator"))
        .filter(post::Column::BoardId.eq(board_id))
        .filter(post::Column::ThreadId.eq(thread_id))
        .filter(condition)
        .filter(
            post::Column::Id.not_in_subquery(
                Query::select()
                    .expr(post::Column::Id.min())
                    .from(post::Entity)
                    .and_where(post::Column::BoardId.eq(board_id))
                    .and_where(post::Column::ThreadId.eq(thread_id))
                    .to_owned(),
            ),
        )
        .exec(&STATE.get().unwrap().db)
        .await
        .unwrap();

    println!("OK");
}
