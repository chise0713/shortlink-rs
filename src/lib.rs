mod shortlink;

use worker::{event, Context, Env, Request, Response, Result, Router};

#[event(start)]
fn start() {
    console_error_panic_hook::set_once();
}

#[event(fetch, respond_with_errors)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/:_", shortlink::root)
        .on_async("/api/v1/:_", shortlink::v1::default)
        .get_async("/api/v1/get", shortlink::v1::get)
        .post_async("/api/v1/create", shortlink::v1::create)
        .put_async("/api/v1/update", shortlink::v1::update)
        .delete_async("/api/v1/delete", shortlink::v1::delete)
        .get_async("/api/v1/list", shortlink::v1::list)
        .options_async("/api/v1/get", shortlink::options::get)
        .options_async("/api/v1/create", shortlink::options::post)
        .options_async("/api/v1/update", shortlink::options::put)
        .options_async("/api/v1/delete", shortlink::options::delete)
        .options_async("/api/v1/list", shortlink::options::get)
        .run(req, env)
        .await
}
