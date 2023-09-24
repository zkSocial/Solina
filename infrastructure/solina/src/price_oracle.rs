// TODO: eventually, we might be able to compute volumes and total liquidity over intent batches
// from the intent data alone. Currently, to simplify the logic, we assume we have access to an API
// to query current prices of tokens, denominated say in ETH
use num_bigint::BigUint;

use crate::TokenAddress;

pub type Price = BigUint;

pub trait PriceOracle {
    fn get_current_price(&self, token_address: TokenAddress) -> Price;
}
