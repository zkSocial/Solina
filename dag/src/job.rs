use crate::{config::Config, input::Input, QmHashBytes};
use plonky2::{field::extension::Extendable, hash::hash_types::RichField};

pub struct Invocation<F: RichField + Extendable<D>, const D: usize> {
    function_hash: QmHashBytes,
    inputs: Vec<Input<F, D>>,
}

impl<F: RichField + Extendable<D>, const D: usize> Invocation<F, D> {
    pub fn new(function_hash: QmHashBytes, inputs: Vec<Input<F, D>>) -> Self {
        Self {
            function_hash,
            inputs,
        }
    }
}

pub struct Job<F: RichField + Extendable<D>, const D: usize> {
    invocation: Invocation<F, D>,
    config: Config,
}

impl<F: RichField + Extendable<D>, const D: usize> Job<F, D> {
    pub fn new(invocation: Invocation<F, D>, config: Config) -> Self {
        Self { invocation, config }
    }
}
