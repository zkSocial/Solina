use plonky2::field::{goldilocks_field::GoldilocksField, secp256k1_scalar::Secp256K1Scalar};

mod circuit;
mod crypto;
mod encoding;
mod intent;
mod utils;

pub(crate) const D: usize = 2;
pub(crate) type F = GoldilocksField;
pub(crate) type FF = Secp256K1Scalar;
