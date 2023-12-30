use crate::{Error, Server};

use std::sync::Arc;

use axum::{
    body::HttpBody,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    response::Response,
    Form,
};
use base_62::base62;
use encoding_rs::SHIFT_JIS;
use rand::Rng;
use sha3::{Digest, Sha3_224};

pub async fn post(
    state: State<Arc<Server>>,
    header: HeaderMap,
    arg: Form<RawArgument>,
) -> Response {
    match post_(state, header, arg).await {
        Ok(o) => o,
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong.").into_response(),
    }
}

async fn post_(
    State(state): State<Arc<Server>>,
    header: HeaderMap,
    Form(arg): Form<RawArgument>,
) -> Result<Response, Error> {
    let mut arg = Argument::from(arg);

    if arg.message.trim().is_empty() {
        return Err("".into());
    }

    if arg.bbs != "board" {
        return Err("".into());
    }

    let mut psql = state.db.begin().await?;
    let timestamp = chrono::Local::now().timestamp();

    if arg.submit == "新規スレッド作成" {
        let subject = match &arg.subject {
            Some(s) => s,
            None => return Err("".into()),
        };

        if subject.trim().is_empty() {
            return Err("".into());
        }

        sqlx::query("INSERT INTO threads(threadid, title, lastupdate) VALUES ($1, $2, $3)")
            .bind(timestamp)
            .bind(subject)
            .bind(timestamp)
            .execute(&mut *psql)
            .await?;
        arg.key = Some(timestamp.to_string());

        do_post(&mut psql, &state.salt, header, &arg, timestamp).await?;
    } else if arg.submit == "書き込む" {
        do_post(&mut psql, &state.salt, header, &arg, timestamp).await?;
    } else {
        return Err("".into());
    }

    psql.commit().await?;

    let mut response = HTML_OK.as_slice().into_response();
    let headers = response.headers_mut();
    headers.clear();
    headers.insert("Content-Type", "text/html; charset=Shift_JIS".try_into()?);
    headers.insert("Cache-Control", "no-store".try_into()?);

    Ok(response)
}

async fn do_post(
    psql: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    salt: &[u8],
    header: HeaderMap,
    arg: &Argument,
    timestamp: i64,
) -> Result<(), Error> {
    let threadid: i64 = match arg.key.as_ref().map(|k| k.split('_').next()) {
        Some(Some(s)) => s.parse()?,
        _ => return Err("".into()),
    };
    let name = sanitize(&arg.from);
    let mail = sanitize(&arg.mail);
    let date = ((timestamp << 16) | (rand::thread_rng().gen::<u16>() as i64)).abs();
    let body = sanitize(&arg.message);

    let ip = header.get("X-Real-IP").ok_or("")?;
    let mut hasher = Sha3_224::new();
    hasher.update(ip);
    hasher.update(salt);
    let mut hashed_ip = hasher.finalize();
    for _ in 0..5000 {
        let mut hasher = Sha3_224::new();
        hasher.update(hashed_ip);
        hasher.update(salt);
        hashed_ip = hasher.finalize();
    }
    let id = base62::encode(&hashed_ip);
    let mut id = id.as_str();
    if id.len() > 10 {
        id = &id[0..10];
    }

    sqlx::query(
        "INSERT INTO posts(threadid, name, mail, date, id, body)
                VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(threadid)
    .bind(name)
    .bind(mail)
    .bind(date)
    .bind(id)
    .bind(body)
    .execute(&mut **psql)
    .await?;

    if arg.mail != "sage" {
        sqlx::query("UPDATE threads SET lastupdate=$1 WHERE threadid=$2")
            .bind(timestamp)
            .bind(threadid)
            .execute(&mut **psql)
            .await?;
    }

    Ok(())
}

fn sanitize(s: &str) -> String {
    let mut result = String::new();

    for i in s.chars() {
        if i == '<' {
            result.push_str("&lt;");
        } else if i == '>' {
            result.push_str("&gt;");
        } else if i == '"' {
            result.push_str("&quot;");
        } else if i == '\'' {
            result.push_str("&#039;");
        } else if i == '\n' {
            result.push_str("<br>");
        } else {
            result.push(i);
        }
    }

    result
}

pub struct Argument {
    submit: String,
    bbs: String,
    key: Option<String>,
    message: String,
    from: String,
    mail: String,
    subject: Option<String>,
}

#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RawArgument {
    submit: Vec<u8>,
    bbs: Vec<u8>,
    key: Option<Vec<u8>>,
    MESSAGE: Vec<u8>,
    FROM: Vec<u8>,
    mail: Vec<u8>,
    subject: Option<Vec<u8>>,
}

impl From<RawArgument> for Argument {
    fn from(mut value: RawArgument) -> Self {
        let mut result = Argument {
            submit: SHIFT_JIS.decode(&value.submit).0.into_owned(),
            bbs: SHIFT_JIS.decode(&value.bbs).0.into_owned(),
            key: value.key.map(|k| SHIFT_JIS.decode(&k).0.into_owned()),
            message: SHIFT_JIS.decode(&value.MESSAGE).0.into_owned(),
            from: SHIFT_JIS.decode(&value.FROM).0.into_owned(),
            mail: SHIFT_JIS.decode(&value.mail).0.into_owned(),
            subject: value.subject.map(|s| SHIFT_JIS.decode(&s).0.into_owned()),
        };

        if result.from.is_empty() {
            result.from = "名無しさん".to_string();
        }

        result
    }
}

const HTML_OK: [u8; 154] = [
    60, 104, 116, 109, 108, 32, 108, 97, 110, 103, 61, 34, 106, 97, 34, 62, 60, 104, 101, 97, 100,
    62, 60, 116, 105, 116, 108, 101, 62, 143, 145, 130, 171, 130, 177, 130, 221, 130, 220, 130,
    181, 130, 189, 129, 66, 60, 47, 116, 105, 116, 108, 101, 62, 60, 47, 104, 101, 97, 100, 62, 60,
    98, 111, 100, 121, 62, 143, 145, 130, 171, 130, 177, 130, 221, 130, 170, 143, 73, 130, 237,
    130, 232, 130, 220, 130, 181, 130, 189, 129, 66, 60, 98, 114, 62, 60, 98, 114, 62, 137, 230,
    150, 202, 130, 240, 144, 216, 130, 232, 145, 214, 130, 166, 130, 233, 130, 220, 130, 197, 130,
    181, 130, 206, 130, 231, 130, 173, 130, 168, 145, 210, 130, 191, 137, 186, 130, 179, 130, 162,
    129, 66, 60, 47, 98, 111, 100, 121, 62, 60, 47, 104, 116, 109, 108, 62,
];
