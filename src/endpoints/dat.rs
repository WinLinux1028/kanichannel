use std::sync::Arc;

use crate::{Error, Server};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    response::Response,
};
use chrono::TimeZone;
use sqlx::Row;

pub async fn get(state: State<Arc<Server>>, arg: Path<Argument>) -> Response {
    match get_(state, arg).await {
        Ok(o) => o,
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong.").into_response(),
    }
}

pub async fn get_(
    State(state): State<Arc<Server>>,
    Path(arg): Path<Argument>,
) -> Result<Response, Error> {
    let mut id_page = match arg.id.strip_suffix(".dat") {
        Some(s) => s.split('_'),
        None => return Err("".into()),
    };

    let threadid: i64 = match id_page.next() {
        Some(s) => s.parse()?,
        None => return Err("".into()),
    };
    let page: i64 = match id_page.next() {
        Some(s) => s.parse()?,
        None => 0,
    };

    if threadid < 0 || page < 0 {
        return Err("".into());
    }

    let thread_title = sqlx::query("SELECT title FROM threads WHERE threadid=$1;")
        .bind(threadid)
        .fetch_optional(&state.db)
        .await?;
    let thread_title: String = match thread_title {
        Some(s) => s.try_get(0)?,
        None => return Err("".into()),
    };

    let posts: Vec<Post> = sqlx::query_as(
        "SELECT name, mail, date, id, body
                FROM posts
                WHERE
                    threadid=$1
                ORDER BY date ASC
                LIMIT 1000
                OFFSET $2;",
    )
    .bind(threadid)
    .bind(page * 1000)
    .fetch_all(&state.db)
    .await?;

    let mut posts = posts.into_iter().map(|i| i.to_string());
    let mut result = Vec::new();

    let i = posts.next().ok_or("")?;
    let (i, _, _) = encoding_rs::SHIFT_JIS.encode(&i);
    result.append(&mut i.into_owned());
    let (i, _, _) = encoding_rs::SHIFT_JIS.encode(&thread_title);
    result.append(&mut i.into_owned());
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
    id: String,
}

#[derive(sqlx::FromRow)]
struct Post {
    name: String,
    mail: String,
    date: i64,
    id: String,
    body: String,
}

impl ToString for Post {
    fn to_string(&self) -> String {
        let date = chrono::Local.timestamp_opt(self.date >> 16, 0).unwrap();

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
            &self.id,
            &self.body
        )
    }
}
