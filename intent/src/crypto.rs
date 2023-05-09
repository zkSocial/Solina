use num_bigint::BigUint;
use plonky2::{
    field::{
        extension::Extendable, goldilocks_field::GoldilocksField,
        secp256k1_scalar::Secp256K1Scalar, types::PrimeField,
    },
    hash::hash_types::RichField,
    iop::witness::PartialWitness,
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_ecdsa::{
    curve::{
        ecdsa::{ECDSAPublicKey, ECDSASecretKey, ECDSASignature},
        secp256k1::Secp256K1,
    },
    gadgets::{biguint::BigUintTarget, ecdsa::verify_message_circuit},
};

use crate::encoding::StructuredEncodedBytes;

pub(crate) type Curve = Secp256K1;
pub(crate) type KeccakHash = [u8; 32];

pub enum Expr {
    BigUint(BigUint),
    U32(u32),
    Signature(ECDSASignature<Curve>),
    PublicKey(ECDSAPublicKey<Curve>),
    Message(Secp256K1Scalar),
}
