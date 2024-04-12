use crate::{intent::Intent, price_oracle::PriceOracle};
use num_bigint::BigUint;
use num_traits::ops::checked::CheckedDiv;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchSolution {
    batch_matches: Vec<Match>,
    total_liquidity: BigUint,
}

impl BatchSolution {
    pub fn new(batch_matches: Vec<Match>, price_oracle: impl PriceOracle) -> Self {
        // TODO: review this formula.
        //
        // 1. Notice we need to be invariant on the order of the proposed tokens.
        // That means, we need to get some consensus on which tokens to use as quote token.
        //
        // 2. Since we are denominating the volume in ETH, that might be actually be
        // already invariant, after denominating everything in ETH.
        let total_liquidity = batch_matches
            .iter()
            .map(|m| {
                m.swapped_amount.token_b_amount.clone()
                    * price_oracle.get_current_price(m.intent_b.inputs.quote_token)
            })
            .sum();
        Self {
            batch_matches,
            total_liquidity,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Match {
    pub(crate) intent_a: Intent,
    pub(crate) intent_b: Intent,
    pub(crate) swapped_amount: SwappedAmount,
}

impl Match {
    pub fn new(intent_a: Intent, intent_b: Intent, swapped_amount: SwappedAmount) -> Self {
        Self {
            intent_a,
            intent_b,
            swapped_amount,
        }
    }

    pub fn intent_a(&self) -> &Intent {
        &self.intent_a
    }

    pub fn intent_b(&self) -> &Intent {
        &self.intent_b
    }

    pub fn swapped_amount(&self) -> &SwappedAmount {
        &self.swapped_amount
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
    pub fn swapped_price(&self) -> Option<BigUint> {
        self.token_b_amount.checked_div(&self.token_a_amount)
    }

    pub fn token_a_amount(&self) -> &BigUint {
        &self.token_a_amount
    }

    pub fn token_b_amount(&self) -> &BigUint {
        &self.token_b_amount
    }
}
