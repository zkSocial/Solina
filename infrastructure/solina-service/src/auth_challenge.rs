use axum::Json;
use ethers::prelude::*;
use ethers::utils::keccak256;
use log::{error, info};
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
    let signature = body.get("signature").ok_or(Error::InvalidRequest)?;
    let signature = signature
        .as_str()
        .expect("Failed to extract signature from request");
    let public_key = body.get("public_key").ok_or(Error::InvalidRequest)?;
    let public_key = public_key
        .as_str()
        .expect("Failed to extract public key from request");

    // TODO: to be refactored (we will need the worker as state). We are currently
    // querying the challenge store via the public key, we should have an id as well
    // to query the latest available challenge for the given public key
    let challenges = CHALLENGES
        .lock()
        .expect("Failed to acquire challenges lock");
    let challenge = challenges.get(public_key).cloned().unwrap_or_default();
    let challenge = "vhbo85kmcqGMjATMiktMPbweQN8q7k59".to_string();

    info!("The challenge is: {}", challenge);

    let address: Address = Address::from_str(&public_key).expect(&format!(
        "Failed to extract Address from public key, {}",
        public_key
    ));
    info!("The address is: {:?}", address);
    info!("The signature is: {:?}", Signature::from_str(&signature));

    let recovered_address = match Signature::from_str(&signature) {
        Ok(sig) => sig.verify(challenge, address).map_err(|e| {
            error!(
                "Failed to recover user address from signature and message, with error: {}",
                e
            );
            Error::AuthError
        })?,
        Err(e) => {
            error!("Failed to obtain signature from request, with error: {}", e);
            return Err(Error::AuthError);
        }
    };

    // Send the response back to the user
    Ok(Json(json!({"status": "successfully verified"})))
}
