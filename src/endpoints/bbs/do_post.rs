use crate::{endpoints::bbs::Argument, utils, Error, Server};
use entity::*;

use std::sync::Arc;

use axum::http::HeaderMap;
use rand::Rng;
use sea_orm::{ActiveModelTrait, IntoActiveModel, TransactionTrait};

pub async fn run(state: Arc<Server>, header: HeaderMap, mut arg: Argument) -> Result<(), Error> {
    validation(&mut arg)?;

    let timestamp = chrono::Local::now().timestamp();
    let id = (timestamp << 16) | (rand::thread_rng().gen::<u16>() as i64);

    let ip = header.get("X-Real-IP").ok_or("")?;
    let mut hashed_ip = utils::hasher(
        ip.as_bytes(),
        &state.salt,
        Some(&(timestamp / 604800).to_le_bytes()),
    );
    hashed_ip.truncate(22);

    let trx = state.db.begin().await?;

    if arg.submit == "新規スレッド作成" {
        thread::Model {
            board_id: arg.bbs.clone(),
            id: timestamp,
            name: arg.subject.ok_or("")?.clone(),
        }
        .into_active_model()
        .insert(&trx)
        .await?;

        arg.key = Some(timestamp.to_string());
    }

    post::Model {
        board_id: arg.bbs.clone(),
        thread_id: arg.key.clone().ok_or("")?.parse()?,
        id,
        name,
        mail: utils::sanitize(&arg.mail),
        poster_id: hashed_ip,
        body: utils::sanitize(&arg.message),
    }
    .into_active_model()
    .insert(&trx)
    .await?;

    trx.commit().await?;

    Ok(())
}

pub fn validation(arg: &mut Argument) -> Result<(), Error> {
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

    if arg.submit == "新規スレッド作成" {
        if page.is_some() {
            return Err("".into());
        }
        if arg.subject.as_ref().ok_or("")?.is_empty() {
            return Err("".into());
        }

        if arg.mail == "sage" {
            arg.mail = "".to_string();
        }
    } else if arg.submit == "書き込む" {
    } else {
        return Err("".into());
    }

    Ok(())
}

pub async fn name_process(
    state: Arc<Server>,
    header: &HeaderMap,
    arg: &mut Argument,
) -> Result<(), Error> {
    if arg.from.is_empty() {
        arg.from = "</b>Anonymous<b>".to_string();
    } else if arg.from == "fusianasan" {
        arg.from = format!("</b>{}<b>", header.get("X-Real-IP").ok_or("")?.to_str()?);
    } else {
        let mut name = arg.from.clone();
        let mut addition = String::new();

        if let Some((name_, trip)) = name.split_once('#') {
            addition.push_str(&utils::hasher(trip.as_bytes(), &state.salt, None));
            name = name_.to_string();
        }

        arg.from = format!(utils::sanitize(&name);
    }

    Ok(())
}
