use num_bigint::BigUint;
use plonky2::field::secp256k1_scalar::Secp256K1Scalar;
use plonky2_ecdsa::curve::{
    ecdsa::{ECDSAPublicKey, ECDSASignature},
    secp256k1::Secp256K1,
};

pub(crate) type Curve = Secp256K1;
pub(crate) type KeccakHash = [u8; 32];

pub enum Expr {
    BigUint(BigUint),
    U32(u32),
    Signature(ECDSASignature<Curve>),
    PublicKey(ECDSAPublicKey<Curve>),
    Message(Secp256K1Scalar),
}
