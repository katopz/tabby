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
const API_URL: &str = "http://192.168.1.33:9090/v1/health";

pub async fn fetch_tabby<T: DeserializeOwned>(url: &str) -> Result<T, TabbyApiError> {
  let response = reqwest::get(url).await.map_err(TabbyApiError::RequestError)?;
  let json = response.json::<T>().await.map_err(TabbyApiError::JsonParseError)?;
  Ok(json)
}

pub async fn stream_tabby<F>(url: &str, callback: F) -> Result<(), TabbyApiError>
where
  F: FnOnce(String) + std::marker::Copy,
{
  let response = reqwest::get(url).await.map_err(TabbyApiError::RequestError)?;
  let mut stream = response.bytes_stream();

  while let Some(item) = stream.next().await {
    let chunk = item.map_err(TabbyApiError::StreamError)?;
    callback(format!("Chunk: {:?}", chunk));
  }

  Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TabbyClientViewData {
  pub health_state: Option<HealthState>,
}

pub async fn fetch_health_view_data() -> TabbyClientViewData {
  match fetch_tabby::<HealthState>(API_URL).await {
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
  F: FnOnce(String) + std::marker::Copy,
{
  match stream_tabby("http://httpbin.org/ip", callback).await {
    Ok(text) => TabbyChatViewData { role: ChatRole::Tabby, text: Some(format!("{:?}", text)) },
    Err(_) => TabbyChatViewData { role: ChatRole::Tabby, text: None },
  };
}
