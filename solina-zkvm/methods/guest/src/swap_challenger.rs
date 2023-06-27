use std::{
    ops::{Div, Mul},
    str::FromStr,
};

use crate::{swap_intent::SwapIntent, Amount, Price, PublicKey, TokenAddress};
use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::ToBigInt;
use solina::{challenger::ChallengeOrganizer, error::SolinaError};

pub type SwapScore = u64;

#[derive(Clone, Debug)]
pub struct SwapRow {
    pub(crate) address: PublicKey,
    pub(crate) base_token_address: TokenAddress,
    pub(crate) quote_token_address: TokenAddress,
    pub(crate) amount_in: Amount,
    pub(crate) amount_out: Amount,
}

impl SwapRow {
    pub fn new(
        address: PublicKey,
        base_token_address: TokenAddress,
        quote_token_address: TokenAddress,
        amount_in: Amount,
        amount_out: Amount,
    ) -> Self {
        Self {
            address,
            base_token_address,
            quote_token_address,
            amount_in,
            amount_out,
        }
    }

    pub fn get_amount_in(&self) -> Amount {
        self.amount_in.clone()
    }

    pub fn get_amount_out(&self) -> Amount {
        self.amount_out.clone()
    }

    pub fn get_address(&self) -> PublicKey {
        self.address
    }
}

#[derive(Debug, Clone)]
pub struct SwapTable {
    pub(crate) rows: Vec<SwapRow>,
}

impl SwapTable {
    pub fn new(rows: Vec<SwapRow>) -> Self {
        Self { rows }
    }

    pub fn get_rows(&self) -> Vec<&SwapRow> {
        self.rows.iter().collect()
    }

    pub fn get_row_for_address(&self, addr: PublicKey) -> Option<&SwapRow> {
        let rows = self
            .rows
            .iter()
            .filter(|x| x.address == addr)
            .collect::<Vec<_>>();
        rows.first().copied()
    }
}

#[derive(Clone, Debug)]
pub struct SwapSolution {
    pub table: SwapTable,
}

pub struct SwapChallengeOrganizer {
    batch_intents: Vec<SwapIntent>,
    solver_registry: Vec<PublicKey>,
    solutions: Vec<(PublicKey, SwapSolution)>,
    token_prices: Vec<(TokenAddress, TokenAddress, Price)>,
}

impl SwapChallengeOrganizer {
    pub fn new(token_prices: Vec<(TokenAddress, TokenAddress, Price)>) -> Self {
        Self {
            batch_intents: vec![],
            solver_registry: vec![],
            solutions: vec![],
            token_prices,
        }
    }

    pub fn add_solver_to_registry(&mut self, solver_address: PublicKey) {
        self.solver_registry.push(solver_address);
    }
}

impl ChallengeOrganizer<PublicKey, SwapIntent> for SwapChallengeOrganizer {
    type Score = SwapScore;
    type Solution = SwapSolution;

    fn verify_solution(
        &self,
        solver_address: PublicKey,
        solution: &Self::Solution,
    ) -> Result<(), SolinaError> {
        // no solver should publish more than once a solution
        if self.solver_registry.contains(&solver_address) {
            return Err(SolinaError::FailedSolutionVerification(String::from(
                "Solver registry already contains solver's public key",
            )));
        }

        for row in solution.table.rows.iter() {
            let row_intent = self
                .batch_intents
                .iter()
                .find(|intent| intent.address == row.address)
                .ok_or(SolinaError::FailedSolutionVerification(String::from(
                    "Solver's solution contains invalid intent address",
                )))?;

            let intent_quote_token_address = row_intent.inputs.quote_token.clone();
            let intent_base_token_address = row_intent.inputs.base_token.clone();
            let intent_quote_amount = row_intent.inputs.quote_amount.clone();
            let intent_min_base_amount = row_intent.constraints.min_base_token_amount.clone();

            let row_quote_token_address = row.base_token_address.clone();
            let row_base_token_address = row.quote_token_address.clone();
            let row_base_token_amount = row.amount_in.clone();
            let row_quote_token_amount = row.amount_out.clone();

            if intent_quote_token_address != row_quote_token_address {
                return Err(SolinaError::FailedSolutionVerification(String::from(
                    "Solver's quote token address mismatches that of current intent",
                )));
            }

            if intent_base_token_address != row_base_token_address {
                return Err(SolinaError::FailedSolutionVerification(String::from(
                    "Solver's base token address mismatches that of current intent",
                )));
            }

            if intent_min_base_amount > row_base_token_amount {
                return Err(SolinaError::FailedSolutionVerification(String::from(
                    "Solver's solution failed to satisfy user intent constraints",
                )));
            }

            let challenger_token_pair_price = self
                .get_token_pair_price(intent_quote_token_address, intent_base_token_address)
                .ok_or(SolinaError::FailedSolutionVerification(String::from(
                    "UnexpectedBehavior: No available challenger price for intent token pair",
                )))?;

            let expected_solution_base_amount =
                BigDecimal::parse_bytes(row_quote_token_amount.to_str_radix(10).as_bytes(), 10)
                    .expect("Failed to parse BigDecimal from bytes")
                    .mul(challenger_token_pair_price)
                    .with_scale(0)
                    .to_bigint() // take the floor of the number
                    .ok_or(SolinaError::ConversionFailed(String::from(
                        "Failed to convert price `BigDecimal` to `BigInt`",
                    )))?;

            if expected_solution_base_amount != row_base_token_amount.into() {
                return Err(SolinaError::FailedSolutionVerification(String::from("Solver's solution base token amount was not computed according to challenger's proposed token pair price")));
            }

            if intent_quote_amount < row_quote_token_amount {
                return Err(SolinaError::FailedSolutionVerification(String::from("Solver's solution is invalid: traded quote amount is higher than intent's provided quote amount")));
            }
        }

        Ok(())
    }

