use serde::{Deserialize, Serialize};
use worker::{Request, Response, Result, RouteContext};

use super::is_authorized;
use crate::shortlink::KV_BINDING;

#[derive(Deserialize)]
struct InputData {
    url: Option<Box<str>>,
    length: Option<i64>,
    number: Option<bool>,
    capital: Option<bool>,
    lowercase: Option<bool>,
    expiration: Option<i64>,
    #[serde(rename = "expirationTtl")]
    expiration_ttl: Option<i64>,
}

#[derive(Serialize)]
struct Data {
    short: Box<str>,
}

fn generate_random_string(characters: &str, max_len: usize) -> Box<str> {
    let mut short_link = String::new();
    let mut i: usize = 0;
    while short_link.len() < max_len {
        i += 1;
        let char = characters
            .chars()
            .nth(getrandom::u32().unwrap() as u8 as usize % characters.len())
            .unwrap();
        short_link.push(char);
        if i <= 2 && short_link.len() > 2 {
            let prefix = short_link[..2].to_string().to_lowercase();
            if prefix.starts_with("p")
                || prefix.starts_with("av")
                || prefix.starts_with("bv")
                || prefix.starts_with("cv")
                || prefix.starts_with("yt")
            {
                short_link.clear();
                i = 0;
                continue;
            }
        }
    }
    short_link.into_boxed_str()
}

pub async fn create(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_authorized(&req, &ctx)? {
        return_response!(error, 403, Method::Post, "Forbidden");
    };
    let kv = ctx.kv(KV_BINDING)?;
    let mut input_data = if let Ok(input_data) = req.json::<InputData>().await {
        input_data
    } else {
        return_response!(error, 400, Method::Post, "Invalid JSON");
    };

    validate_input_data!(input_data, Method::Post);
    let redir_url = input_data.url.unwrap();

    let mut characters = String::new();
    if input_data.number.unwrap_or(true) {
        characters += "0123456789";
    }
    if input_data.capital.unwrap_or(true) {
        characters += "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    }
    if input_data.lowercase.unwrap_or(true) {
        characters += "abcdefghijklmnopqrstuvwxyz";
    }
    let characters = characters.into_boxed_str();

    let metainput_data = if redir_url.len() > 1022 {
        &redir_url.clone()[0..1022]
    } else {
        &redir_url.clone()
    };
    let short_link = loop {
        let max_len = input_data.length.unwrap_or(6) as usize;
        let s = generate_random_string(&characters, max_len);
        if kv.get(&s).text().await?.is_none() {
            break s;
        }
    };

    let mut builder = kv.put(&short_link, redir_url)?.metadata(metainput_data)?;
    if let Some(exp) = input_data.expiration {
        builder = builder.expiration(exp.try_into().unwrap());
    } else if let Some(ttl) = input_data.expiration_ttl {
        builder = builder.expiration_ttl(ttl.try_into().unwrap());
    }
    builder.execute().await?;

    let domain = req.url()?.domain().unwrap().to_string();
    return_response!(
        ok,
        Method::Post,
        "Good",
        Data {
            short: format!("{}/{}", domain, short_link).into(),
        }
    );
}
