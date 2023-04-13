use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct ProvedResult {
    method_id: Vec<u8>,
    outputs: String,
    receipt: String,
}

#[wasm_bindgen]
pub fn verify(serialized_result: String) -> bool {
    let result: ProvedResult = match serde_json::from_str(&serialized_result) {
        Ok(r) => r,
        Err(e) => {
            alert(&format!("Failed to decerialize: {}", e));
            return false;
        }
    };

    let receipt: Receipt = match serde_json::from_str(&result.receipt) {
        Ok(r) => r,
        Err(e) => {
            alert(&format!("Failed to decerialize: {}", e));
            return false;
        }
    };

    match receipt.verify(result.method_id.as_slice()) {
        Ok(_) => true,
        Err(_e) => false,
    }
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}
