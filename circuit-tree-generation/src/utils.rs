use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::Target,
    plonk::circuit_builder::CircuitBuilder,
};

/// Extends a given vector of `Target`s of a certain length len
/// to a power of 2 len vector of `Target`s. This is done, by
/// appending with a constant `Target` of the length with a fixed `to_exted_target` to the original vector,
pub fn extend_targets_to_power_of_two<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    mut targets: Vec<Vec<Target>>,
    to_extend_target: Target,
) -> Vec<Vec<Target>> {
    let log_2_len = targets.len().ilog2();
    if 2_u64.pow(log_2_len) == targets.len() as u64 {
        return targets;
    }
    let diff = 2_u64.pow(log_2_len + 1) - targets.len() as u64 - 1;

    // append length of `targets`
    targets.push(vec![
        builder.constant(F::from_canonical_u64(targets.len() as u64))
    ]);
    // trivially extend the vector until we obtain a power 2 length output vector
    let to_extend_targets = vec![vec![to_extend_target]; diff as usize];
    targets.extend(to_extend_targets);
    targets
}
