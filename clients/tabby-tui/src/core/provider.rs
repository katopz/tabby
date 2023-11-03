use crate::core::error::TabbyApiError;
use futures::StreamExt;
use reqwest::{self, header::HeaderMap, Client, Request, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use strum::EnumString;
use tabby::{
  chat::{ChatCompletionChunk, Message},
  serve::health::HealthState,
};
use thiserror::Error;
use uuid::Uuid;

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

  pub async fn stream<F>(
    &self,
    path: &str,
    id: &str,
    messages: &Vec<Message>,
    callback: F,
  ) -> Result<String, TabbyApiError>
  where
    F: Fn(String),
  {
    let url = get_api_url(&self.url, path);
    let json_messages: Vec<serde_json::Value> = messages
      .iter()
      .filter(|message| message.content.len() > 0)
      .map(|message| {
        serde_json::json!({
            "role": message.role.clone(),
            "content": message.content.clone(),
        })
      })
      .collect();

    let json_data = serde_json::json!({
        "messages": json_messages,
        "id": id,
    });

    let json_data_str: String = json_data.to_string();

    let request_builder = self.client.post(url).header("Content-Type", "application/json").json(&json_data);

    // Stream the response body as bytes
    let response = request_builder.send().await.map_err(TabbyApiError::RequestError)?;
    let mut body = response.bytes_stream();

    let mut combined_chunk_str = "".to_string();
    while let Some(chunk) = body.next().await {
      let chunk_bytes = chunk.map_err(TabbyApiError::StreamError)?;
      let chunk_text = std::str::from_utf8(&chunk_bytes).map_err(TabbyApiError::StreamUtf8Error)?;
      let chat_completion_chunk =
        serde_json::from_str::<ChatCompletionChunk>(&chunk_text).map_err(TabbyApiError::StreamJsonError)?;

      callback(chat_completion_chunk.content.clone());

      combined_chunk_str.push_str(&chat_completion_chunk.content.clone());
    }

    Ok(combined_chunk_str)
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
