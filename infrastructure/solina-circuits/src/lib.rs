use plonky2::{
    field::goldilocks_field::GoldilocksField, hash::poseidon::PoseidonHash,
    plonk::config::PoseidonGoldilocksConfig,
};

pub mod match_circuit;
pub mod solver_circuit;

pub const D: usize = 2;

pub type F = GoldilocksField;
pub type C = PoseidonGoldilocksConfig;
pub type H = PoseidonHash;
