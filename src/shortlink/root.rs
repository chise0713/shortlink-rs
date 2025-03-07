use std::str::FromStr;

use worker::{Request, Response, Result, RouteContext, Url};

macro_rules! redirect_if_prefix {
    ($path:expr, { $($prefix:literal => $url:expr),* $(,)? }) => {
        $(
        if $path.to_lowercase().starts_with($prefix) {
            let path = &$path[$prefix.len()..];
            let redir_url = Url::from_str(&format!($url, path))?;
            return Response::redirect_with_status(redir_url, 301);
        }
        )*
    };
}

pub async fn root(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.kv("KVNamespace")?;
    let path = &req.url()?.path().to_string()[1..];

    redirect_if_prefix!(path, {
        "p" => "https://asen.page/{}/",
        "av" => "https://www.bilibili.com/video/av{}/",
        "bv" => "https://www.bilibili.com/video/BV{}/",
        "cv" => "https://www.bilibili.com/video/cv{}/",
        "yt" => "https://www.youtube.com/watch?v={}",
    });

    if let Some(link) = kv.get(path).text().await? {
        let redir_url = Url::from_str(&link)?;
        Response::redirect_with_status(redir_url, 301)
    } else {
        Response::error("Not Found", 404)
    }
}
