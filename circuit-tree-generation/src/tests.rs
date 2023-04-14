use crate::tree_generation::MerkleTreeGeneration;
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField, types::Field},
    hash::{hash_types::RichField, merkle_proofs, merkle_tree::MerkleTree, poseidon::PoseidonHash},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::{self, CircuitBuilder},
        circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = GoldilocksField;
type H = PoseidonHash;

#[test]
fn tree_generation() {
    let f_zero: F = F::ZERO;
    let f_one: F = F::ONE;
    let f_two: F = F::from_canonical_u64(2);
    let f_three: F = F::from_canonical_u64(3);

    let config = CircuitConfig::standard_recursion_config();
    let mut circuit_builder = CircuitBuilder::<F, D>::new(config);

    let merkle_tree_leaves = vec![vec![f_zero], vec![f_one], vec![f_two], vec![f_three]];
    let merkle_tree = MerkleTree::<F, H>::new(merkle_tree_leaves.clone(), 0);
    let merkle_tree_root = merkle_tree.cap.0[0];

    let mut merkle_tree_leaf_targets = Vec::with_capacity(4);
    (0..4).for_each(|_| merkle_tree_leaf_targets.push(vec![circuit_builder.add_virtual_target()]));
    let zero_extend_target = circuit_builder.zero();
    let merkle_root_target = circuit_builder
        .add_merkle_root_target(merkle_tree_leaf_targets.clone(), zero_extend_target);

    let should_be_root_hash_target = circuit_builder.add_virtual_hash();
    for i in 0..4 {
        circuit_builder.connect(
            merkle_root_target.elements[i],
            should_be_root_hash_target.elements[i],
        );
    }

    let mut partial_witness = PartialWitness::<F>::new();
    for (vec_target, vec_f) in merkle_tree_leaf_targets.iter().zip(merkle_tree_leaves) {
        partial_witness.set_target(vec_target[0], vec_f[0]);
    }

    partial_witness.set_hash_target(should_be_root_hash_target, merkle_tree_root);

    let circuit_data = circuit_builder.build::<C>();
    // let proof_with_pis = circuit_data.prove(partial_witness).unwrap();

    // assert!(circuit_data.verify(proof_with_pis).is_ok());
}
