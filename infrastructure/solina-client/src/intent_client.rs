use anyhow::anyhow;
use reqwest::{header, header::HeaderMap, IntoUrl, Url};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug)]
/// An intent client whose purpose is to send http requests to the server.
/// The request consists of the intent data, that the server will store.
pub struct IntentClient {
    client: reqwest::Client,
    endpoint: Url,
    request_id: i64,
}

impl IntentClient {
    pub fn connect<T: IntoUrl>(endpoint: T) -> Result<Self, anyhow::Error> {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(header::CONTENT_TYPE, "applications/json".parse().unwrap());
                headers
            })
            .build()?;

        Ok(Self {
            client,
            endpoint: endpoint.into_url()?,
            request_id: 0,
        })
    }

    fn next_request_id(&mut self) -> i64 {
        self.request_id += 1;
        self.request_id
    }

    pub async fn send_request<T: Serialize, R: DeserializeOwned>(
        &mut self,
        data: T,
    ) -> Result<R, anyhow::Error> {
        let intent_data = serde_json::to_value(data)
            .map_err(|e| anyhow!("Failed to serialize data to JSON, with error: {}", e))?;
        let request_json = json!(
            {
                "jsonrpc": "2.0",
                "id": self.next_request_id(),
                "value": intent_data
            }
        );
        let response = self
            .client
            .post(self.endpoint.clone())
            .body(request_json.to_string())
            .send()
            .await?;
        let value: Value = response.json().await?;
        let json_value = jsonrpc_value(value)?;
        match serde_json::from_value(json_value) {
            Ok(r) => Ok(r),
            Err(e) => Err(anyhow!("Failed to deserialize response, with error: {}", e)),
        }
    }
}

fn jsonrpc_value(value: Value) -> Result<Value, anyhow::Error> {
    if let Some(error) = value.get("error") {
        let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(anyhow!(
            "Request Failed with status: code = {}, message = {}",
            code,
            message.to_string()
        ));
    }

    let result = value
        .get("result")
        .ok_or_else(|| anyhow!("Missing result field"))?;
    Ok(result.clone())
}
