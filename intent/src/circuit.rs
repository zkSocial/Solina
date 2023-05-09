use num_bigint::BigUint;
use plonky2::{
    field::{
        extension::Extendable,
        goldilocks_field::GoldilocksField,
        types::{Field, PrimeField},
    },
    hash::hash_types::RichField,
    iop::witness::PartialWitness,
    iop::{target::Target, witness::WitnessWrite},
    plonk::circuit_builder::{self, CircuitBuilder},
};
use plonky2_ecdsa::{
    curve::{
        curve_types::Curve,
        ecdsa::{ECDSAPublicKey, ECDSASecretKey, ECDSASignature},
        secp256k1::Secp256K1,
    },
    gadgets::{
        biguint::{BigUintTarget, WitnessBigUint},
        curve::{AffinePointTarget, CircuitBuilderCurve},
        ecdsa::{verify_message_circuit, ECDSAPublicKeyTarget, ECDSASignatureTarget},
        nonnative::{CircuitBuilderNonNative, NonNativeTarget},
    },
};

use crate::{
    crypto::Curve as C,
    encoding::{StructuredEncodedBytes, StructuredEncodedU32, ENCODED_U32_LEN},
    D, F, FF,
};

// pub(crate) const D: usize = 2;
// pub(crate) type C = Secp256K1;
// pub(crate) type F = GoldilocksField;

// pub trait Compile {
//     type Targets;
//     fn compile(
//         &self,
//         circuit_builder: &mut CircuitBuilder<F, D>,
//         partial_witness: &mut PartialWitness<F>,
//     );
// }

// pub enum Expr {
//     BigUint(BigUint),
//     U32(u32),
//     Signature(ECDSASecretKey<C>),
//     PublicKey(ECDSAPublicKey<C>),
//     Message(<C as Curve>::ScalarField),
// }

pub(crate) trait SignedStructuredHashedMessageCircuit {
    fn verify_signed_message(&mut self) -> ECDSASignatureCircuitTargets;
    fn verify_keccak_structured_hash_message(
        &mut self,
        message: StructuredEncodedU32,
    ) -> KeccakStructuredHashTargets;
}

pub(crate) struct ECDSASignatureCircuitTargets {
    public_key_ecdsa_target: ECDSAPublicKeyTarget<C>,
    message_nonnative_target: NonNativeTarget<FF>,
    signature_ecdsa_signature_target: ECDSASignatureTarget<C>,
}

pub(crate) struct KeccakStructuredHashTargets {
    pub(crate) message_targets: [Target; ENCODED_U32_LEN],
}

impl SignedStructuredHashedMessageCircuit for CircuitBuilder<F, D> {
    fn verify_signed_message(&mut self) -> ECDSASignatureCircuitTargets {
        let public_key_affine_target = self.add_virtual_affine_point_target::<C>();
        let public_key_ecdsa_target = ECDSAPublicKeyTarget(public_key_affine_target);

        let message_nonnative_target = self.add_virtual_nonnative_target::<FF>();

        let r_ecdsa_nonnative_target = self.add_virtual_nonnative_target::<FF>();
        let s_ecdsa_nonnative_target = self.add_virtual_nonnative_target::<FF>();
        let signature_ecdsa_signature_target = ECDSASignatureTarget {
            r: r_ecdsa_nonnative_target,
            s: s_ecdsa_nonnative_target,
        };

        verify_message_circuit(
            self,
            message_nonnative_target.clone(),
            signature_ecdsa_signature_target.clone(),
            public_key_ecdsa_target.clone(),
        );

        ECDSASignatureCircuitTargets {
            public_key_ecdsa_target,
            message_nonnative_target,
            signature_ecdsa_signature_target,
        }
    }

