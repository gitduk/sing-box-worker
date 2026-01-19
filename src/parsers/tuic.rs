use serde_json::{json, Value};
use url::Url;
use worker::*;

pub fn parse(data: &str) -> Result<Value> {
    let url = Url::parse(data).map_err(|e| Error::RustError(format!("URL parse error: {}", e)))?;

    // Parse netloc (uuid:password@host:port)
    let uuid = url.username();
    let password = url.password().unwrap_or("");
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
        format!("tuic_{}", host)
    } else {
        tag
    };

    let mut node = json!({
        "tag": tag,
        "type": "tuic",
        "server": host,
        "server_port": port,
        "uuid": uuid,
        "password": password
    });

    // Handle congestion control
    if let Some(cc) = query.get("congestion_control") {
        node["congestion_control"] = json!(cc);
    } else {
        // Default to bbr
        node["congestion_control"] = json!("bbr");
    }

    // Handle UDP relay mode
    if let Some(udp_mode) = query.get("udp_relay_mode") {
        node["udp_relay_mode"] = json!(udp_mode);
    } else {
        // Default to native
        node["udp_relay_mode"] = json!("native");
    }

    // Handle zero RTT handshake
    if let Some(zero_rtt) = query.get("zero_rtt_handshake") {
        if zero_rtt == "1" || zero_rtt.to_lowercase() == "true" {
            node["zero_rtt_handshake"] = json!(true);
        }
    }

    // Handle heartbeat
    if let Some(heartbeat) = query.get("heartbeat") {
        node["heartbeat"] = json!(heartbeat);
    }

    // Handle TLS
    let insecure = query.get("allow_insecure") == Some(&"1".to_string())
        || query.get("allowInsecure") == Some(&"1".to_string())
        || query.get("insecure") == Some(&"1".to_string());

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

    // Handle disable_sni
    if let Some(disable_sni) = query.get("disable_sni") {
        if disable_sni == "1" || disable_sni.to_lowercase() == "true" {
            tls_config["disable_sni"] = json!(true);
        }
    }

    node["tls"] = tls_config;

    Ok(node)
}
