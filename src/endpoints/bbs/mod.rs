mod command;
mod do_post;

use crate::{utils, Error, Server};

use std::sync::Arc;

use axum::{
    extract::{RawForm, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    response::Response,
};

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
    if !referer.starts_with(&format!("https://{}/", &state.config.domain)) {
        return Err("".into());
    }

    let arg: RawArgument = utils::decord_form(form.into_iter().collect(), encoding_rs::SHIFT_JIS)?;
    let arg = Argument::from(arg);

    if arg.mail == "command" {
        command::run(state, header, arg).await?;
    } else {
        do_post::run(state, header, arg).await?;
    }

    let mut response = HTML_OK.as_slice().into_response();
    let headers = response.headers_mut();
    headers.clear();
    headers.insert("Content-Type", "text/html; charset=Shift_JIS".try_into()?);
    headers.insert("Cache-Control", "no-store".try_into()?);

    Ok(response)
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
