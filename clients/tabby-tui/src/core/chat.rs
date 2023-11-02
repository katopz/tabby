use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::EnumString;
use tabby::serve::HealthState;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TabbyClientViewData {
  pub health_state: Option<HealthState>,
}
