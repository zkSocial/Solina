use plonky2::plonk::circuit_builder::CircuitBuilder;
use solina::solver::Match;
use zktree::circuit_compiler::CircuitCompiler;

pub type TokenAddressTargets = [Target; 4];

impl CircuitCompiler<F, C, D> for Match {
    type Targets = MatchTargets;
    type OutTargets = ();

    fn compile(&self) -> (CircuitBuilder<F, D>, Self::Targets, Self::OutTargets) {
        todo!();
    }
}

pub struct MatchTargets {
    intent_a_quote_token_targets: TokenAddressTargets,
    intent_b_base_token_targets: TokenAddressTargets,
    intent_a_base_token_targets: TokenAddressTargets,
    intent_b_quote_token_targets: TokenAddressTargets,
}

pub struct MatchCircuitData {
    circuit_builder: CircuitBuilder<F, D>,
    targets: MatchTargets,
}

pub fn generate_match_circuit() -> (CircuitBuilder<F, D>, MatchTargets) {
    // 1. Verify that both intents have appropriate token addresses
    let intent_a_quote_token_targets = circuit_builder.add_virtual_targets(4);
    let intent_b_base_token_targets = circuit_builder.add_virtual_targets(4);

    circuit_builder.connect(intent_a_quote_token_targets, intent_b_base_token_targets);

    let intent_a_base_token_targets = circuit_builder.add_virtual_targets(4);
    let intent_b_quote_token_targets = circuit_builder.add_virtual_targets(4);

    circuit_builder.connect(intent_a_base_token_targets, intent_b_quote_token_targets);

    // 2. Verify that the amount being swapped does not exceed the desired one, for each intent.
    let intent_a_quote_amount_targets =
        circuit_builder.add_biguint_target(self.intent_a.quote_amount);
    let intent_b_quote_amount_targets =
        circuit_builder.add_biguint_target(self.intent_b.quote_amount);

    let match_intent_a_amount_targets =
        circuit_builder.add_biguint_target(self.swapped_amount.token_a_amount);
    let match_intent_b_amount_targets =
        circuit_builder.add_biguint_target(self.swapped_amount.token_b_amount);
    // TODO: check for the inequalities
}
