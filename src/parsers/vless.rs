use serde_json::{json, Value};
use url::Url;
use worker::*;

pub fn parse(data: &str) -> Result<Value> {
    let url = Url::parse(data).map_err(|e| Error::RustError(format!("URL parse error: {}", e)))?;

    // Parse netloc (username@host:port)
    let username = url.username();
    let host = url
        .host_str()
        .ok_or_else(|| Error::RustError("Missing host".to_string()))?;
    let port = url
        .port()
        .ok_or_else(|| Error::RustError("Missing port".to_string()))?;

    let query: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();

    let tag = urlencoding::decode(url.fragment().unwrap_or(""))
        .map_err(|e| Error::RustError(format!("Fragment decode error: {}", e)))?
        .to_string();

    let tag = if tag.is_empty() {
        format!("vless_{}", host)
    } else {
        tag
    };

    let mut node = json!({
        "tag": tag,
        "type": "vless",
        "server": host,
        "server_port": port,
        "uuid": username,
        "packet_encoding": query.get("packetEncoding").map(|s| s.as_str()).unwrap_or("xudp")
    });

    // Handle flow
    if query.contains_key("flow") {
        node["flow"] = json!("xtls-rprx-vision");
    }

    // Handle TLS
    let security = query.get("security").map(|s| s.as_str()).unwrap_or("");
    if !matches!(security, "" | "none" | "None") || query.get("tls") == Some(&"1".to_string()) {
        let mut tls_config = json!({
            "enabled": true,
            "insecure": query.get("allowInsecure") == Some(&"1".to_string()),
            "server_name": query.get("sni")
                .or_else(|| query.get("peer"))
                .map(|s| s.as_str())
                .unwrap_or("")
        });

        // Handle REALITY
        if security == "reality" || query.contains_key("pbk") {
            let mut reality = json!({
                "enabled": true,
                "public_key": query.get("pbk").map(|s| s.as_str()).unwrap_or("")
            });

            if let Some(sid) = query.get("sid") {
                if !sid.is_empty() && sid.to_lowercase() != "none" {
                    reality["short_id"] = json!(sid);
                }
            }

            tls_config["reality"] = reality;
            tls_config["utls"] = json!({
                "enabled": true,
                "fingerprint": query.get("fp").map(|s| s.as_str()).unwrap_or("chrome")
            });
        } else if let Some(fp) = query.get("fp") {
            tls_config["utls"] = json!({
                "enabled": true,
                "fingerprint": fp
            });
        }

        node["tls"] = tls_config;
    }

    // Handle transport
    if let Some(transport_type) = query.get("type") {
        match transport_type.as_str() {
            "ws" => {
                let path = query.get("path").map(|s| s.as_str()).unwrap_or("/");
                let clean_path = path.split("?ed=").next().unwrap_or(path);

                let host_header = query
                    .get("host")
                    .or_else(|| query.get("sni"))
                    .filter(|s| *s != "None")
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let mut transport = json!({
                    "type": "ws",
                    "path": clean_path,
                    "headers": {
                        "Host": host_header
                    }
                });

                // Handle early data
                if let Some(ed_str) = path.split("?ed=").nth(1) {
                    if let Ok(ed_value) = ed_str.parse::<u32>() {
                        transport["early_data_header_name"] = json!("Sec-WebSocket-Protocol");
                        transport["max_early_data"] = json!(ed_value);
                    }
                }

                node["transport"] = transport;
            }
            "grpc" => {
                node["transport"] = json!({
                    "type": "grpc",
                    "service_name": query.get("serviceName").map(|s| s.as_str()).unwrap_or("")
                });
            }
            "http" => {
                node["transport"] = json!({
                    "type": "http"
                });
            }
            _ => {}
        }
    }

    Ok(node)
}
