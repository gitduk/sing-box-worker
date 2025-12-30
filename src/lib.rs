use worker::*;

mod config;
mod parsers;
mod utils;

use config::process_config;
use parsers::parse_subscription;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        .get("/", |_, _| {
            Response::ok("sing-box Subscription Converter - Rust + Cloudflare Workers Edition")
        })
        .get_async("/sub", handle_config)
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
    console_log!("Processing {} subscription(s)", subscription_urls.len());

    // Fetch subscription content
    let user_agent = params
        .get("ua")
        .or_else(|| params.get("UA"))
        .map(|s| s.as_str())
        .unwrap_or("v2rayng");

    console_log!("User-Agent: {}", user_agent);

    // Fetch and parse all subscriptions
    let mut all_nodes = Vec::new();

    for (index, subscription_url) in subscription_urls.iter().enumerate() {
        let url = subscription_url.trim();
        if url.is_empty() {
            continue;
        }

        console_log!("Fetching subscription {} of {}: {}", index + 1, subscription_urls.len(), url);

        let headers = Headers::new();
        headers.set("User-Agent", user_agent)?;

        let mut init = RequestInit::new();
        init.headers = headers;

        let fetch_req = Request::new_with_init(url, &init)?;

        console_log!("Sending request...");
        let mut resp = match Fetch::Request(fetch_req).send().await {
            Ok(r) => r,
            Err(e) => {
                console_log!("Error fetching subscription {}: {}", index + 1, e);
                continue; // Skip failed subscriptions
            }
        };

        let status = resp.status_code();
        console_log!("Response status: {}", status);

        if status != 200 {
            console_log!("Failed to fetch subscription {} (status {}), skipping", index + 1, status);
            continue;
        }

        console_log!("Reading response content...");
        let content = match resp.text().await {
            Ok(c) => c,
            Err(e) => {
                console_log!("Error reading response for subscription {}: {}", index + 1, e);
                continue;
            }
        };
        console_log!("Content length: {} bytes", content.len());

        // Parse subscription
        match parse_subscription(&content) {
            Ok(nodes) => {
                console_log!("Parsed {} nodes from subscription {}", nodes.len(), index + 1);
                all_nodes.extend(nodes);
            },
            Err(e) => {
                console_log!("Parse error for subscription {}: {}", index + 1, e);
                continue;
            }
        }
    }

    console_log!("Total nodes from all subscriptions: {}", all_nodes.len());

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
        console_log!("Fetching remote template: {}", config_url);

        match Request::new(config_url, worker::Method::Get) {
            Ok(template_req) => {
                console_log!("Sending template request...");
                match Fetch::Request(template_req).send().await {
                    Ok(mut template_resp) => {
                        let status = template_resp.status_code();
                        console_log!("Template response status: {}", status);

                        if status != 200 {
                            console_log!("Failed to fetch template (status {}), using default", status);
                            None
                        } else {
                            match template_resp.text().await {
                                Ok(text) => {
                                    console_log!("Template fetched successfully, length: {} bytes", text.len());
                                    Some(text)
                                },
                                Err(e) => {
                                    console_log!("Error reading template response: {}", e);
                                    None
                                }
                            }
                        }
                    },
                    Err(e) => {
                        console_log!("Error fetching template: {}", e);
                        None
                    }
                }
            },
            Err(e) => {
                console_log!("Error creating template request: {}", e);
                None
            }
        }
    } else {
        console_log!("No config URL provided, using built-in template");
        None
    };

    let template_index = params
        .get("file")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    console_log!("Processing config with {} nodes, template_index: {}", processed_nodes.len(), template_index);

    let config_json = match process_config(processed_nodes, template_index, template_str.as_deref()) {
        Ok(config) => {
            console_log!("Config processed successfully");
            config
        },
        Err(e) => {
            console_log!("Error processing config: {}", e);
            return Response::error(format!("Config processing error: {}", e), 500);
        }
    };

    console_log!("Returning JSON response");
    Response::from_json(&config_json)
}
