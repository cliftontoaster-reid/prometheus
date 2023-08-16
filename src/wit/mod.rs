use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urlencoding::encode;

pub struct Client {
  pub token: String,
}

impl Client {
  pub async fn message(
    &self,
    msg: String,
    dynamic_entity: Option<DynamicEntitiesList>,
  ) -> Result<Message, Error> {
    let cl = ReqwestClient::new();
    let entity_string = if dynamic_entity.is_some() {
      encode(&serde_json::to_string(&dynamic_entity).unwrap_or("".to_string())).to_string()
    } else {
      "".to_string()
    };

    let url = format!(
      "https://api.wit.ai/message?q={}{}",
      encode(&msg),
      entity_string
    );
    cl.get(url)
      .header("Authorization", format!("Bearer {}", self.token))
      .send()
      .await
      .unwrap()
      .json()
      .await
      .unwrap()
  }
}

#[derive(Debug, Deserialize)]
pub struct Error {
  pub error: String,
  pub code: String,
}
#[derive(Debug, Deserialize)]
pub struct Message {
  pub text: String,
  pub intents: Vec<Intent>,
  pub entities: HashMap<String, Vec<Entity>>,
}
#[derive(Debug, Deserialize)]
pub struct Intent {
  pub id: String,
  pub name: String,
  pub confidence: f32,
}
#[derive(Debug, Deserialize)]
pub struct Entity {
  pub id: String,
  pub name: String,
  pub role: String,
  pub start: u16,
  pub end: u16,
  pub body: String,
  pub confidence: f32,
  #[serde(rename = "type")]
  pub value_type: String,

  pub value: Option<String>,
  pub values: Option<Vec<Values>>,
}
#[derive(Debug, Serialize)]
pub struct DynamicEntitiesList {
  pub entities: HashMap<String, Vec<DynamicEntitiesEntry>>,
}
#[derive(Debug, Serialize)]
pub struct DynamicEntitiesEntry {
  pub keyword: String,
  pub synonyms: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub enum Values {
  Interval(Interval),
}
#[derive(Debug, Deserialize)]
pub struct Interval {
  #[serde(rename = "type")]
  pub value_type: String,
  pub from: GrainnedValue,
  pub to: GrainnedValue,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GrainnedValue {
  pub grain: String,
  pub value: String,
}
