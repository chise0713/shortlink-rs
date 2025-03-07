use serde::{Deserialize, Serialize};
use worker::{Request, Response, Result, RouteContext};

use crate::shortlink::KV_BINDING;

#[derive(Deserialize)]
struct Query {
    q: Box<str>,
}

#[derive(Serialize)]
struct Data {
    url: Box<str>,
}

macro_rules! predefined_response {
    ($path:expr, { $($prefix:literal => $url:expr),* $(,)? }) => {
        $(
        if $path.to_lowercase().starts_with($prefix) {
            let path = &$path[$prefix.len()..];
            return_response!(ok, Method::Get, "Good", Data {
                url: format!($url, &path).into(),
            })
        }
        )*
    };
}

pub async fn get(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.kv(KV_BINDING)?;
    let q = if let Ok(query) = req.query::<Query>() {
        query.q
    } else {
        return_response!(error, 400, Method::Get, "Provide q");
    };

    predefined_response!(q, {
        "p" => "https://asen.page/{}/",
        "av" => "https://www.bilibili.com/video/av{}/",
        "bv" => "https://www.bilibili.com/video/BV{}/",
        "cv" => "https://www.bilibili.com/video/cv{}/",
        "yt" => "https://www.youtube.com/watch?v={}",
    });

    if let Some(link) = kv.get(&q).text().await? {
        return_response!(ok, Method::Get, "Good", Data { url: link.into() });
    } else {
        return_response!(error, 404, Method::Get, "Not Found");
    }
}
