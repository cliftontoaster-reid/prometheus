pub mod discord;

use reqwest::Client as ReqwestCLient;
use serde::{Deserialize, Serialize};
use urlencoding::encode;
use serde_json::from_str;
use toml::to_string_pretty;

pub async fn check_url(urls: Vec<String>, key: String) -> Response {
  let cl = ReqwestCLient::new();
  let mut entries: Vec<ThreatEntry> = Vec::new();
  for url in urls {
    entries.push(ThreatEntry { url: url })
  }

  let data = Request {
    client: ClientInfo {
      client_id: "prometheus".to_string(),
      client_version: env!("CARGO_PKG_VERSION").to_string(),
    },
    threat_info: ThreatInfo {
      threat_types: vec![
        "MALWARE".to_string(),
        "SOCIAL_ENGINEERING".to_string(),
        "UNWANTED_SOFTWARE".to_string(),
        "POTENTIALLY_HARMFUL_APPLICATION".to_string(),
      ],
      platform_types: vec!["ANY_PLATFORM".to_string()],
      threat_entry_types: vec!["URL".to_string()],
      threat_entries: entries,
    },
  };
  println!("{}", to_string_pretty(&data).unwrap());
  let res = cl.post(format!(
    "https://safebrowsing.googleapis.com/v4/threatMatches:find?key={}",
    encode(&key)
  ))
  .header("Content-Type", "application/json")
  .json(&data)
  .send()
  .await
  .unwrap().text().await.unwrap();
  println!("{}", &res);
  if res == "{}" {
    Response::default()
  } else {
    from_str(&res).unwrap()
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
  pub client: ClientInfo,
  #[serde(rename = "threatInfo")]
  pub threat_info: ThreatInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInfo {
  #[serde(rename = "clientId")]
  pub client_id: String,
  #[serde(rename = "clientVersion")]
  pub client_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatInfo {
  #[serde(rename = "threatTypes")]
  pub threat_types: Vec<String>,
  #[serde(rename = "platformTypes")]
  pub platform_types: Vec<String>,
  #[serde(rename = "threatEntryTypes")]
  pub threat_entry_types: Vec<String>,
  #[serde(rename = "threatEntries")]
  pub threat_entries: Vec<ThreatEntry>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ThreatEntry {
  pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Response {
  pub matches: Vec<Match>,
}

impl Response {
  pub fn is_malicious(&self) -> bool {
    !self.matches.is_empty()
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Match {
  #[serde(rename = "threatType")]
  pub threat_type: String,
  #[serde(rename = "platformType")]
  pub platform_type: String,
  #[serde(rename = "threatEntryType")]
  pub threat_entry_type: String,
  pub threat: ThreatEntry,
  #[serde(rename = "threatEntryMetadata")]
  pub threat_entry_metadata: Option<ThreatEntryMetadata>,
  #[serde(rename = "cacheDuration")]
  pub cache_duration: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ThreatEntryMetadata {
  pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Entry {
  pub key: String,
  pub value: String,
}
