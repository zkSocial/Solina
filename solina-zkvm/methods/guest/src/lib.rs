use num_bigint::BigUint;

pub mod swap_challenger;
pub mod swap_intent;
pub mod swap_solver;

pub type TokenAddress = BigUint;
pub type Amount = BigUint;
pub type PublicKey = [u8; 32];
