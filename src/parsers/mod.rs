pub mod hysteria2;
pub mod shadowsocks;
pub mod trojan;
pub mod tuic;
pub mod vless;
pub mod vmess;

use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;

use crate::error::AppError;

pub struct ParseResult {
    pub nodes: Vec<Value>,
    pub sub_urls: Vec<String>,
}

pub fn parse_subscription(content: &str) -> Result<ParseResult, AppError> {
    let mut nodes = Vec::new();
    let mut sub_urls = Vec::new();

    // Try to decode entire content as base64 first
    let decoded_content = match general_purpose::STANDARD.decode(content.trim()) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => content.to_string(),
    };

    // Parse each line
    for line in decoded_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check if line is an HTTP/HTTPS subscription URL
        if line.starts_with("http://") || line.starts_with("https://") {
            sub_urls.push(line.to_string());
            continue;
        }

        // Try to decode line as base64 (for subscriptions with per-line encoding)
        let decoded_line = match general_purpose::STANDARD.decode(line) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => line.to_string(),
        };

        // Decoded base64 may contain multiple URIs separated by newlines
        for sub_line in decoded_line.lines() {
            let sub_line = sub_line.trim();
            if sub_line.is_empty() || sub_line.starts_with('#') {
                continue;
            }
            // Check decoded lines for HTTP/HTTPS URLs too
            if sub_line.starts_with("http://") || sub_line.starts_with("https://") {
                sub_urls.push(sub_line.to_string());
                continue;
            }
            if let Some(node) = parse_node(sub_line) {
                nodes.push(node);
            }
        }
    }

    Ok(ParseResult { nodes, sub_urls })
}

fn parse_node(line: &str) -> Option<Value> {
    if line.starts_with("vmess://") {
        vmess::parse(line).ok()
    } else if line.starts_with("vless://") {
        vless::parse(line).ok()
    } else if line.starts_with("trojan://") {
        trojan::parse(line).ok()
    } else if line.starts_with("ss://") {
        shadowsocks::parse(line).ok()
    } else if line.starts_with("hysteria2://") || line.starts_with("hy2://") {
        hysteria2::parse(line).ok()
    } else if line.starts_with("tuic://") {
        tuic::parse(line).ok()
    } else {
        None
    }
}
