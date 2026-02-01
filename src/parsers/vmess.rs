use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;

#[derive(Debug, Deserialize)]
struct VmessConfig {
    ps: Option<String>,
    add: String,
    port: String,
    id: String,
    aid: Option<String>,
    scy: Option<String>,
    net: Option<String>,
    #[serde(rename = "type")]
    _type: Option<String>,
    host: Option<String>,
    path: Option<String>,
    tls: Option<String>,
    sni: Option<String>,
    fp: Option<String>,
}

pub fn parse(data: &str) -> Result<Value, AppError> {
    let info = data
        .strip_prefix("vmess://")
        .ok_or(AppError::InvalidFormat("Invalid vmess URL".to_string()))?;

    // Decode base64
    let decoded = general_purpose::STANDARD.decode(info)?;

    let json_str = String::from_utf8(decoded)?;

    let config: VmessConfig = serde_json::from_str(&json_str)?;

    let mut node = json!({
        "tag": config.ps.unwrap_or_else(|| format!("vmess_{}", &config.add)),
        "type": "vmess",
        "server": config.add,
        "server_port": config.port.parse::<u16>().unwrap_or(443),
        "uuid": config.id,
        "security": config.scy.unwrap_or_else(|| "auto".to_string()),
        "alter_id": config.aid.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0),
        "packet_encoding": "xudp"
    });

    // Handle TLS
    if let Some(tls) = config.tls {
        if !tls.is_empty() && tls != "none" {
            let mut tls_config = json!({
                "enabled": true,
                "insecure": true,
                "server_name": config.host.clone().unwrap_or_default()
            });

            if let Some(sni) = config.sni {
                tls_config["server_name"] = json!(sni);
                if let Some(fp) = config.fp {
                    tls_config["utls"] = json!({
                        "enabled": true,
                        "fingerprint": fp
                    });
                }
            }

            node["tls"] = tls_config;
        }
    }

    // Handle transport
    if let Some(net) = config.net {
        match net.as_str() {
            "ws" => {
                let mut transport = json!({
                    "type": "ws"
                });

                if let Some(host) = config.host {
                    transport["headers"] = json!({
                        "Host": host
                    });
                }

                if let Some(path) = config.path {
                    // Remove ?ed= parameter if present
                    let clean_path = path.split("?ed=").next().unwrap_or(&path);
                    transport["path"] = json!(clean_path);

                    // Check for early data
                    if let Some(ed_str) = path.split("?ed=").nth(1) {
                        if let Ok(ed_value) = ed_str.parse::<u32>() {
                            transport["early_data_header_name"] = json!("Sec-WebSocket-Protocol");
                            transport["max_early_data"] = json!(ed_value);
                        }
                    }
                }

                node["transport"] = transport;
            }
            "h2" | "http" => {
                let mut transport = json!({
                    "type": "http"
                });

                if let Some(host) = config.host {
                    transport["host"] = json!(host);
                }

                if let Some(path) = config.path {
                    transport["path"] = json!(path);
                }

                node["transport"] = transport;
            }
            "grpc" => {
                node["transport"] = json!({
                    "type": "grpc",
                    "service_name": config.path.unwrap_or_default()
                });
            }
            _ => {}
        }
    }

    Ok(node)
}
