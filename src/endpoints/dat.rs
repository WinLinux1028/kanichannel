use crate::{Error, Server};
use entity::*;

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    response::Response,
};
use chrono::TimeZone;
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect};

const MAX_POSTS: u64 = 1000;

pub async fn get(state: State<Arc<Server>>, arg: Path<Argument>) -> Response {
    match get_(state, arg).await {
        Ok(o) => o,
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn get_(
    State(state): State<Arc<Server>>,
    Path(arg): Path<Argument>,
) -> Result<Response, Error> {
    let thread: u64 = match arg.id.strip_suffix(".dat") {
        Some(s) => s.parse()?,
        None => return Err("".into()),
    };

    let mut page: u64;
    let board;
    match arg.board.rsplit_once('_') {
        Some((b, p)) => {
            board = b;
            page = p.parse()?;
        }
        None => {
            board = &arg.board;
            page = 0;
        }
    }

    let thread_title;
    let posts = if thread == 1000000001 {
        page = 0;
        thread_title = "Welcome to virtual board.".to_string();

        let body = format!(
            "There is nothing here.<br>You should go to https://{}/{}/",
            &state.config.domain, &arg.board
        );

        vec![Post {
            name: "</b>System<b>".to_string(),
            mail: "".to_string(),
            id: 65536000065536,
            poster_id: "System".to_string(),
            body,
        }]
    } else {
        let thread = thread::Entity::find()
            .filter(thread::Column::BoardId.eq(board))
            .filter(thread::Column::Id.eq(thread))
            .one(&state.db)
            .await?
            .ok_or("")?;
        thread_title = thread.name;

        let mut posts_ = post::Entity::find()
            .filter(post::Column::BoardId.eq(board))
            .filter(post::Column::ThreadId.eq(thread.id))
            .order_by_asc(post::Column::Id)
            .limit(MAX_POSTS)
            .offset(page * MAX_POSTS)
            .into_model::<Post>()
            .all(&state.db)
            .await?;

        if posts_.len() == usize::try_from(MAX_POSTS)? {
            let body = format!(
                "Next thread: https://{}/test/read.cgi/{}_{}/{}/",
                &state.config.domain,
                board,
                page + 1,
                thread.id
            );

            posts_.push(Post {
                name: "</b>Info<b>".to_string(),
                mail: "".to_string(),
                id: posts_.last().ok_or("")?.id,
                poster_id: "Info".to_string(),
                body,
            })
        }
        if page != 0 {
            let body = format!(
                "Previous thread: https://{}/test/read.cgi/{}_{}/{}/",
                &state.config.domain,
                board,
                page - 1,
                thread.id
            );

            posts_.insert(
                0,
                Post {
                    name: "</b>Info<b>".to_string(),
                    mail: "".to_string(),
                    id: thread.id << 16,
                    poster_id: "Info".to_string(),
                    body,
                },
            )
        }

        posts_
    };

    let mut result = Vec::new();
    let mut posts = posts.into_iter().map(|i| i.to_string());

    let i = posts.next().ok_or("")?;
    let (i, _, _) = encoding_rs::SHIFT_JIS.encode(&i);
    result.append(&mut i.into_owned());
    let (i, _, _) = encoding_rs::SHIFT_JIS.encode(&thread_title);
    result.append(&mut i.into_owned());
    if page != 0 {
        result.extend_from_slice(format!(" Part{}", page + 1).as_bytes());
    }
    result.push(0x0A);

    for i in posts {
        let (i, _, _) = encoding_rs::SHIFT_JIS.encode(&i);
        result.append(&mut i.into_owned());
        result.push(0x0A);
    }

    let mut response = result.into_response();
    let headers = response.headers_mut();
    headers.clear();
    headers.insert("Content-Type", "text/plain; charset=Shift_JIS".try_into()?);
    headers.insert("Cache-Control", "public, max-age=10".try_into()?);

    Ok(response)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Argument {
    board: String,
    id: String,
}

#[derive(FromQueryResult)]
struct Post {
    name: String,
    mail: String,
    id: i64,
    poster_id: String,
    body: String,
}

impl ToString for Post {
    fn to_string(&self) -> String {
        let date = chrono::Local.timestamp_opt(self.id >> 16, 0).unwrap();

        let weekday = match format!("{}", date.format("%w")).as_str() {
            "0" => "日",
            "1" => "月",
            "2" => "火",
            "3" => "水",
            "4" => "木",
            "5" => "金",
            "6" => "土",
            _ => "?",
        };

        format!(
            "{}<>{}<>{}({}) {}.00 ID:{}<>{}<>",
            &self.name,
            &self.mail,
            date.format("%Y/%m/%d"),
            weekday,
            date.format("%T"),
            &self.poster_id,
            &self.body
        )
    }
}
