pub mod config;
pub mod input;
pub mod job;
pub mod job_circuit;
pub mod session;

pub(crate) const U64_BYTES_LEN: usize = 8;
pub(crate) type QmHashBytes = [u8; 32];
