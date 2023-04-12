use crate::{config::Config, input::Input};
use plonky2::{field::extension::Extendable, hash::hash_types::RichField};

pub struct Invocation<F: RichField + Extendable<D>, const D: usize> {
    function_hash: [F; 4],
    inputs: Vec<Input<F, D>>,
}

impl<F: RichField + Extendable<D>, const D: usize> Invocation<F, D> {
    pub fn new(function_hash: [F; 4], inputs: Vec<Input<F, D>>) -> Self {
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

pub enum Effect {
    Error,
    FutureEffect,
    ContinueExecutation,
}

pub struct ExecuteResult<F: RichField + Extendable<D>, const D: usize> {
    invocation_hash: [F; 4],
    pure_output_hash: [F; 4],
    effects: Vec<Effect>,
}

pub struct Session<F: RichField + Extendable<D>, const D: usize> {
    job_hash: [F; 4],
    result: ExecuteResult<F, D>,
    trace_hash: [F; 4],
    error_hash: Option<[F; 4]>,
}
