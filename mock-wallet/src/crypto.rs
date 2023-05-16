use num_bigint::BigUint;
use plonky2::field::types::Field;
use plonky2_ecdsa::curve::ecdsa::{ECDSAPublicKey, ECDSASecretKey};
use secp256k1::ecdsa::Signature;
use secp256k1::hashes::sha256;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{KeyPair, Message, PublicKey, Secp256k1, SecretKey};

pub trait MockWallet {
    fn initialize_wallet() -> Self;
    fn get_public_key(&self) -> PublicKey;
    fn get_public_key_plonky2_format(&self) -> ECDSAPublicKey<C>;
    fn sign_message(&self, message: &Message) -> Signature;
    fn verify_message(&self, message: &Message, signature: &Signature) -> bool;
}

pub type C = plonky2_ecdsa::curve::secp256k1::Secp256K1;

pub struct Wallet {
    private_key: SecretKey,
}

impl MockWallet for Wallet {
    fn initialize_wallet() -> Self {
        let secp = Secp256k1::new();
        let (private_key, _public_key) = secp.generate_keypair(&mut OsRng);
        Self { private_key }
    }

    fn get_public_key(&self) -> PublicKey {
        let secp = Secp256k1::new();
        PublicKey::from_secret_key(&secp, &self.private_key)
    }

    fn get_public_key_plonky2_format(&self) -> ECDSAPublicKey<C> {
        let secret_key_slice = &self.private_key.secret_bytes()[..];
        let secret_key_biguint = BigUint::from_bytes_le(secret_key_slice);
        let secp256k1_scalar_noncanonical_sk =
            plonky2::field::secp256k1_scalar::Secp256K1Scalar::from_noncanonical_biguint(
                secret_key_biguint,
            );
        let secret_key: ECDSASecretKey<C> = ECDSASecretKey(secp256k1_scalar_noncanonical_sk);
        let public_key = secret_key.to_public();
        public_key
    }

    fn sign_message(&self, message: &Message) -> Signature {
        let secp = Secp256k1::new();
        let signature = secp.sign_ecdsa(message, &self.private_key);
        signature
    }

