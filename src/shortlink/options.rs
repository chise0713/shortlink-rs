#![allow(dead_code)]
use paste::paste;
use worker::{Cors, Method, Request, Response, Result, RouteContext};

macro_rules! cors_function {
    ($func_name:ident, $method:expr) => {
        pub async fn $func_name(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
            let cors = Cors::new()
                .with_origins(["*"])
                .with_allowed_headers(["Content-Type", "Authorization"])
                .with_methods([Method::Options, $method])
                .with_max_age(86400);
            Response::ok("")?.with_status(204).with_cors(&cors)
        }
    };
}

macro_rules! cors_functions {
    ( $( Method::$variant:ident ),* $(,)? ) => {
        $(
            paste! {
                cors_function!( [<$variant:lower>], Method::$variant );
            }
        )*
    };
}

cors_functions!(
    Method::Head,
    Method::Get,
    Method::Post,
    Method::Put,
    Method::Patch,
    Method::Delete,
    Method::Options,
    Method::Connect,
    Method::Trace,
);
