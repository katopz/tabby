use crate::core::error::TabbyApiError;
use futures::StreamExt;
use reqwest::{self, header::HeaderMap, Client, Request, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::EnumString;
use tabby::{chat::Message, serve::health::HealthState};
use thiserror::Error;

#[derive(Clone)]
pub struct HttpProvider {
  client: Client,
  headers: HeaderMap,
  url: String,
}

fn get_api_url(url: &str, path: &str) -> String {
  let mut url = url.to_owned();
  url.push_str(path);
  url
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

  pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, TabbyApiError> {
    let url = get_api_url(&self.url, path);
    let request_builder = self.client.get(url);

    self.send_request::<T>(request_builder, None).await
  }

  pub async fn post<T: DeserializeOwned>(&self, path: &str, maybe_body: Option<String>) -> Result<T, TabbyApiError> {
    let url = get_api_url(&self.url, path);
    let request_builder = self.client.post(url);

    self.send_request::<T>(request_builder, maybe_body).await
  }

  pub async fn stream<F>(&self, path: &str, messages: &Vec<Message>, callback: F) -> Result<(), TabbyApiError>
  where
    F: Fn(String),
  {
    let url = get_api_url(&self.url, path);
    let request_builder = self.client.post(url).json(messages);

    let response = request_builder.send().await.map_err(TabbyApiError::RequestError)?;
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
