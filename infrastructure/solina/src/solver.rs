use crate::price_oracle::PriceOracle;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchSolution {
    batch_matches: Vec<Match>,
    total_liquidity: BigUint,
}

impl BatchSolution {
    pub fn new(batch_matches: Vec<Match>, price_oracle: PriceOracle) -> Self {
        // TODO: review this formula.
        //
        // 1. Notice we need to be invariant on the order of the proposed tokens.
        // That means, we need to get some consensus on which tokens to use as quote token.
        //
        // 2. Since we are denominating the volume in ETH, that might be actually be
        // already invariant, after denominating everything in ETH.
        let total_liquidity = batch_matches.iter().sum(|m| {
            m.swapped_amount.token_b_amount * price_oracle.get_current_price(m.intent_b.quote_token)
        });
        Self {
            batch_matches,
            total_liquidity,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Match {
    intent_a: Intent,
    intent_b: Intent,
    swapped_amount: SwappedAmount,
}

impl Match {
    pub fn new(intent_a: Intent, intent_b: Intent, swapped_amount: BigUint) -> Self {
        Self {
            intent_a,
            intent_b,
            swapped_amount,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SwappedAmount {
    token_a_amount: BigUint,
    token_b_amount: BigUint,
}

impl SwappedAmount {
    pub fn new(token_a_amount: BigUint, token_b_amount: BigUint) -> Self {
        Self {
            token_a_amount,
            token_b_amount,
        }
    }

    // TODO: check this
    pub fn swapped_price(&self) -> BigUint {
        self.token_b_amount.checked_div(self.token_a_amount)
    }
}
