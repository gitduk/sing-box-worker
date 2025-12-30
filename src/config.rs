use serde_json::Value;
use worker::*;

// Default config templates
const TEMPLATE_BASIC: &str = include_str!("../templates/basic.json");

pub fn process_config(nodes: Vec<Value>, template_index: usize, custom_template: Option<&str>) -> Result<Value> {
    // Load template
    let template_str = if let Some(custom) = custom_template {
        custom
    } else {
        match template_index {
            0 => TEMPLATE_BASIC,
            _ => TEMPLATE_BASIC,
        }
    };

    let mut config: Value = serde_json::from_str(template_str)
        .map_err(|e| Error::RustError(format!("Template parse error: {}", e)))?;

    // Add nodes to outbounds
    if let Some(outbounds) = config.get_mut("outbounds") {
        if let Some(outbounds_array) = outbounds.as_array_mut() {
            // Find proxy selector
            if let Some(proxy_selector) = outbounds_array
                .iter_mut()
                .find(|o| o.get("tag").and_then(|t| t.as_str()) == Some("Proxy"))
            {
                // Add node tags to selector
                if let Some(selector_outbounds) = proxy_selector.get_mut("outbounds") {
                    if let Some(selector_array) = selector_outbounds.as_array_mut() {
                        for node in &nodes {
                            if let Some(tag) = node.get("tag") {
                                selector_array.push(tag.clone());
                            }
                        }
                    }
                }
            }

            // Append all nodes
            for node in nodes {
                outbounds_array.push(node);
            }
        }
    }

    Ok(config)
}
