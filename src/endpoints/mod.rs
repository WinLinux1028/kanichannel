mod bbs;
mod dat;
mod subject;

use crate::Router;

use axum::routing;

pub fn create_router() -> Router {
    Router::new()
        .route("/board/subject.txt", routing::get(subject::get))
        .route("/test/bbs.cgi", routing::post(bbs::post))
        .route("/board/dat/:id", routing::get(dat::get))
}
