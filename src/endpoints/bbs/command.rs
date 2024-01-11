use crate::{endpoints::bbs::Argument, utils, Error, Server};
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

pub async fn run(state: Arc<Server>, header: HeaderMap, arg: Argument) -> Result<(), Error> {
    Ok(())
}
