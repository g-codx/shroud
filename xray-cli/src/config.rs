use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct XrayConfig {
    pub log: LogSettings,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
}

#[derive(Serialize, Deserialize)]
pub struct LogSettings {
    pub loglevel: String,
}

#[derive(Serialize, Deserialize)]
pub struct Inbound {
    pub port: u16,
    pub protocol: String,
    pub settings: InboundSettings,
}

#[derive(Serialize, Deserialize)]
pub struct InboundSettings {
    pub auth: String,
    pub udp: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Outbound {
    pub protocol: String,
    pub settings: OutboundSettings,
    #[serde(rename = "streamSettings")]
    pub stream_settings: Option<StreamSettings>,
}

#[derive(Serialize, Deserialize)]
pub struct OutboundSettings {
    pub vnext: Vec<VNext>,
}

#[derive(Serialize, Deserialize)]
pub struct VNext {
    pub address: String,
    pub port: u16,
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub encryption: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StreamSettings {
    #[serde(rename = "network")]
    pub network: String,
    #[serde(rename = "security")]
    pub security: String,
    #[serde(rename = "realitySettings")]
    pub reality_settings: Option<RealitySettings>,
}

#[derive(Serialize, Deserialize)]
pub struct RealitySettings {
    #[serde(rename = "serverName")]
    pub server_name: String,
    #[serde(rename = "fingerprint")]
    pub fingerprint: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "shortId")]
    pub short_id: String,
    #[serde(rename = "spiderX")]
    pub spider_x: String,
}

pub fn parse_vless_url(url: &str) -> Result<String, String> {
    let parsed = Url::parse(url).map_err(|e| e.to_string())?;
    let uuid = parsed.username();
    let uuid = Uuid::parse_str(uuid).map_err(|e| e.to_string())?;

    let host = parsed.host_str().ok_or("No host provided")?;
    let port = parsed.port().unwrap_or(443);

    let query_pairs: Vec<_> = parsed.query_pairs().collect();

    let encryption = query_pairs.iter()
        .find(|(k, _)| k == "encryption")
        .map(|(_, v)| v.to_string())
        .unwrap_or("none".into());

    let flow = query_pairs.iter()
        .find(|(k, _)| k == "flow")
        .map(|(_, v)| Some(v.to_string()))
        .unwrap_or(None);

    let transport_type = query_pairs.iter()
        .find(|(k, _)| k == "type")
        .map(|(_, v)| v.to_string())
        .unwrap_or("tcp".into());

    let security = query_pairs.iter()
        .find(|(k, _)| k == "security")
        .map(|(_, v)| v.to_string())
        .unwrap_or("none".into());

    let reality_settings = if security == "reality" {
        let pbk = query_pairs.iter()
            .find(|(k, _)| k == "pbk")
            .map(|(_, v)| v.to_string())
            .ok_or("Missing pbk for reality")?;

        let fp = query_pairs.iter()
            .find(|(k, _)| k == "fp")
            .map(|(_, v)| v.to_string())
            .unwrap_or("chrome".to_string());

        let sni = query_pairs.iter()
            .find(|(k, _)| k == "sni")
            .map(|(_, v)| v.to_string())
            .ok_or("Missing sni for reality")?;

        let sid = query_pairs.iter()
            .find(|(k, _)| k == "sid")
            .map(|(_, v)| v.to_string())
            .unwrap_or("df".to_string());

        let spx = query_pairs.iter()
            .find(|(k, _)| k == "spx")
            .map(|(_, v)| v.to_string())
            .unwrap_or("/".to_string());

        Some(RealitySettings {
            server_name: sni,
            fingerprint: fp,
            public_key: pbk,
            short_id: sid,
            spider_x: spx,
        })
    } else {
        None
    };

    let config = XrayConfig {
        log: LogSettings {
            loglevel: "warning".to_string(),
        },
        inbounds: vec![Inbound {
            port: 1080,
            protocol: "socks".to_string(),
            settings: InboundSettings {
                auth: "noauth".to_string(),
                udp: true,
            },
        }],
        outbounds: vec![Outbound {
            protocol: "vless".to_string(),
            settings: OutboundSettings {
                vnext: vec![VNext {
                    address: host.to_string(),
                    port,
                    users: vec![User {
                        id: uuid.to_string(),
                        encryption,
                        flow,
                    }],
                }],
            },
            stream_settings: Some(StreamSettings {
                network: transport_type,
                security,
                reality_settings,
            }),
        }],
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    Ok(json)
}