use crate::Error;

use encoding_rs::Encoding;
use percent_encoding::NON_ALPHANUMERIC;
use serde::de::DeserializeOwned;

pub fn decord_form<T>(body: Vec<u8>, encoding: &'static Encoding) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let mut result = String::new();
    for i in body.split(|i| *i == 0x26) {
        let mut iter = i.split(|i| *i == 0x3D);

        let name = iter.next().ok_or("")?;
        let name: Vec<u8> = percent_encoding::percent_decode(name).collect();
        let name = encoding.decode(&name).0.into_owned();
        let name: String = percent_encoding::utf8_percent_encode(&name, NON_ALPHANUMERIC).collect();

        let value: Vec<u8> = iter.flatten().copied().collect();
        let value: Vec<u8> = percent_encoding::percent_decode(&value).collect();
        let value = encoding.decode(&value).0.into_owned();
        let value: String =
            percent_encoding::utf8_percent_encode(&value, NON_ALPHANUMERIC).collect();

        result.push_str(&name);
        result.push('=');
        result.push_str(&value);
        result.push('&');
    }
    result.pop();

    Ok(serde_urlencoded::from_str(&result)?)
}
