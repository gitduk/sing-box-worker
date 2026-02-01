use serde_json::{json, Value};
use url::Url;

use crate::error::AppError;

pub fn parse(data: &str) -> Result<Value, AppError> {
    let url = Url::parse(data)?;

    let password = url.username();
    let host = url.host_str().ok_or(AppError::MissingField("host"))?;
    let port = url.port().ok_or(AppError::MissingField("port"))?;

    let query: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();

    let tag = urlencoding::decode(url.fragment().unwrap_or(""))
        .map_err(|e| AppError::InvalidFormat(format!("Fragment decode error: {}", e)))?
        .to_string();

    let tag = if tag.is_empty() {
        format!("trojan_{}", host)
    } else {
        tag
    };

    let mut node = json!({
        "tag": tag,
        "type": "trojan",
        "server": host,
        "server_port": port,
        "password": password,
        "tls": {
            "enabled": true,
            "insecure": query.get("allowInsecure") == Some(&"1".to_string())
        }
    });

    // Handle TLS options
    if let Some(tls) = node.get_mut("tls") {
        if let Some(alpn) = query.get("alpn") {
            let alpn_list: Vec<String> = alpn
                .trim_matches(|c| c == '{' || c == '}')
                .split(',')
                .map(|s| s.to_string())
                .collect();
            tls["alpn"] = json!(alpn_list);
        }

        if let Some(sni) = query.get("sni") {
            tls["server_name"] = json!(sni);
        }

        if let Some(fp) = query.get("fp") {
            tls["utls"] = json!({
                "enabled": true,
                "fingerprint": fp
            });
        }
    }

    // Handle transport
    if let Some(transport_type) = query.get("type") {
        match transport_type.as_str() {
            "ws" => {
                let path = query.get("path").map(|s| s.as_str()).unwrap_or("/");
                let clean_path = path.split("?ed=").next().unwrap_or(path);

                let mut transport = json!({
                    "type": "ws",
                    "path": clean_path
                });

                if let Some(host) = query.get("host") {
                    transport["headers"] = json!({
                        "Host": host
                    });
                }

                // Handle early data
                if let Some(ed_str) = path.split("?ed=").nth(1) {
                    if let Ok(ed_value) = ed_str.parse::<u32>() {
                        transport["early_data_header_name"] = json!("Sec-WebSocket-Protocol");
                        transport["max_early_data"] = json!(ed_value);
                    }
                }

                node["transport"] = transport;
            }
            "h2" => {
                let mut transport = json!({
                    "type": "http"
                });

                if let Some(host) = query.get("host") {
                    transport["host"] = json!(host);
                } else {
                    transport["host"] = json!(node["server"]);
                }

                if let Some(path) = query.get("path") {
                    transport["path"] = json!(path);
                } else {
                    transport["path"] = json!("/");
                }

                node["transport"] = transport;
            }
            "grpc" => {
                node["transport"] = json!({
                    "type": "grpc",
                    "service_name": query.get("serviceName").map(|s| s.as_str()).unwrap_or("")
                });
            }
            _ => {}
        }
    }

    Ok(node)
}
