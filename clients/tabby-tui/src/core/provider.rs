use crate::core::error::TabbyApiError;
use futures::StreamExt;
use reqwest::{self, header::HeaderMap, Client, Request, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::EnumString;
use tabby::serve::HealthState;
use thiserror::Error;

#[derive(Clone)]
pub struct HttpProvider {
  client: Client,
  headers: HeaderMap,
  url: String,
}

impl HttpProvider {
  pub fn new(url: &str) -> Self {
    Self { client: Client::new(), headers: HeaderMap::new(), url: url.to_owned() }
  }

  pub fn with_client(client: Client, url: String) -> Self {
    let headers = HeaderMap::new();
    HttpProvider { client, url, headers }
  }

  pub fn with_headers(client: Client, url: String, headers: HeaderMap) -> Self {
    let headers = HeaderMap::new();
    HttpProvider { client, url, headers }
  }

  pub fn with_client_headers(client: Client, url: String, headers: HeaderMap) -> Self {
    HttpProvider { client, url, headers }
  }

  async fn send_request<T: DeserializeOwned>(
    &self,
    request_builder: RequestBuilder,
    maybe_body: Option<String>,
  ) -> Result<T, TabbyApiError> {
    let client = if let Some(body) = &maybe_body { request_builder.body(body.clone()) } else { request_builder };

    let response = client.send().await.map_err(TabbyApiError::RequestError)?;
    response.json::<T>().await.map_err(TabbyApiError::JsonParseError)
  }

  pub async fn get<T: DeserializeOwned>(&self, route_str: &str) -> Result<T, TabbyApiError> {
    let url = format!("{}{}", self.url, route_str);
    self.send_request(self.client.get(&url), None).await
  }

  pub async fn post<T: DeserializeOwned>(
    &self,
    request: &Request,
    maybe_body: Option<String>,
  ) -> Result<T, TabbyApiError> {
    let url = &self.url;
    let request_builder = self.client.post(url);
    self.send_request(request_builder, maybe_body).await
  }

  pub async fn stream<F>(&self, route_str: &str, callback: F) -> Result<(), TabbyApiError>
  where
    F: Fn(String),
  {
    let url = format!("{}{}", self.url, route_str);
    let response = self.client.get(&url).send().await.map_err(TabbyApiError::RequestError)?;
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
