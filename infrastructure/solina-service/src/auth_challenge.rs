use axum::Json;
use ethers::prelude::*;
use ethers::utils::keccak256;
use log::{error, info};
use rand::Rng;
use serde_json::{json, Value};
use std::{collections::HashMap, str::FromStr, sync::Mutex};

use crate::{
    error::{Error, Result},
    json_rpc_server::AppState,
};

pub(crate) fn generate_challenge() -> String {
    let challenge: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    challenge
}

pub(crate) fn extract_address(value: &Value) -> Result<String> {
    let address = value.get("address").ok_or(Error::InvalidRequest)?;
    let address = address.as_str().ok_or({
        error!("Failed to extract address from request, with error");
        Error::InvalidRequest
    })?;
    Ok(address.to_string())
}

pub(crate) fn extract_signature(value: &Value) -> Result<String> {
    let signature = value.get("signature").ok_or(Error::InvalidRequest)?;
    let signature = signature.as_str().ok_or({
        error!("Failed to extract signature from request");
        Error::InvalidRequest
    })?;
    Ok(signature.to_string())
}

pub(crate) fn verify_signature(
    address: String,
    challenge: String,
    signature: String,
) -> Result<Json<Value>> {
    info!("The challenge is: {}", challenge);

    let address: Address = Address::from_str(&address).expect(&format!(
        "Failed to extract Address from public key, {}",
        address
    ));
    info!("The address is: {:?}", address);

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
