use axum::Json;
use ethers::prelude::*;
use ethers::utils::keccak256;
use log::error;
use once_cell::sync::Lazy;
use rand::Rng;
use serde_json::{json, Value};
use std::{collections::HashMap, str::FromStr, sync::Mutex};

use crate::error::{Error, Result};

// TODO: This is a simple in-memory store for the challenges. Add it to the database instead
pub(crate) static CHALLENGES: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub(crate) fn generate_challenge() -> String {
    let challenge: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let mut challenges = CHALLENGES.lock().unwrap();
    challenges.insert("user_challenge".to_string(), challenge.clone());

    challenge
}

pub(crate) fn verify_signature(Json(body): &Json<Value>) -> Result<Json<Value>> {
    let signature = body
        .get("signature")
        .ok_or(Error::InvalidRequest)?
        .to_string();
    let public_key = body
        .get("public_key")
        .ok_or(Error::InvalidRequest)?
        .to_string();

    // TODO: to be refactored (we will need the worker as state). We are currently
    // querying the challenge store via the public key, we should have an id as well
    // to query the latest available challenge for the given public key
    let challenges = CHALLENGES.lock().unwrap();
    let challenge = challenges.get(&public_key).cloned().unwrap_or_default();

    // We prepend the original message, following EIP-191, see https://eips.ethereum.org/EIPS/eip-191.
    let mut message = "\x19Ethereum Signed Message:\n".to_string();

    message.push_str(&format!("{}", challenge.len()));
    message.push_str(&challenge);

    let message_hash = keccak256(message.as_bytes());

    let recovered_address = match Signature::from_str(&signature) {
        Ok(sig) => sig.recover(message_hash).map_err(|e| {
            error!(
                "Failed to recover user address from signature and message, with error: {}",
                e
            );
            Error::AuthError
        })?,
        Err(e) => {
            error!(
                "Failed to recover user address from signature and message, with error: {}",
                e
            );
            return Err(Error::AuthError);
        }
    };

    let address: Address = public_key
        .parse()
        .expect("Failed to parse public key to address");
    if address != recovered_address {
        error!("Failed to recover user address from signature and message");
        return Err(Error::AuthError);
    }

    // Send the response back to the user
    Ok(Json(json!({"status": "successfully verified"})))
}
