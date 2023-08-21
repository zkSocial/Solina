use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use solina::structured_hash::StructuredHashInterface;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub(crate) enum TradeDirection {
    Buy,
    Sell,
}

/// Inputs for a swap
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SwapInputs {
    /// address
    from: BigUint,
    /// quote token
    quote_token: BigUint,
    /// base token
    base_token: BigUint,
    /// quote amount
    quote_amount: BigUint,
    /// trade direction
    direction: SwapDirection,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Intent {}
