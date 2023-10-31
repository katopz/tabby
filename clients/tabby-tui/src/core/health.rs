use reqwest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tabby::serve::HealthState;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TabbyApiError {
  #[error("Request failed: {0}")]
  RequestError(reqwest::Error),

  #[error("JSON parsing error: {0}")]
  JsonParseError(reqwest::Error),
}

// TODO: Use env
const API_URL: &str = "http://192.168.1.33:9090/v1/health";

pub async fn fetch_tabby<T: DeserializeOwned>(url: &str) -> Result<T, TabbyApiError> {
  let response = reqwest::get(url).await.map_err(TabbyApiError::RequestError)?;
  let json = response.json::<T>().await.map_err(TabbyApiError::JsonParseError)?;
  Ok(json)
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TabbyChatViewData {
  pub text: Option<String>,
}

pub async fn fetch_chat_view_data() -> TabbyChatViewData {
  TabbyChatViewData { text: Some("hi!".to_string()) }
  // match fetch_tabby::<HealthState>(API_URL).await {
  //   Ok(text) => TabbyChatViewData { text: Some(text) },
  //   Err(_) => TabbyChatViewData { text: None },
  // }
}