    fn verify_message(&self, message: &Message, signature: &Signature) -> bool {
        let secp = Secp256k1::new();
        let public_key = self.get_public_key();
        secp.verify_ecdsa(message, signature, &public_key).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hex_literal::hex;
    use num_bigint::BigUint;
    use plonky2::field::{
        packed::PackedField,
        secp256k1_scalar::Secp256K1Scalar,
        types::{Field, PrimeField, Sample},
    };
    use plonky2_ecdsa::curve::ecdsa::{sign_message, verify_message, ECDSASecretKey};
    use secp256k1::{ecdh::shared_secret_point, rand::RngCore};

    use super::*;

    #[test]
    fn it_works_signature() {
        let secp = Secp256k1::new();
        let wallet = Wallet::initialize_wallet();
        let public_key = wallet.get_public_key();

        let mut data = [0u8; 32];
        OsRng.fill_bytes(&mut data);

        let message = Message::from_hashed_data::<sha256::Hash>(&data);

        dbg!(message);

        let signature = wallet.sign_message(&message);
        assert!(secp.verify_ecdsa(&message, &signature, &public_key).is_ok());
    }

    // ------------------------------------------------------------------------------------------
    // Test suite for comparing both Rust secp256k1 and plonky2_ecdsa
    // libraries. Our goal is to show that signatures are compatible across
    // these two libraries.

    #[test]
    fn test_curve_generators() {
        // x coordinate generator used in plonky2_ecdsa rust library,
        // in little endian representation. See documentation
        // https://docs.rs/crate/plonky2_ecdsa/0.1.0/source/src/curve/secp256k1.rs
        // biguint repr = 55066263022277343669578718895168534326250603453777594175500187360389116729240
        let mut plonky2_ecdsa_gen_x_u64: [u64; 4] = [
            0x59F2815B16F81798,
            0x029BFCDB2DCE28D9,
            0x55A06295CE870B07,
            0x79BE667EF9DCBBAC,
        ];
        let plonky_2_ecdsa_gen_x_u8 = plonky2_ecdsa_gen_x_u64
            .iter()
            .flat_map(|u| u.to_le_bytes())
            .collect::<Vec<u8>>();

        plonky2_ecdsa_gen_x_u64.reverse();
        println!(
            "FLAG: {}",
            BigUint::from_bytes_be(
                &plonky2_ecdsa_gen_x_u64
                    .iter()
                    .flat_map(|u| u.to_be_bytes())
                    .collect::<Vec<_>>()
            )
        );

        // x coordinate generator used in secp256k1 rust library,
        // in big little endian representation. See documentation
        // https://docs.rs/crate/secp256k1/latest/source/src/constants.rs
        // biguint repr = 55066263022277343669578718895168534326250603453777594175500187360389116729240
        let secp256k1_gen_x_u8: [u8; 32] = [
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87,
            0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b,
            0x16, 0xf8, 0x17, 0x98,
        ];
        println!(
            "PLONKY2_BIGUINT: {}",
            BigUint::from_bytes_le(&plonky_2_ecdsa_gen_x_u8)
        );
        println!(
            "SECP256K1_BIGUINT = {}",
            BigUint::from_bytes_be(&secp256k1_gen_x_u8)
        );

        // y coordinate generator used in plonky2_ecdsa rust library,
        // in little endian representation. See documentation
        // https://docs.rs/crate/plonky2_ecdsa/0.1.0/source/src/curve/secp256k1.rs
        // biguint repr = 32670510020758816978083085130507043184471273380659243275938904335757337482424
        let plonky_2_ecdsa_gen_y_u64: [u64; 4] = [
            0x9C47D08FFB10D4B8,
            0xFD17B448A6855419,
            0x5DA4FBFC0E1108A8,
            0x483ADA7726A3C465,
        ];
        let plonky_2_ecdsa_gen_y_u8 = plonky_2_ecdsa_gen_y_u64
            .iter()
            .flat_map(|u| u.to_le_bytes())
            .collect::<Vec<u8>>();

        // y coordinate generator used in secp256k1 rust library,
        // in big little endian representation. See documentation
        // https://docs.rs/crate/secp256k1/latest/source/src/constants.rs
        // biguint repr = 32670510020758816978083085130507043184471273380659243275938904335757337482424
        let secp256k1_gen_y_u8: [u8; 32] = [
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65, 0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11,
            0x08, 0xa8, 0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19, 0x9c, 0x47, 0xd0, 0x8f,
            0xfb, 0x10, 0xd4, 0xb8,
        ];

        assert_eq!(
            BigUint::from_bytes_le(&plonky_2_ecdsa_gen_x_u8),
            BigUint::from_bytes_be(&secp256k1_gen_x_u8)
        );
        assert_eq!(
            BigUint::from_bytes_le(&plonky_2_ecdsa_gen_y_u8),
            BigUint::from_bytes_be(&secp256k1_gen_y_u8)
        )
    }

    #[test]
    fn aux_test() {
        type C = plonky2_ecdsa::curve::secp256k1::Secp256K1;

        let plonky2_msg = Secp256K1Scalar::rand();
        let plonky2_sk = ECDSASecretKey::<C>(Secp256K1Scalar::rand());
        let plonky2_pk = plonky2_sk.to_public();
        let plonky2_sig = sign_message(plonky2_msg, plonky2_sk);

        assert_eq!(plonky2_sig.r.0.len(), 4);
        assert_eq!(plonky2_sig.s.0.len(), 4);

        assert!(verify_message(plonky2_msg, plonky2_sig, plonky2_pk));

        // to compare data formats across secp256k1 vs plonky_ecdsa2
        // we need to reverse the previous u64 data representation
        // to convert to big endian bytes
        let mut msg_u64_be_byte_data = plonky2_msg.0.clone();

        msg_u64_be_byte_data.reverse();
        let msg_u8_be_byte_data = msg_u64_be_byte_data
            .iter()
            .flat_map(|a| a.to_be_bytes())
            .collect::<Vec<u8>>();

        let secp = Secp256k1::new();
        let secp256k1_msg = Message::from_slice(&msg_u8_be_byte_data).unwrap();
        let mut plonky2_sk_u64_data = plonky2_sk.0 .0.clone();
        plonky2_sk_u64_data.reverse();
        let plonky2_sk_u8_data = plonky2_sk_u64_data
            .iter()
            .flat_map(|u| u.to_be_bytes())
            .collect::<Vec<u8>>();
        let secp256k1_secret_key = SecretKey::from_slice(&plonky2_sk_u8_data).unwrap();
        let secp256k1_public_key = secp256k1_secret_key.public_key(&secp);
        let mut signature_u64_data = [plonky2_sig.r.0.clone(), plonky2_sig.s.0.clone()].concat();
        assert_eq!(signature_u64_data.len(), 8);
        signature_u64_data.reverse();
        let signature_u8_data = signature_u64_data
            .iter()
            .flat_map(|u| u.to_be_bytes())
            .collect::<Vec<u8>>();

        let secp256k1_signature =
            Signature::from_compact(&signature_u8_data).expect("Failed to parse signature");

        assert!(secp
            .verify_ecdsa(&secp256k1_msg, &secp256k1_signature, &secp256k1_public_key)
            .is_ok());

        // let message = msg
        //     .0
        //     .iter()
        //     .flat_map(|a| a.to_le_bytes())
        //     .collect::<Vec<u8>>();
        // let private_key = sk.0.to_canonical_biguint().to_bytes_le();
        // let signature = [
        //     sig.r.to_canonical_biguint().to_bytes_le(),
        //     sig.s.to_canonical_biguint().to_bytes_le(),
        // ]
        // .concat();

        // let secp = Secp256k1::new();
        // let secret_key = SecretKey::from_slice(&private_key).unwrap();
        // let public_key = secret_key.public_key(&secp);
        // let signature = Signature::from_compact(&signature).unwrap();
        // let message = Message::from_slice(&message);

        // verify_message(msg, sig, pk);
    }

    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};

    #[test]
    fn aux_test1() {
        type C = plonky2_ecdsa::curve::secp256k1::Secp256K1;

        let msg = Secp256K1Scalar::rand();
        let sk = ECDSASecretKey::<C>(Secp256K1Scalar::rand());
        let pk = sk.to_public();
        let sig = sign_message(msg, sk);

        let secp = Secp256k1::new();
        let data =
            sk.0 .0
                .iter()
                .flat_map(|u| u.to_be_bytes())
                .collect::<Vec<u8>>();
        let aux_sk = SecretKey::from_slice(&data).unwrap();

        assert_eq!(aux_sk.as_ref(), data.as_slice());
        let aux_pk = aux_sk.public_key(&secp);
        println!(
            "FLAG: PUBLIC KEY = {:?}",
            BigUint::from_bytes_be(&aux_pk.x_only_public_key().0.serialize()[1..])
        );
        let public_key_x = pk.0.x.to_canonical_biguint();
        let public_key_y = pk.0.y.to_canonical_biguint();
        println!("FLAG: public key = {:?}", public_key_x);
    }
}
