use crate::utils::extend_targets_to_power_of_two;
use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::Target,
    plonk::circuit_builder::CircuitBuilder,
};

pub trait MerkleTreeGeneration<F: RichField + Extendable<D>, const D: usize> {
    type Hasher;
    fn build_merkle_root_target(
        builder: &mut CircuitBuilder<F, D>,
        targets: Vec<Vec<Target>>,
        to_extend_target: Target,
    ) -> HashOutTarget;
}

impl<F: RichField + Extendable<D>, const D: usize> MerkleTreeGeneration<F, D>
    for CircuitBuilder<F, D>
{
    type Hasher = PoseidonHash;
    fn build_merkle_root_target(
        builder: &mut CircuitBuilder<F, D>,
        targets: Vec<Vec<Target>>,
        to_extend_target: Target,
    ) -> HashOutTarget {
        // extend `targets` to a length of power of two vector
        let targets = extend_targets_to_power_of_two(builder, targets, to_extend_target);
        // build the merkle tree root target
        let merkle_tree_height = targets.len().ilog2();
        let mut tree_hash_targets = vec![];
        for i in 0..targets.len() {
            let hash_target = builder.hash_or_noop::<Self::Hasher>(targets[i].clone());
            tree_hash_targets.push(hash_target);
        }
        let mut current_tree_height_index = 0;
        for height in 0..merkle_tree_height {
            // TODO: do we want to loop over all the height, or until cap(1) ?
            for i in 0..(1 << merkle_tree_height - height) {
                let hash_targets = builder.hash_n_to_hash_no_pad::<Self::Hasher>(
                    [
                        tree_hash_targets[i as usize].elements.clone(),
                        tree_hash_targets[i as usize + 1].elements.clone(),
                    ]
                    .concat(),
                );
                tree_hash_targets.push(hash_targets);
            }
            current_tree_height_index += 1 << height;
        }
        *tree_hash_targets.last().unwrap()
    }
}
