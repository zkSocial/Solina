use crate::{error::SolinaError, intent::Intent};

pub trait ChallengeOrganizer<Addr, Int>
where
    Int: Intent,
{
    type Score;
    type Solution;

    fn verify_solution(
        &self,
        solver_addr: Addr,
        solution: &Self::Solution,
    ) -> Result<(), SolinaError>;
    fn propose_batch_intent(&self) -> Vec<&Int>;
    fn compute_solution_score(solution: Self::Solution) -> Self::Score;
    fn submit_intent(&mut self, intent: Int);
    fn submit_solution(
        &mut self,
        solver_addr: Addr,
        solution: Self::Solution,
    ) -> Result<(), SolinaError>;
}
