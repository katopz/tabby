use crate::core::error::TabbyApiError;
use futures::StreamExt;
use reqwest::{self, Request};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::EnumString;
use tabby::serve::HealthState;
use thiserror::Error;

#[derive(Clone)]
pub struct HttpProvider {
  client: reqwest::Client,
  headers: reqwest::header::HeaderMap,
  url: String,
}

impl HttpProvider {
  pub fn new(url: &str) -> Self {
    Self { client: reqwest::Client::new(), headers: reqwest::header::HeaderMap::new(), url: url.to_owned() }
  }
}

impl HttpProvider {
  pub async fn get<T: DeserializeOwned>(&self, route_str: &str) -> Result<T, TabbyApiError> {
    let client = &self.client;
    let url = format!("{}{}", self.url.clone(), route_str);
    let headers = self.headers.clone();

    let response = reqwest::get(url).await.map_err(TabbyApiError::RequestError)?;
    let json = response.json::<T>().await.map_err(TabbyApiError::JsonParseError)?;
    Ok(json)
  }

  // pub async fn post<T: DeserializeOwned>(&self, request: &Request) -> Result<T, TabbyApiError> {
  //   let client = &self.client;
  //   let url = self.url.clone();
  //   let headers = self.headers.clone();

  //   let response = reqwest::Client::new().post(url).await.map_err(TabbyApiError::RequestError)?;
  //   let json = response.json::<T>().await.map_err(TabbyApiError::JsonParseError)?;
  //   Ok(json)
  // }

  pub async fn stream<F>(&self, route_str: &str, callback: F) -> Result<(), TabbyApiError>
  where
    F: Fn(String),
  {
    let client = &self.client;
    let url = format!("{}{}", self.url.clone(), route_str);
    let headers = self.headers.clone();

    let response = reqwest::get(url).await.map_err(TabbyApiError::RequestError)?;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
      let chunk_bytes = item.map_err(TabbyApiError::StreamError)?;
      let chunk_str = std::str::from_utf8(&chunk_bytes).expect("Invalid UTF-8");
      callback(format!("Chunk: {:?}", chunk_str.to_string()));
    }

    Ok(())
  }
}

#[derive(Clone)]
pub enum Provider {
  Http(HttpProvider),
}

impl Provider {
  pub fn new(url: &str) -> Self {
    Self::Http(HttpProvider::new(url))
  }
}
