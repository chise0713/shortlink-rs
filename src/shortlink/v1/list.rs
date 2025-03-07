use serde::{Deserialize, Serialize};
use worker::{Request, Response, Result, RouteContext};

use super::is_authorized;
use crate::shortlink::KV_BINDING;

#[derive(Deserialize)]
struct Query {
    q: Option<Box<str>>,
    c: Option<Box<str>>,
    all: Option<bool>,
}

#[derive(Serialize)]
struct Short {
    key: Box<str>,
    #[serde(rename = "noHttps")]
    no_https: Box<str>,
    full: Box<str>,
}

#[derive(Serialize)]
struct Link {
    short: Short,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<Box<str>>,
    expiration: i64,
}

#[derive(Serialize)]
struct Data {
    cursor: Option<Box<str>>,
    list_complete: bool,
    links: Box<[Link]>,
}

pub async fn list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_authorized(&req, &ctx)? {
        return_response!(error, 403, Method::Get, "Forbidden");
    };
    let kv = ctx.kv(KV_BINDING)?;

    let query = if let Ok(query) = req.query::<Query>() {
        query
    } else {
        return_response!(error, 400, Method::Get, "Provide q");
    };

    let mut cursor = query.c.map(|c| c.into_string());
    let mut final_entries;
    let list_complete;
    if query.all.unwrap_or(false) {
        let mut all_entries = Vec::new();
        loop {
            let list_builder = kv.list();
            let mut entries = if let Some(cursor) = cursor {
                list_builder.cursor(cursor).execute().await?
            } else {
                list_builder.execute().await?
            };
            cursor = entries.cursor;
            all_entries.append(&mut entries.keys);
            if entries.list_complete {
                list_complete = true;
                break;
            }
        }
        final_entries = all_entries;
    } else {
        let list_builder = kv.list();
        let entries = if let Some(cursor) = &cursor {
            list_builder.cursor(cursor.clone()).execute().await?
        } else {
            list_builder.execute().await?
        };
        list_complete = entries.list_complete;
        final_entries = entries.keys;
    }

    if let Some(query) = query.q {
        final_entries = final_entries
            .into_iter()
            .filter(|key| {
                if let Some(metadata) = &key.metadata {
                    metadata.to_string().contains(query.as_ref())
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();
    }
    let domain = req.url()?.domain().unwrap().to_string();
    let data = Data {
        cursor: cursor.map(|c| c.into_boxed_str()),
        list_complete,
        links: final_entries
            .iter()
            .map(|key| {
                let url_str: Box<str> = key
                    .metadata
                    .clone()
                    .unwrap_or_default()
                    .to_string()
                    .strip_prefix('"')
                    .unwrap()
                    .strip_suffix('"')
                    .unwrap()
                    .into();

                let short = Short {
                    key: key.name.clone().into(),
                    no_https: format!("{}/{}", domain, key.name).into(),
                    full: format!("https://{}/{}", domain, key.name).into(),
                };

                let url = if url_str.len() == 1022 {
                    None
                } else {
                    Some(url_str)
                };

                Link {
                    short,
                    url,
                    expiration: key.expiration.unwrap().try_into().unwrap(),
                }
            })
            .collect(),
    };

    return_response!(ok, Method::Get, "Success", data);
}
