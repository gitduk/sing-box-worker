use base64::{engine::general_purpose, Engine as _};
use serde_json::{json, Value};
use url::Url;

use crate::error::AppError;

pub fn parse(data: &str) -> Result<Value, AppError> {
    // Try URL decoding first in case the URL is encoded
    let decoded_data =
        urlencoding::decode(data).unwrap_or_else(|_| std::borrow::Cow::Borrowed(data));

    let url = Url::parse(decoded_data.as_ref())?;

    // Decode userinfo (method:password)
    let userinfo = url.username();

    // URL decode the userinfo first (in case it contains URL-encoded characters)
    let userinfo =
        urlencoding::decode(userinfo).unwrap_or_else(|_| std::borrow::Cow::Borrowed(userinfo));

    let decoded_userinfo = if userinfo.contains(':') {
        userinfo.to_string()
    } else {
        // SIP002 format: base64 encoded userinfo
        // Add padding if needed for base64 decode
        let padded_userinfo = match userinfo.len() % 4 {
            2 => format!("{}==", userinfo),
            3 => format!("{}=", userinfo),
            _ => userinfo.to_string(),
        };

        // Try base64 decode
        match general_purpose::STANDARD.decode(&padded_userinfo) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(e) => {
                return Err(AppError::Base64Decode(e));
            }
        }
    };

    let parts: Vec<&str> = decoded_userinfo.split(':').collect();
    if parts.len() < 2 {
        return Err(AppError::InvalidFormat(format!(
            "Invalid shadowsocks format, decoded userinfo: {}",
            decoded_userinfo
        )));
    }

    let method = parts[0];
    let password = parts[1..].join(":");

    let host = url.host_str().ok_or(AppError::MissingField("host"))?;
    let port = url.port().ok_or(AppError::MissingField("port"))?;

    let tag = urlencoding::decode(url.fragment().unwrap_or(""))
        .map_err(|e| AppError::InvalidFormat(format!("Fragment decode error: {}", e)))?
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
