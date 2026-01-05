use worker::*;

mod config;
mod parsers;
mod utils;

use config::process_config;
use parsers::parse_subscription;

const UI_HTML: &str = include_str!("ui.html");

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        .get("/", |_, _| {
            Response::from_html(UI_HTML)
        })
        .get_async("/sub", handle_config)
        .options("/sub", |_, _| {
            let headers = worker::Headers::new();
            headers.set("Access-Control-Allow-Origin", "*")?;
            headers.set("Access-Control-Allow-Methods", "GET, OPTIONS")?;
            headers.set("Access-Control-Allow-Headers", "Content-Type")?;
            Response::ok("").map(|r| r.with_headers(headers))
        })
        .run(req, env)
        .await
}

async fn handle_config(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    // Parse query parameters
    let req_url = req.url()?;
    let query_params = req_url.query_pairs();

    let mut params = std::collections::HashMap::new();

    for (key, value) in query_params {
        params.insert(key.to_string(), value.to_string());
    }

    // Get subscription URLs from 'urls' parameter
    let urls_param = params
        .get("urls")
        .ok_or_else(|| Error::RustError("Missing 'urls' parameter".to_string()))?
        .clone();

    // Split multiple URLs by pipe separator
    let subscription_urls: Vec<&str> = urls_param.split('|').collect();

    // Fetch subscription content
    let user_agent = params
        .get("ua")
        .or_else(|| params.get("UA"))
        .map(|s| s.as_str())
        .unwrap_or("v2rayng");

    // Fetch and parse all subscriptions
    let mut all_nodes = Vec::new();

    for subscription_url in subscription_urls.iter() {
        let url = subscription_url.trim();
        if url.is_empty() {
            continue;
        }

        let headers = Headers::new();
        headers.set("User-Agent", user_agent)?;

        let mut init = RequestInit::new();
        init.headers = headers;

        let fetch_req = Request::new_with_init(url, &init)?;

        let mut resp = match Fetch::Request(fetch_req).send().await {
            Ok(r) => r,
            Err(_) => continue, // Skip failed subscriptions
        };

        let status = resp.status_code();

        if status != 200 {
            continue;
        }

        let content = match resp.text().await {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Parse subscription
        if let Ok(nodes) = parse_subscription(&content) {
            all_nodes.extend(nodes);
        }
    }

    if all_nodes.is_empty() {
        return Response::error("No nodes parsed from any subscription", 400);
    }

    // Apply filters
    let emoji = params
        .get("emoji")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let prefix = params.get("prefix").map(|s| s.as_str()).unwrap_or("");

    // Process nodes with filters
    let mut processed_nodes = all_nodes;

    if emoji == 1 {
        processed_nodes = processed_nodes
            .into_iter()
            .map(|mut node| {
                if let Some(tag) = node.get_mut("tag") {
                    if let Some(tag_str) = tag.as_str() {
                        *tag = serde_json::Value::String(utils::add_emoji(tag_str));
                    }
                }
                node
            })
            .collect();
    }

    if !prefix.is_empty() {
        processed_nodes = processed_nodes
            .into_iter()
            .map(|mut node| {
                if let Some(tag) = node.get_mut("tag") {
                    if let Some(tag_str) = tag.as_str() {
                        *tag = serde_json::Value::String(format!("{}{}", prefix, tag_str));
                    }
                }
                node
            })
            .collect();
    }

    // Load config template
    let template_str = if let Some(config_url) = params.get("config") {
        // Fetch remote template
        match Request::new(config_url, worker::Method::Get) {
            Ok(template_req) => {
                match Fetch::Request(template_req).send().await {
                    Ok(mut template_resp) => {
                        if template_resp.status_code() == 200 {
                            // Validate JSON before using
                            if let Ok(text) = template_resp.text().await {
                                if serde_json::from_str::<serde_json::Value>(&text).is_ok() {
                                    Some(text)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    },
                    Err(_) => None
                }
            },
            Err(_) => None
        }
    } else {
        None
    };

    let template_index = params
        .get("file")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let config_json = process_config(processed_nodes, template_index, template_str.as_deref())?;

    // Format JSON with indentation
    let formatted_json = serde_json::to_string_pretty(&config_json)
        .map_err(|e| Error::RustError(format!("JSON serialization error: {}", e)))?;

    let headers = worker::Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;

    Ok(Response::ok(formatted_json)?.with_headers(headers))
}
