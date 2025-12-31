pub mod shadowsocks;
pub mod trojan;
pub mod vless;
pub mod vmess;

use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use worker::*;

pub fn parse_subscription(content: &str) -> Result<Vec<Value>> {
    use worker::console_log;

    let mut nodes = Vec::new();

    console_log!("Parsing subscription, content length: {} bytes", content.len());

    // Try to decode as base64
    let decoded_content = match general_purpose::STANDARD.decode(content.trim()) {
        Ok(bytes) => {
            console_log!("Successfully decoded base64, decoded length: {} bytes", bytes.len());
            String::from_utf8_lossy(&bytes).to_string()
        },
        Err(_) => {
            console_log!("Content is not base64, using as-is");
            content.to_string()
        }
    };

    // Parse each line
    let mut line_count = 0;
    let mut parsed_count = 0;
    let mut failed_count = 0;

    for line in decoded_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        line_count += 1;

        // Determine protocol and parse
        if let Some(node) = parse_node(line) {
            parsed_count += 1;
            if let Some(tag) = node.get("tag").and_then(|t| t.as_str()) {
                console_log!("  Parsed node {}: {}", parsed_count, tag);
            }
            nodes.push(node);
        } else {
            failed_count += 1;
            // Log first part of failed line for debugging
            let preview = if line.len() > 100 { &line[..100] } else { line };
            console_log!("  Failed to parse line {}: {}...", line_count, preview);
        }
    }

    console_log!("Subscription parsing complete: {} lines, {} parsed, {} failed",
        line_count, parsed_count, failed_count);

    Ok(nodes)
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
    } else {
        None
    }
}
