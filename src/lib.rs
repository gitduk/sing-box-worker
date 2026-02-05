use worker::*;

mod config;
mod error;
mod parsers;
mod utils;

use config::process_config;
use error::AppError;
use parsers::{parse_subscription, ParseResult};

const UI_HTML: &str = include_str!("ui.html");

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        .get("/", |_, _| Response::from_html(UI_HTML))
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

async fn handle_config(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match handle_config_inner(req, ctx).await {
        Ok(resp) => Ok(resp),
        Err(e) => Response::error(e.to_string(), e.status_code()),
    }
}

async fn handle_config_inner(
    req: Request,
    _ctx: RouteContext<()>,
) -> std::result::Result<Response, AppError> {
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
        .ok_or(AppError::MissingField("urls"))?
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
        if let Ok(ParseResult { nodes, sub_urls }) = parse_subscription(&content) {
            all_nodes.extend(nodes);

            // Fetch and parse nested subscription URLs
            for sub_url in sub_urls {
                let sub_headers = Headers::new();
                sub_headers.set("User-Agent", user_agent)?;

                let mut sub_init = RequestInit::new();
                sub_init.headers = sub_headers;

                let sub_req = match Request::new_with_init(&sub_url, &sub_init) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let mut sub_resp = match Fetch::Request(sub_req).send().await {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                if sub_resp.status_code() != 200 {
                    continue;
                }

                let sub_content = match sub_resp.text().await {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                if let Ok(sub_result) = parse_subscription(&sub_content) {
                    all_nodes.extend(sub_result.nodes);
                    // Only one level deep — don't follow sub_result.sub_urls
                }
            }
        }
    }

    if all_nodes.is_empty() {
        return Err(AppError::InvalidFormat(
            "No nodes parsed from any subscription".to_string(),
        ));
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

    // Apply enn (exclude node names) filter
    if let Some(enn_pattern) = params.get("enn") {
        if !enn_pattern.is_empty() {
            processed_nodes = utils::filter_by_keywords(processed_nodes, enn_pattern, true);
        }
    }

    // Deduplicate tags: append numbering to duplicates
    {
        let mut tag_count: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for node in &processed_nodes {
            if let Some(tag) = node.get("tag").and_then(|t| t.as_str()) {
                *tag_count.entry(tag.to_string()).or_insert(0) += 1;
            }
        }

        let mut tag_index: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for node in &mut processed_nodes {
            if let Some(tag) = node.get("tag").and_then(|t| t.as_str()).map(|s| s.to_string()) {
                if tag_count.get(&tag).copied().unwrap_or(0) > 1 {
                    let idx = tag_index.entry(tag.clone()).or_insert(0);
                    *idx += 1;
                    if let Some(tag_val) = node.get_mut("tag") {
                        *tag_val = serde_json::Value::String(format!("{} {}", tag, idx));
                    }
                }
            }
        }
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
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
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
    let formatted_json = serde_json::to_string_pretty(&config_json)?;

    let headers = worker::Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;

    Ok(Response::ok(formatted_json)?.with_headers(headers))
}
