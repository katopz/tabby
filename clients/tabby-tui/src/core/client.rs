use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use tabby::services::health::HealthState;
use tabby_common::api::event::Message;

use super::chat::{ChatRole, TabbyChatViewData, TabbyClientViewData};
use crate::core::provider::Provider;

#[derive(EnumString, Display, Debug)]
pub enum EndPoint {
  #[strum(serialize = "/v1")]
  V1,
  #[strum(serialize = "/v1beta")]
  V1Beta,
  #[strum(disabled)]
  CustomUrl(String),
}

#[derive(EnumString, Display, Debug)]
pub enum Route {
  #[strum(serialize = "/health")]
  Health,
  #[strum(serialize = "/chat/completions")]
  ChatCompletions,
}

#[derive(Clone)]
pub struct TabbyClient {
  provider: Provider,
}

impl Default for TabbyClient {
  fn default() -> Self {
    let endpoint_url_string = EndPoint::V1.to_string();
    Self { provider: Provider::new(&endpoint_url_string) }
  }
}

impl TabbyClient {
  pub fn new(api_url: &str, endpoint: &EndPoint) -> Self {
    let endpoint_url_string = match endpoint {
      EndPoint::CustomUrl(url) => url.to_string(),
      _ => format!("{}{}", api_url, endpoint.to_string()),
    };

    Self { provider: Provider::new(&endpoint_url_string) }
  }

  pub async fn get_health(&self) -> TabbyClientViewData {
    let Provider::Http(provider) = &self.provider;

    match provider.get::<HealthState>(&Route::Health.to_string()).await {
      Ok(health_state) => TabbyClientViewData { health_state: Some(health_state) },
      // FIXME: Forward error
      Err(_) => TabbyClientViewData { health_state: None },
    }
  }

  pub async fn get_chat_completions<F>(&self, id: &str, messages: &Vec<Message>, callback: F) -> TabbyChatViewData
  where
    F: Fn(String),
  {
    let Provider::Http(provider) = &self.provider;

    match provider.stream(&Route::ChatCompletions.to_string(), id, messages, callback).await {
      Ok(text) => TabbyChatViewData { role: ChatRole::Assistant, text: format!("{:?}", text) },
      Err(err) => TabbyChatViewData { role: ChatRole::Assistant, text: format!("{err:?}") },
    }
  }
}
