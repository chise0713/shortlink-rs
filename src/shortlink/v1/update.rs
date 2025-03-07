use serde::Deserialize;
use worker::{Request, Response, Result, RouteContext};

use super::is_authorized;
use crate::shortlink::KV_BINDING;

#[derive(Deserialize)]
struct InputData {
    url: Option<Box<str>>,
    short: Option<Box<str>>,
    expiration: Option<i64>,
    #[serde(rename = "expirationTtl")]
    expiration_ttl: Option<i64>,
}

pub async fn update(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_authorized(&req, &ctx)? {
        return_response!(error, 403, Method::Put, "Forbidden");
    };
    let kv = ctx.kv(KV_BINDING)?;
    let mut input_data = if let Ok(input_data) = req.json::<InputData>().await {
        input_data
    } else {
        return_response!(error, 400, Method::Put, "Invalid JSON");
    };

    validate_input_data!(input_data, Method::Put);

    let short_link = if let Some(short_link) = input_data.short {
        short_link
    } else {
        return_response!(error, 400, Method::Put, "No short link is provided");
    };
    let short_value = if let Some(short_value) = kv.get(&short_link).text().await? {
        short_value
    } else {
        return_response!(
            error,
            400,
            Method::Put,
            "Update a non-exist short link is not permit"
        );
    };
    let url = input_data.url.unwrap_or(short_value.into());

    let metadata = if url.len() > 1022 {
        &url.clone()[0..1022]
    } else {
        &url.clone()
    };

    let mut builder = kv.put(&short_link, url)?.metadata(metadata)?;
    if let Some(exp) = input_data.expiration {
        builder = builder.expiration(exp.try_into().unwrap());
    } else if let Some(ttl) = input_data.expiration_ttl {
        builder = builder.expiration_ttl(ttl.try_into().unwrap());
    }
    builder.execute().await?;

    return_response!(ok, Method::Put, "Good");
}
