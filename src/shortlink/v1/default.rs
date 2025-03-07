use worker::{Request, Response, Result, RouteContext};

pub async fn default(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    if ["get", "create", "update", "delete", "list"]
        .iter()
        .any(|p| req.path().strip_prefix("/api/v1/") == Some(*p))
    {
        Response::error("Method Not Allowed", 405)
    } else {
        Response::error("Not Found", 404)
    }
}
