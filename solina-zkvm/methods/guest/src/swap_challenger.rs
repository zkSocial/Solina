use crate::{swap_intent::SwapIntent, Amount, PublicKey, TokenAddress};
use solina::{challenger::ChallengeOrganizer, error::SolinaError};

pub type SwapScore = u64;

#[derive(Clone, Debug)]
pub struct SwapRow {
    pub(crate) addr: PublicKey,
    pub(crate) amount_in: Amount,
    pub(crate) amount_out: Amount,
}

impl SwapRow {
    pub fn new(addr: PublicKey, amount_in: Amount, amount_out: Amount) -> Self {
        Self {
            addr,
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
        self.addr
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
            .filter(|x| x.addr == addr)
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
    token_prices: Vec<(TokenAddress, TokenAddress, Amount)>,
}

impl SwapChallengeOrganizer {
    pub fn new(token_prices: Vec<(TokenAddress, TokenAddress, Amount)>) -> Self {
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

        for row in solution.table.rows.iter() {}

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
    ) -> Option<Amount> {
        self.token_prices
            .iter()
            .filter_map(|(a, b, p)| match (a, b) {
                (token_a, token_b) => Some(p.clone()),
                (token_b, token_a) => Some(p.clone()),
                _ => None,
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
                BigUint::from_str("1_000_000").unwrap(),
            ),
            (
                BigUint::from_str("0").unwrap(),
                BigUint::from_str("2").unwrap(),
                BigUint::from_str("5_000_000").unwrap(),
            ),
        ]);

        assert_eq!(
            challenger.get_token_pair_price(
                BigUint::from_str("0").unwrap(),
                BigUint::from_str("1").unwrap(),
            ),
            Some(BigUint::from_str("1_000_000").unwrap())
        )
    }
}
