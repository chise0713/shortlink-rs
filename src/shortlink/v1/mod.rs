use serde::Serialize;

#[derive(Serialize)]
struct ResponseField<T> {
    ok: bool,
    msg: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T: Serialize> ResponseField<T> {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

macro_rules! __response_builder {
    ($ok_status:expr, $msg:expr, $input_data:expr) => {
        super::ResponseField {
            ok: $ok_status,
            msg: $msg.into(),
            data: $input_data,
        }
        .to_json()
    };
}

macro_rules! return_response {
    ($resp:expr, $method:expr) => {{
        let cors = worker::Cors::new().with_origins(["*"]);
        let mut headers = worker::Headers::new();
        headers.append("Content-Type", "application/json")?;
        return $resp.with_headers(headers).with_cors(&cors);
    }};
    (error, $status:expr, $method:expr, $msg:expr) => {
        return_response!(
            Response::error(__response_builder!(false, $msg, None::<()>), $status)?,
            $method
        )
    };
    (ok, $method:expr, $msg:expr) => {
        return_response!(
            Response::ok(__response_builder!(true, $msg, None::<()>))?,
            $method
        )
    };
    (ok, $method:expr, $msg:expr, $data:expr) => {
        return_response!(
            Response::ok(__response_builder!(true, $msg, Some($data)))?,
            $method
        )
    };
}

fn is_authorized(req: &worker::Request, ctx: &worker::RouteContext<()>) -> worker::Result<bool> {
    let tokens = ctx.env.var("tokens")?.to_string();
    let allowed_tokens: Box<[&str]> = serde_json::from_str(&tokens)?;
    let token_header = if let Some(token_header) = req.headers().get("Authorization")? {
        token_header
    } else {
        return Ok(false);
    };
    let token = if let Some(token) = token_header.strip_prefix("Bearer ") {
        token
    } else {
        return Ok(false);
    };
    for t in allowed_tokens {
        if t == token {
            return Ok(true);
        }
    }
    Ok(false)
}

macro_rules! validate_input_data {
    ($input_data:expr,$method:expr) => {
        if let Some(url) = $input_data.url.as_ref() {
            if !url.starts_with("https://") && !url.starts_with("http://") {
                return_response!(error, 400, $method, "Invalid URL");
            }
        } else {
            return_response!(error, 400, $method, "Invalid URL");
        }
        if $input_data.expiration.is_some() && $input_data.expiration_ttl.is_some() {
            return_response!(
                error,
                400,
                $method,
                "Provide either expiration or expirationTtl, not both"
            );
        }
        if let Some(expiration) = $input_data.expiration {
            if expiration < chrono::Utc::now().timestamp() {
                return_response!(
                    error,
                    400,
                    $method,
                    "expiration must be greater than the current time"
                );
            }
        }
        if $input_data.expiration.is_none() && $input_data.expiration_ttl.is_none() {
            $input_data.expiration_ttl = Some(2592000);
        }
        if let Some(expiration_ttl) = $input_data.expiration_ttl {
            if expiration_ttl < 60 {
                return_response!(
                    error,
                    400,
                    $method,
                    "expirationTtl must be at least 60 seconds"
                );
            }
            if !(-2147483648..=2147483647).contains(&expiration_ttl) {
                return_response!(
                    error,
                    400,
                    $method,
                    "expirationTtl must be between -2147483648 and 2147483647 (inclusive)"
                );
            }
        }
    };
}

macro_rules! import_modules {
    ($($mod_name:ident),*) => {
        $(
            mod $mod_name;
            pub use $mod_name::$mod_name;
        )*
    };
}

import_modules!(get, create, update, delete, list, default);