    fn verify_keccak_structured_hash_message(
        &mut self,
        message: StructuredEncodedU32,
    ) -> KeccakStructuredHashTargets {
        // add virtual targets for message
        let message_targets = self.add_virtual_target_arr::<ENCODED_U32_LEN>();
        // set previous targets as public inputs
        self.register_public_inputs(&message_targets);

        let first_byte_encoding = self.constant(F::from_canonical_u32(0x19));
        let second_byte_encoding = self.constant(F::from_canonical_u32(0x01));

        // enforce that the first two bytes are 0x19, 0x01, respectively
        self.connect(message_targets[0], first_byte_encoding);
        self.connect(message_targets[1], second_byte_encoding);

        let message = message.to_array();
        let mut goldilocks_encoded_message = [F::ZERO; ENCODED_U32_LEN];
        (0..ENCODED_U32_LEN)
            .for_each(|i| goldilocks_encoded_message[i] = F::from_canonical_u32(message[i]));

        KeccakStructuredHashTargets { message_targets }
    }
}

pub(crate) trait SignedStructuredHashedMessageWitness {
    fn verify_signed_message(
        &mut self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        message: FF,
        public_key: ECDSAPublicKey<C>,
        signature: ECDSASignature<C>,
        targets: ECDSASignatureCircuitTargets,
    );
    fn verify_keccak_structured_hash_message(
        &mut self,
        message: StructuredEncodedU32,
        targets: KeccakStructuredHashTargets,
    );
}

impl SignedStructuredHashedMessageWitness for PartialWitness<F> {
    fn verify_signed_message(
        &mut self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        message: FF,
        public_key: ECDSAPublicKey<C>,
        signature: ECDSASignature<C>,
        targets: ECDSASignatureCircuitTargets,
    ) {
        let ECDSASignatureCircuitTargets {
            message_nonnative_target,
            public_key_ecdsa_target,
            signature_ecdsa_signature_target,
        } = targets;
        let message_biguint_target =
            circuit_builder.nonnative_to_canonical_biguint(&message_nonnative_target);
        self.set_biguint_target(&message_biguint_target, &message.to_canonical_biguint());
    }

    fn verify_keccak_structured_hash_message(
        &mut self,
        message: StructuredEncodedU32,
        targets: KeccakStructuredHashTargets,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use plonky2::{
        field::{secp256k1_scalar::Secp256K1Scalar, types::Sample},
        plonk::{
            circuit_data::CircuitConfig,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };
    use plonky2_ecdsa::curve::{curve_types::CurveScalar, ecdsa::sign_message};

    use super::*;

    #[test]
    fn it_works_signature_circuit_verification() {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;

        type Curve = Secp256K1;

        let pw = PartialWitness::<F>::new();

        let config = CircuitConfig::standard_ecc_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let msg = Secp256K1Scalar::rand();
        let msg_target = builder.constant_nonnative(msg);

        let sk = ECDSASecretKey::<Curve>(Secp256K1Scalar::rand());
        let pk = ECDSAPublicKey((CurveScalar(sk.0) * Curve::GENERATOR_PROJECTIVE).to_affine());

        let pk_target = ECDSAPublicKeyTarget(builder.constant_affine_point(pk.0));

        let sig = sign_message(msg, sk);

        let ECDSASignature { r, s } = sig;
        let r_target = builder.constant_nonnative(r);
        let s_target = builder.constant_nonnative(s);
        let sig_target: ECDSASignatureTarget<Curve> = ECDSASignatureTarget {
            r: r_target,
            s: s_target,
        };

        let ECDSASignatureCircuitTargets {
            public_key_ecdsa_target,
            message_nonnative_target,
            signature_ecdsa_signature_target,
        } = builder.verify_signed_message();

        builder.connect_affine_point(&public_key_ecdsa_target.0, &pk_target.0);
        builder.connect_nonnative(&sig_target.r, &signature_ecdsa_signature_target.r);
        builder.connect_nonnative(&sig_target.s, &signature_ecdsa_signature_target.s);
        builder.connect_nonnative(&msg_target, &message_nonnative_target);

        dbg!(builder.num_gates());
        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();
        data.verify(proof).expect("Failed to verify proof data")
    }
}
