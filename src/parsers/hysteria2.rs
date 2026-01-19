use serde_json::{json, Value};
use url::Url;
use worker::*;

pub fn parse(data: &str) -> Result<Value> {
    let url = Url::parse(data).map_err(|e| Error::RustError(format!("URL parse error: {}", e)))?;

    // Parse netloc (password@host:port)
    let password = url.username();
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
        format!("hysteria2_{}", host)
    } else {
        tag
    };

    let mut node = json!({
        "tag": tag,
        "type": "hysteria2",
        "server": host,
        "server_port": port,
        "password": password
    });

    // Handle TLS
    let security = query.get("security").map(|s| s.as_str());
    if security == Some("tls") || security.is_none() {
        let insecure = query.get("insecure") == Some(&"1".to_string())
            || query.get("allowInsecure") == Some(&"1".to_string());

        let mut tls_config = json!({
            "enabled": true,
            "insecure": insecure
        });

        if let Some(sni) = query.get("sni") {
            if !sni.is_empty() {
                tls_config["server_name"] = json!(sni);
            }
        }

        if let Some(alpn) = query.get("alpn") {
            // ALPN can be comma-separated
            let alpn_list: Vec<&str> = alpn.split(',').collect();
            tls_config["alpn"] = json!(alpn_list);
        }

        node["tls"] = tls_config;
    }

    // Handle obfs if present
    if let Some(obfs) = query.get("obfs") {
        if !obfs.is_empty() && obfs != "none" {
            let mut obfs_config = json!({
                "type": obfs
            });

            if let Some(obfs_password) = query.get("obfs-password").or_else(|| query.get("obfsPassword")) {
                obfs_config["password"] = json!(obfs_password);
            }

            node["obfs"] = obfs_config;
        }
    }

    // Handle up/down speeds
    if let Some(up) = query.get("up") {
        if let Ok(up_mbps) = up.parse::<u64>() {
            node["up_mbps"] = json!(up_mbps);
        }
    }

    if let Some(down) = query.get("down") {
        if let Ok(down_mbps) = down.parse::<u64>() {
            node["down_mbps"] = json!(down_mbps);
        }
    }

    Ok(node)
}
