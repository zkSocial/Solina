use plonky2_ecdsa::gadgets::biguint::{BigUintTarget};

// todo: proper typing
pub struct TransferIntent {
    /// address
    from: BigUintTarget,
    /// address
    to: BigUintTarget,
    /// amount expressed in Wei
    amount: BigUintTarget,
    /// address
    token: BigUintTarget,
    /// before as unix timestamp; this would fit in u32 till 2106
    before: BigUintTarget,
}