    fn propose_batch_intent(&self) -> Vec<&SwapIntent> {
        self.batch_intents.iter().map(|x| x).collect::<Vec<_>>()
    }

    fn compute_solution_score(_solution: Self::Solution) -> Self::Score {
        todo!()
    }

    fn submit_intent(&mut self, intent: SwapIntent) {
        self.batch_intents.push(intent);
    }

    fn submit_solution(
        &mut self,
        solver_addr: PublicKey,
        solution: Self::Solution,
    ) -> Result<(), SolinaError> {
        self.verify_solution(solver_addr, &solution)?;
        self.solutions.push((solver_addr, solution));
        Ok(())
    }
}

impl SwapChallengeOrganizer {
    pub fn all_token_pairs(&self) -> Vec<(&Amount, &Amount)> {
        self.token_prices.iter().map(|(a, b, _)| (a, b)).collect()
    }

    pub fn contains_token_pair(&self, token_a: &TokenAddress, token_b: &TokenAddress) -> bool {
        let all_token_pairs = self.all_token_pairs();
        if all_token_pairs.contains(&(token_a, token_b))
            || all_token_pairs.contains(&(token_b, token_a))
        {
            return true;
        }
        false
    }

    pub fn get_token_pair_price(
        &self,
        token_a: TokenAddress,
        token_b: TokenAddress,
    ) -> Option<Price> {
        self.token_prices
            .iter()
            .filter_map(|(a, b, p)| {
                if (a, b) == (&token_a, &token_b) {
                    Some(p.clone())
                } else if (a, b) == (&token_b, &token_a) {
                    // the price of token pair (a, b) is the inverse of the price of token pair (b, a)
                    Some(BigDecimal::from(1).div(p.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .first()
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn it_works_token_pair_price() {
        let challenger = SwapChallengeOrganizer::new(vec![
            (
                BigUint::from_str("0").unwrap(),
                BigUint::from_str("1").unwrap(),
                BigDecimal::from_str("1_000_000").unwrap(),
            ),
            (
                BigUint::from_str("0").unwrap(),
                BigUint::from_str("2").unwrap(),
                BigDecimal::from_str("5_000_000").unwrap(),
            ),
            (
                BigUint::from_str("1").unwrap(),
                BigUint::from_str("2").unwrap(),
                BigDecimal::from_str("15_000_000").unwrap(),
            ),
        ]);

        assert_eq!(
            challenger.get_token_pair_price(
                BigUint::from_str("0").unwrap(),
                BigUint::from_str("1").unwrap(),
            ),
            Some(BigDecimal::from_str("1_000_000").unwrap())
        );

        assert_eq!(
            challenger.get_token_pair_price(
                BigUint::from_str("0").unwrap(),
                BigUint::from_str("2").unwrap(),
            ),
            Some(BigDecimal::from_str("5_000_000").unwrap())
        );

        assert_eq!(
            challenger.get_token_pair_price(
                BigUint::from_str("1").unwrap(),
                BigUint::from_str("2").unwrap(),
            ),
            Some(BigDecimal::from_str("15_000_000").unwrap())
        );

        // assert_eq!(
        //     challenger.get_token_pair_price(
        //         BigUint::from_str("1").unwrap(),
        //         BigUint::from_str("0").unwrap(),
        //     ),
        //     Some(BigDecimal::from_str("0").unwrap())
        // );

        assert_eq!(
            challenger.get_token_pair_price(
                BigUint::from_str("0").unwrap(),
                BigUint::from_str("2").unwrap(),
            ),
            Some(BigDecimal::from_str("5_000_000").unwrap())
        );

        assert_eq!(
            challenger.get_token_pair_price(
                BigUint::from_str("1").unwrap(),
                BigUint::from_str("2").unwrap(),
            ),
            Some(BigDecimal::from_str("15_000_000").unwrap())
        )
    }

    #[test]
    fn aux_test() {
        let x = BigDecimal::from_str("1.99999001").unwrap();
        let (z, y) = x.as_bigint_and_exponent();
        let w = x.with_scale(0);
        println!("{}", w.to_bigint().unwrap());
    }
}
