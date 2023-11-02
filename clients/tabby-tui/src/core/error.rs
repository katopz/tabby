use reqwest;
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
