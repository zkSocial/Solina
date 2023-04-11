use std::marker::PhantomData;

use crate::config::Config;
use libipld::{
    cbor::DagCborCodec,
    prelude::{Decode, Encode},
    IpldCodec,
};

/// Fixed hash length
pub type FixedHash = [u8; 32]; // TODO: should we use bytes or F elements (for Poseidon hash)?

#[derive(Clone, Debug)]
pub struct Input<T> {
    value: T,
}

impl<T: Encode<IpldCodec> + 'static> Encode<IpldCodec> for Input<T> {
    fn encode<W: std::io::Write>(&self, c: IpldCodec, w: &mut W) -> libipld::Result<()> {
        T::encode(&self.value, c, w)
    }
}

impl<T: Decode<IpldCodec> + 'static> Decode<IpldCodec> for Input<T> {
    fn decode<R: std::io::Read + std::io::Seek>(c: IpldCodec, r: &mut R) -> libipld::Result<Self> {
        let value = T::decode(c, r)?;
        Ok(Self { value })
    }
}

pub struct Invocation<T> {
    function: FixedHash,
    // TODO: we need to have dynamic dispatch over encoded data s
    inputs: Vec<T>,
}

impl<T> Invocation<T> {
    fn get_inputs(function_hash: FixedHash, inputs: Vec<T>) -> Self {
        Self {
            function: function_hash,
            inputs,
        }
    }
}

pub struct Job<T> {
    invocation: Invocation<T>,
    config: Config,
}
