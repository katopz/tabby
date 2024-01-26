use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::EnumString;
use strum_macros::Display;
use tabby::services::health::HealthState;

#[derive(Debug, PartialEq, PartialOrd, EnumString, Serialize, Deserialize, Clone, Eq, Ord, Display)]
pub enum ChatRole {
  #[strum(serialize = "user")]
  User,

  #[strum(serialize = "assistant")]
  Assistant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TabbyChatViewData {
  pub role: ChatRole,
  pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TabbyClientViewData {
  pub health_state: Option<HealthState>,
}
