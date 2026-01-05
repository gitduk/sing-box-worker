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

    // Collect all node tags
    let node_tags: Vec<String> = nodes
        .iter()
        .filter_map(|node| node.get("tag").and_then(|t| t.as_str()).map(|s| s.to_string()))
        .collect();

    // Add nodes to outbounds
    if let Some(outbounds) = config.get_mut("outbounds") {
        if let Some(outbounds_array) = outbounds.as_array_mut() {
            // Process each outbound to handle {all} placeholder and filters
            for outbound in outbounds_array.iter_mut() {
                let mut has_all_placeholder = false;
                let mut filtered_tags = node_tags.clone();

                // Check if {all} placeholder exists
                if let Some(outbound_list) = outbound.get("outbounds") {
                    if let Some(list_array) = outbound_list.as_array() {
                        for item in list_array.iter() {
                            if let Some(s) = item.as_str() {
                                if s == "{all}" {
                                    has_all_placeholder = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if has_all_placeholder {
                    // Apply filter if present
                    if let Some(filter) = outbound.get("filter") {
                        if let Some(filter_array) = filter.as_array() {
                            for filter_rule in filter_array {
                                if let Some(action) = filter_rule.get("action").and_then(|a| a.as_str()) {
                                    if let Some(keywords) = filter_rule.get("keywords").and_then(|k| k.as_array()) {
                                        let patterns: Vec<String> = keywords
                                            .iter()
                                            .filter_map(|k| k.as_str().map(|s| s.to_string()))
                                            .collect();

                                        for pattern in patterns {
                                            let regex_pattern = pattern.replace("|", "|");
                                            if let Ok(re) = regex::Regex::new(&regex_pattern) {
                                                filtered_tags.retain(|tag| {
                                                    let matches = re.is_match(tag);
                                                    match action {
                                                        "include" => matches,
                                                        "exclude" => !matches,
                                                        _ => true,
                                                    }
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Replace {all} with filtered node tags
                    if let Some(outbound_list) = outbound.get_mut("outbounds") {
                        if let Some(list_array) = outbound_list.as_array_mut() {
                            list_array.clear();
                            for tag in &filtered_tags {
                                list_array.push(serde_json::Value::String(tag.clone()));
                            }
                        }
                    }
                }

                // Remove filter field as it's not part of sing-box standard
                if outbound.is_object() {
                    if let Some(obj) = outbound.as_object_mut() {
                        obj.remove("filter");
                    }
                }
            }

            // Recursively remove empty outbounds and clean up references
            let mut removed_tags = std::collections::HashSet::new();
            let mut changed = true;
            let mut iterations = 0;
            const MAX_ITERATIONS: usize = 100;

            while changed && iterations < MAX_ITERATIONS {
                changed = false;
                iterations += 1;

                // Collect tags of outbounds with empty outbounds list
                let mut newly_removed = std::collections::HashSet::new();
                for outbound in outbounds_array.iter() {
                    if let Some(tag) = outbound.get("tag").and_then(|t| t.as_str()) {
                        if !removed_tags.contains(tag) {
                            if let Some(outbound_list) = outbound.get("outbounds") {
                                if let Some(list_array) = outbound_list.as_array() {
                                    if list_array.is_empty() {
                                        newly_removed.insert(tag.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                if !newly_removed.is_empty() {
                    removed_tags.extend(newly_removed.clone());
                    changed = true;

                    // Remove references to deleted outbounds from remaining outbounds
                    for outbound in outbounds_array.iter_mut() {
                        if let Some(outbound_list) = outbound.get_mut("outbounds") {
                            if let Some(list_array) = outbound_list.as_array_mut() {
                                list_array.retain(|item| {
                                    if let Some(tag) = item.as_str() {
                                        !newly_removed.contains(tag)
                                    } else {
                                        true
                                    }
                                });
                            }
                        }
                    }
                }
            }

            // Finally remove all outbounds with empty outbounds list
            outbounds_array.retain(|outbound| {
                if let Some(outbound_list) = outbound.get("outbounds") {
                    if let Some(list_array) = outbound_list.as_array() {
                        // Keep if outbounds list is not empty
                        !list_array.is_empty()
                    } else {
                        // Keep if outbounds is not an array (shouldn't happen)
                        true
                    }
                } else {
                    // Keep if no outbounds field (e.g., direct, block, dns outbounds)
                    true
                }
            });

            // Append all nodes
            for node in nodes {
                outbounds_array.push(node);
            }
        }
    }

    Ok(config)
}
