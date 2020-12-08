use reqwest::{Client, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{event, Level};

pub mod methods;
pub mod types;

fn deserialize_opt_usize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let value = isize::deserialize(deserializer)?;
    Ok(if value < 0 {
        None
    } else {
        Some(value as usize)
    })
}

#[derive(Debug)]
enum KodiError {
    Jsonrpc {
        method: String,
        id: usize,
        code: i64,
        message: String,
    },
    Deserialize {
        method: String,
        payload: String,
        error: serde_json::error::Error,
    },
}

impl std::fmt::Display for KodiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for KodiError {}

pub trait KodiMethod: std::fmt::Debug + Serialize {
    const NAME: &'static str;
    type Response: DeserializeOwned;
}

#[derive(Debug, Serialize)]
struct KodiRequest<M: KodiMethod> {
    jsonrpc: &'static str,
    #[serde(rename = "method")]
    name: &'static str,
    params: M,
    id: usize,
}

impl<M: KodiMethod> KodiRequest<M> {
    fn new(method: M, id: usize) -> Self {
        Self {
            jsonrpc: "2.0",
            name: M::NAME,
            params: method,
            id,
        }
    }

    async fn send(
        self,
        client: &Client,
        url: Url,
    ) -> Result<M::Response, Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_string(&self)?;
        event!(Level::DEBUG, "-> {body}", body = body);
        let text = client.post(url).body(body).send().await?.text().await?;
        event!(Level::DEBUG, "<- {text}", text = text);
        let resp: KodiResponse<M::Response> =
            serde_json::from_str(&text).map_err(|error| KodiError::Deserialize {
                method: format!("{:?}", self),
                error,
                payload: text,
            })?;
        match resp.kind {
            KodiResponseKind::Result(result) => Ok(result),
            KodiResponseKind::Error { code, message } => Err(KodiError::Jsonrpc {
                method: format!("{:?}", self),
                id: resp.id,
                code,
                message,
            })?,
        }
    }
}

#[derive(Debug, Deserialize)]
enum KodiResponseKind<T> {
    #[serde(rename = "result")]
    Result(T),
    #[serde(rename = "error")]
    Error { code: i64, message: String },
}

#[derive(Debug, Deserialize)]
struct KodiResponse<T> {
    #[serde(flatten)]
    kind: KodiResponseKind<T>,
    id: usize,
}

pub struct KodiClient {
    url: Url,
    client: Client,
    next_id: AtomicUsize,
}

impl KodiClient {
    pub fn new(client: Client, url: Url) -> Self {
        Self {
            url,
            client,
            next_id: AtomicUsize::new(0),
        }
    }

    pub async fn send_method<M: KodiMethod>(
        &self,
        method: M,
    ) -> Result<M::Response, Box<dyn std::error::Error + Send + Sync>> {
        KodiRequest::new(method, self.next_id.fetch_add(1, Ordering::Relaxed))
            .send(&self.client, self.url.clone())
            .await
    }
}
