use crate::{Error, Server};
use entity::*;

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    response::Response,
};
use encoding_rs::SHIFT_JIS;
use sea_orm::{
    sea_query::Expr, ColumnTrait, EntityTrait, FromQueryResult, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};

pub async fn get(
    state: State<Arc<Server>>,
    parg: Path<PathArgument>,
    qarg: Query<QueryArgument>,
) -> Response {
    match get_(state, parg, qarg).await {
        Ok(o) => o,
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

async fn get_(
    State(state): State<Arc<Server>>,
    Path(parg): Path<PathArgument>,
    Query(qarg): Query<QueryArgument>,
) -> Result<Response, Error> {
    let threads = if let Some((_, page)) = parg.board.rsplit_once('_') {
        let _: u64 = page.parse()?;
        vec![Subject {
            id: 1000000001,
            name: "Welcome to the virtual board.".to_string(),
            post_count: 1,
        }]
    } else {
        let page = qarg.page.unwrap_or(0);

        thread::Entity::find()
            .select_only()
            .column(thread::Column::Id)
            .column(thread::Column::Name)
            .column_as(post::Column::Id.count(), "post_count")
            .column_as(post::Column::Id.max(), "id_max")
            .join(JoinType::InnerJoin, thread::Relation::Post.def())
            .filter(thread::Column::BoardId.eq(&parg.board))
            .filter(post::Column::Mail.ne("sage"))
            .group_by(thread::Column::Id)
            .group_by(thread::Column::Name)
            .order_by_desc(Expr::cust("id_max"))
            .limit(1000)
            .offset(page * 1000)
            .into_model::<Subject>()
            .all(&state.db)
            .await?
    };

    if threads.is_empty() {
        return Err("".into());
    }

    let mut result = Vec::new();
    for i in threads.into_iter().map(|i| i.to_string()) {
        result.append(&mut SHIFT_JIS.encode(&i).0.into_owned());
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
pub struct QueryArgument {
    page: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PathArgument {
    board: String,
}

#[derive(FromQueryResult)]
struct Subject {
    id: i64,
    name: String,
    post_count: i64,
}

impl ToString for Subject {
    fn to_string(&self) -> String {
        format!("{}.dat<>{} ({})", self.id, &self.name, self.post_count)
    }
}
