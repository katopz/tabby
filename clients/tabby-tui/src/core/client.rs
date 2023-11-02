use crate::core::provider::Provider;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use tabby::HealthState;

use super::chat::{ChatRole, TabbyChatViewData, TabbyClientViewData};

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
      _ => endpoint.to_string(),
    };

    Self { provider: Provider::new(&endpoint_url_string) }
  }

  pub async fn get_health(&self) -> TabbyClientViewData {
    let Provider::Http(provider) = &self.provider;

    match provider.get::<HealthState>(&Route::Health.to_string()).await {
      Ok(health_state) => TabbyClientViewData { health_state: Some(health_state) },
      Err(_) => TabbyClientViewData { health_state: None },
    }
  }

  pub async fn get_chat_completions<F>(&self, callback: F)
  where
    F: Fn(String),
  {
    let Provider::Http(provider) = &self.provider;

    match provider.stream(&Route::Health.to_string(), callback).await {
      Ok(text) => TabbyChatViewData { role: ChatRole::Tabby, text: Some(format!("{:?}", text)) },
      Err(_) => TabbyChatViewData { role: ChatRole::Tabby, text: None },
    };
  }
}
