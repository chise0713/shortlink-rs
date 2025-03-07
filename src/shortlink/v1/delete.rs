use serde::Deserialize;
use worker::{Request, Response, Result, RouteContext};

use super::is_authorized;
use crate::shortlink::KV_BINDING;

#[derive(Deserialize)]
struct InputData {
    short: Option<Box<str>>,
}

pub async fn delete(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_authorized(&req, &ctx)? {
        return_response!(error, 403, Method::Delete, "Forbidden");
    };
    let kv = ctx.kv(KV_BINDING)?;
    let input_data = if let Ok(input_data) = req.json::<InputData>().await {
        input_data
    } else {
        return_response!(error, 400, Method::Delete, "Invalid JSON");
    };
    let short_link = if let Some(short_link) = input_data.short {
        short_link
    } else {
        return_response!(error, 400, Method::Delete, "Provide short");
    };

    if kv.get(&short_link).text().await?.is_none() {
        return_response!(
            error,
            400,
            Method::Delete,
            "Delete a non-exist short link is not permit"
        );
    };
    kv.delete(&short_link).await?;

    return_response!(ok, Method::Delete, "Good");
}
