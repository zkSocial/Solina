use reqwest::Url;
use serde_json::{json, Value};
use solina_client::intent_client::IntentClient;
use std::io::stdin;
use tokio;

#[tokio::main]
async fn main() {
    let endpoint = Url::parse("http://localhost:8081").expect("Invalid URL");

    let mut intent_client =
        IntentClient::connect(endpoint).expect("Failed to connect client to endpoint");
    let mut input = String::new();

    while let Ok(n) = stdin().read_line(&mut input) {
        if n == 0 {
            break;
        }

        let message = input.trim();

        if input == "exit" {
            break;
        }

        intent_client
            .send_request::<&str, String>(message)
            .await
            .expect("Failed to send request successfully");
        input.clear();
    }
}
