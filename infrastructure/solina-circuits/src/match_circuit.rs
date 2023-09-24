use crate::{C, D, F};
use plonky2::{
    iop::target::Target,
    plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitConfig},
};
use solina::solver::Match;
use zktree::circuit_compiler::CircuitCompiler;

pub type TokenAddressTargets = [Target; 4];

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

pub fn generate_match_circuit(match_instance: Match) -> (CircuitBuilder<F, D>, MatchTargets) {
    let mut circuit_builder =
        CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_zk_config());

    // 1. Verify that both intents have appropriate token addresses
    let intent_a_quote_token_targets = circuit_builder.add_virtual_targets(4);
    let intent_b_base_token_targets = circuit_builder.add_virtual_targets(4);

    (0..4).for_each(|i| {
        circuit_builder.connect(
            intent_a_quote_token_targets[i],
            intent_b_base_token_targets[i],
        )
    });

    let intent_a_base_token_targets = circuit_builder.add_virtual_targets(4);
    let intent_b_quote_token_targets = circuit_builder.add_virtual_targets(4);

    circuit_builder.connect(intent_a_base_token_targets, intent_b_quote_token_targets);

    // 2. Verify that the amount being swapped does not exceed the desired one, for each intent.
    let intent_a_quote_amount_targets =
        circuit_builder.add_virtual_biguint_target(match_instance.intent_a().inputs.quote_amount);
    let intent_b_quote_amount_targets =
        circuit_builder.add_virtual_biguint_target(match_instance.intent_b().inputs.quote_amount);

    let match_intent_a_amount_targets = circuit_builder
        .add_virtual_biguint_target(match_instance.swapped_amount().token_a_amount());
    let match_intent_b_amount_targets = circuit_builder
        .add_virtual_biguint_target(match_instance.swapped_amount().token_b_amount());
    // TODO: check for the inequalities

    todo!()
}
