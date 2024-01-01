use std::sync::Arc;

use crate::{Error, Server};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    response::Response,
};

pub async fn get(state: State<Arc<Server>>, arg: Query<Argument>) -> Response {
    match get_(state, arg).await {
        Ok(o) => o,
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

async fn get_(
    State(state): State<Arc<Server>>,
    Query(arg): Query<Argument>,
) -> Result<Response, Error> {
    let offset = match arg.page.map(|p| (p, p >= 0)) {
        Some((s, true)) => s * 1000,
        Some((_, false)) => return Err("".into()),
        None => 0,
    };

    let threads: Vec<Subject> = sqlx::query_as(
        "SELECT threadid, title, posts
                FROM threads
                    NATURAL INNER JOIN 
                        (SELECT threadid, COUNT(threadid) AS posts
                            FROM posts
                            GROUP BY threadid)
                ORDER BY lastupdate DESC
                LIMIT 1000
                OFFSET $1;",
    )
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::new();
    for i in threads.into_iter().map(|i| i.to_string()) {
        let (i, _, _) = encoding_rs::SHIFT_JIS.encode(&i);
        result.append(&mut i.into_owned());
        result.push(0x0A);
    }

    let mut response = result.into_response();
    let headers = response.headers_mut();
    headers.clear();
    headers.insert("Content-Type", "text/plain; charset=Shift_JIS".try_into()?);
    headers.insert("Cache-Control", "public, max-age=30".try_into()?);

    Ok(response)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Argument {
    page: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct Subject {
    threadid: i64,
    title: String,
    posts: i64,
}

impl ToString for Subject {
    fn to_string(&self) -> String {
        format!("{}.dat<>{} ({})", self.threadid, &self.title, self.posts)
    }
}
