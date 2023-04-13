use crate::job::{Invocation, Job};
use plonky2::{
    field::extension::Extendable,
    hash::{hash_types::RichField, keccak::KeccakHash},
    plonk::circuit_builder::CircuitBuilder,
};

/// 1. Simple accumulator is generic (wrapping around the Merkle Tree);
/// 2. Job is a trait (subfolder in which we add demo use cases);
/// 3. Accumulator has one circuit, if we naively recurse with the verifier of the previous circuit
///    we enforce that the function being used is always the same.

/// Generates circuit for proving a job:
/// 1. Generates [`BoolTarget`]'s for function hash;
/// 2. Generates [`BoolTarget`]'s for function signature;
/// 3. Connects the hash of function signature with function hash;
/// 4. For each input, adds targets for each of its elements
pub(crate) fn build_job_circuit<F: RichField + Extendable<D>, const D: usize>(
    circuit_builder: &mut CircuitBuilder<F, D>,
    job: Job<F, D>,
    function_signature: Vec<u8>,
) {
    let Invocation {
        function_hash,
        inputs,
    } = job.invocation;

    for input in inputs {
        let input_len = input.field_values.len();
        circuit_builder.add_virtual_targets(input_len);
    }

    let mut input_hashes = Vec::with_capacity(32 * 8);
    for _ in 0..32 * 8 {
        input_hashes.push(circuit_builder.add_virtual_bool_target_safe());
    }
}
