use crate::{utils, Error, Server};
use entity::*;

use std::sync::Arc;

use axum::{
    extract::{RawForm, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    response::Response,
};
use base_62::base62;
use rand::Rng;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, IntoActiveModel, TransactionTrait};
use sha3::{Digest, Sha3_224};

pub async fn post(state: State<Arc<Server>>, header: HeaderMap, form: RawForm) -> Response {
    match post_(state, header, form).await {
        Ok(o) => o,
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

async fn post_(
    State(state): State<Arc<Server>>,
    header: HeaderMap,
    RawForm(form): RawForm,
) -> Result<Response, Error> {
    let referer = header.get("referer").ok_or("")?.to_str()?;
    if referer.starts_with(&format!("https://{}/", &state.config.domain)) {
        return Err("".into());
    }

    let arg: RawArgument = utils::decord_form(form.into_iter().collect(), encoding_rs::SHIFT_JIS)?;
    let mut arg = Argument::from(arg);

    if arg.message.is_empty() {
        return Err("".into());
    }
    if arg.key.as_deref() == Some("1000000001") {
        return Err("".into());
    }

    let mut page = None;
    if let Some((bbs, page_)) = arg.bbs.rsplit_once('_') {
        page = Some(page_.to_string());
        arg.bbs = bbs.to_string();
    }

    let trx = state.db.begin().await?;
    let timestamp = chrono::Local::now().timestamp();

    if arg.submit == "新規スレッド作成" {
        if page.is_some() {
            return Err("".into());
        }
        let subject = arg.subject.as_ref().ok_or("")?;
        if subject.is_empty() {
            return Err("".into());
        }

        thread::Model {
            board_id: arg.bbs.clone(),
            id: timestamp,
            name: subject.clone(),
        }
        .into_active_model()
        .insert(&trx)
        .await?;

        if arg.mail == "sage" {
            arg.mail = "".to_string();
        }
        arg.key = Some(timestamp.to_string());
    } else if arg.submit == "書き込む" {
    } else {
        return Err("".into());
    }

    do_post(&trx, &state.salt, header, &arg, timestamp).await?;
    trx.commit().await?;

    let mut response = HTML_OK.as_slice().into_response();
    let headers = response.headers_mut();
    headers.clear();
    headers.insert("Content-Type", "text/html; charset=Shift_JIS".try_into()?);
    headers.insert("Cache-Control", "no-store".try_into()?);

    Ok(response)
}

async fn do_post(
    trx: &DatabaseTransaction,
    salt: &[u8],
    header: HeaderMap,
    arg: &Argument,
    timestamp: i64,
) -> Result<(), Error> {
    let ip = header.get("X-Real-IP").ok_or("")?;
    let mut hasher = Sha3_224::new();
    hasher.update(ip);
    hasher.update(salt);
    hasher.update((timestamp / 604800).to_le_bytes());
    let mut hashed_ip = hasher.finalize();
    for _ in 0..5000 {
        let mut hasher = Sha3_224::new();
        hasher.update(hashed_ip);
        hasher.update(salt);
        hashed_ip = hasher.finalize();
    }
    let poster_id = base62::encode(&hashed_ip);
    let mut poster_id = poster_id.as_str();
    if poster_id.len() > 10 {
        poster_id = &poster_id[0..10];
    }

    let id = ((timestamp << 16) | (rand::thread_rng().gen::<u16>() as i64)).abs();

    let name = if arg.from.is_empty() {
        "</b>Anonymous".to_string()
    } else if arg.from == "fusianasan" {
        format!("</b>{}", header.get("X-Real-IP").ok_or("")?.to_str()?)
    } else {
        sanitize(&arg.from)
    };

    post::Model {
        board_id: arg.bbs.clone(),
        thread_id: arg.key.clone().ok_or("")?.parse()?,
        id,
        name,
        mail: sanitize(&arg.mail),
        poster_id: poster_id.to_string(),
        body: sanitize(&arg.message),
    }
    .into_active_model()
    .insert(trx)
    .await?;

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
    submit: String,
    bbs: String,
    key: Option<String>,
    MESSAGE: String,
    FROM: String,
    mail: String,
    subject: Option<String>,
}

impl From<RawArgument> for Argument {
    fn from(value: RawArgument) -> Self {
        Self {
            submit: value.submit,
            bbs: value.bbs,
            key: value.key,
            message: value.MESSAGE.trim_end().to_string(),
            from: value.FROM.trim().to_string(),
            mail: value.mail.trim().to_string(),
            subject: value.subject.map(|s| s.trim().to_string()),
        }
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
