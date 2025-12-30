use base64::{engine::general_purpose, Engine as _};
use serde_json::{json, Value};
use url::Url;
use worker::*;

pub fn parse(data: &str) -> Result<Value> {
    let url = Url::parse(data).map_err(|e| Error::RustError(format!("URL parse error: {}", e)))?;

    // Decode userinfo (method:password)
    let userinfo = url.username();
    let decoded_userinfo = if userinfo.contains(':') {
        userinfo.to_string()
    } else {
        // Try base64 decode
        match general_purpose::STANDARD.decode(userinfo) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => userinfo.to_string(),
        }
    };

    let parts: Vec<&str> = decoded_userinfo.split(':').collect();
    if parts.len() < 2 {
        return Err(Error::RustError("Invalid shadowsocks format".to_string()));
    }

    let method = parts[0];
    let password = parts[1..].join(":");

    let host = url
        .host_str()
        .ok_or_else(|| Error::RustError("Missing host".to_string()))?;
    let port = url
        .port()
        .ok_or_else(|| Error::RustError("Missing port".to_string()))?;

    let tag = urlencoding::decode(url.fragment().unwrap_or(""))
        .map_err(|e| Error::RustError(format!("Fragment decode error: {}", e)))?
        .to_string();

    let tag = if tag.is_empty() {
        format!("ss_{}", host)
    } else {
        tag
    };

    let query: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();

    let mut node = json!({
        "tag": tag,
        "type": "shadowsocks",
        "server": host,
        "server_port": port,
        "method": method,
        "password": password
    });

    // Handle plugin
    if let Some(plugin) = query.get("plugin") {
        if plugin.starts_with("obfs") || plugin.contains("simple-obfs") {
            // Parse obfs parameters
            if let Some(opts) = plugin.split(';').nth(1) {
                let obfs_params: std::collections::HashMap<String, String> = opts
                    .split(';')
                    .filter_map(|s| {
                        let parts: Vec<&str> = s.split('=').collect();
                        if parts.len() == 2 {
                            Some((parts[0].to_string(), parts[1].to_string()))
                        } else {
                            None
                        }
                    })
                    .collect();

                if let Some(obfs_type) = obfs_params.get("obfs") {
                    node["plugin"] = json!("obfs-local");
                    node["plugin_opts"] = json!(format!(
                        "obfs={};obfs-host={}",
                        obfs_type,
                        obfs_params
                            .get("obfs-host")
                            .map(|s| s.as_str())
                            .unwrap_or("")
                    ));
                }
            }
        } else if plugin.contains("v2ray-plugin") {
            node["plugin"] = json!("v2ray-plugin");
        }
    }

    Ok(node)
}
