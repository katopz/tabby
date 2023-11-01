use futures::StreamExt;
use reqwest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::EnumString;
use tabby::serve::HealthState;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TabbyApiError {
  #[error("Request failed: {0}")]
  RequestError(reqwest::Error),

  #[error("JSON parsing error: {0}")]
  JsonParseError(reqwest::Error),

  #[error("Streaming error: {0}")]
  StreamError(reqwest::Error),
}

// TODO: Use env
const HEALTH_API_URL: &str = "http://192.168.1.33:9090/v1/health";

// TODO: Use /v1beta/chat/completions
const CHAT_API_URL: &str = "http://192.168.1.33:9090/v1/health";

pub async fn fetch_tabby<T: DeserializeOwned>(url: &str) -> Result<T, TabbyApiError> {
  let response = reqwest::get(url).await.map_err(TabbyApiError::RequestError)?;
  let json = response.json::<T>().await.map_err(TabbyApiError::JsonParseError)?;
  Ok(json)
}

pub async fn stream_tabby<F>(url: &str, callback: F) -> Result<(), TabbyApiError>
where
  F: Fn(String),
{
  let response = reqwest::get(url).await.map_err(TabbyApiError::RequestError)?;
  let mut stream = response.bytes_stream();

  while let Some(item) = stream.next().await {
    let chunk_bytes = item.map_err(TabbyApiError::StreamError)?;
    let chunk_str = std::str::from_utf8(&chunk_bytes).expect("Invalid UTF-8");
    callback(format!("Chunk: {:?}", chunk_str.to_string()));
  }

  Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TabbyClientViewData {
  pub health_state: Option<HealthState>,
}

pub async fn fetch_health_view_data() -> TabbyClientViewData {
  match fetch_tabby::<HealthState>(HEALTH_API_URL).await {
    Ok(health_state) => TabbyClientViewData { health_state: Some(health_state) },
    Err(_) => TabbyClientViewData { health_state: None },
  }
}

#[derive(Debug, PartialEq, PartialOrd, EnumString, Serialize, Deserialize, Clone, Eq, Ord)]

pub enum ChatRole {
  #[strum(serialize = "user")]
  User,

  #[strum(serialize = "tabby")]
  Tabby,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TabbyChatViewData {
  pub role: ChatRole,
  pub text: Option<String>,
}

pub async fn fetch_chat_view_data<F>(callback: F)
where
  F: Fn(String),
{
  match stream_tabby(CHAT_API_URL, callback).await {
    Ok(text) => TabbyChatViewData { role: ChatRole::Tabby, text: Some(format!("{:?}", text)) },
    Err(_) => TabbyChatViewData { role: ChatRole::Tabby, text: None },
  };
}
