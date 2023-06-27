use bigdecimal::BigDecimal;
use num_bigint::BigUint;

pub mod swap_challenger;
pub mod swap_intent;
pub mod swap_solver;

pub type Amount = BigUint;
pub type Price = BigDecimal;
pub type PublicKey = [u8; 32];
pub type TokenAddress = BigUint;
