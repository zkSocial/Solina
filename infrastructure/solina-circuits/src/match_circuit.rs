use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitConfig},
};
use plonky2_u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target};
use solina::solver::Match;

pub type TokenAddressTargets = [U32Target; 4];

pub struct MatchTargets {
    intent_a_quote_token_targets: TokenAddressTargets,
    intent_b_base_token_targets: TokenAddressTargets,
    intent_a_base_token_targets: TokenAddressTargets,
    intent_b_quote_token_targets: TokenAddressTargets,
}

pub struct MatchCircuitData<F: RichField + Extendable<D>, const D: usize> {
    circuit_builder: CircuitBuilder<F, D>,
    targets: MatchTargets,
}

pub fn generate_match_circuit<F, const D: usize>(match_instance: Match) -> MatchCircuitData<F, D>
where
    F: RichField + Extendable<D>,
{
    let mut circuit_builder =
        CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_zk_config());

    // 1. Verify that both intents have appropriate token addresses
    let intent_a_quote_token_targets = circuit_builder.add_virtual_u32_targets(4);
    let intent_b_base_token_targets = circuit_builder.add_virtual_u32_targets(4);

    (0..4).for_each(|i| {
        circuit_builder.connect_u32(
            intent_a_quote_token_targets[i],
            intent_b_base_token_targets[i],
        )
    });

    let intent_a_base_token_targets = circuit_builder.add_virtual_u32_targets(4);
    let intent_b_quote_token_targets = circuit_builder.add_virtual_u32_targets(4);

    intent_a_base_token_targets
        .iter()
        .zip(intent_b_quote_token_targets)
        .for_each(|(a, b)| circuit_builder.connect_u32(*a, b));

    // 2. Verify that the amount being swapped does not exceed the desired one, for each intent.
    let intent_a_quote_amount_targets = circuit_builder.add_virtual_u32_targets(
        match_instance
            .intent_a()
            .inputs
            .quote_amount
            .to_u32_digits()
            .len(),
    );
    let intent_b_quote_amount_targets = circuit_builder.add_virtual_u32_targets(
        match_instance
            .intent_b()
            .inputs
            .quote_amount
            .to_u32_digits()
            .len(),
    );

    let match_intent_a_amount_targets = circuit_builder.add_virtual_u32_targets(
        match_instance
            .swapped_amount()
            .token_a_amount()
            .to_u32_digits()
            .len(),
    );
    let match_intent_b_amount_targets = circuit_builder.add_virtual_u32_targets(
        match_instance
            .swapped_amount()
            .token_b_amount()
            .to_u32_digits()
            .len(),
    );
    // TODO: check for the inequalities

    todo!()
}
