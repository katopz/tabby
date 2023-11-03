use std::str::Utf8Error;

use reqwest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TabbyApiError {
  #[error("Stream utf8 failed: {0}")]
  StreamUtf8Error(Utf8Error),

  #[error("Stream Json failed: {0}")]
  StreamJsonError(serde_json::Error),

  #[error("Request failed: {0}")]
  RequestError(reqwest::Error),

  #[error("JSON parsing error: {0}")]
  JsonParseError(reqwest::Error),

  #[error("Streaming error: {0}")]
  StreamError(reqwest::Error),
}
