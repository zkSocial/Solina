use plonky2::{field::extension::Extendable, hash::hash_types::RichField};

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

impl<F: RichField + Extendable<D>, const D: usize> ExecuteResult<F, D> {
    pub fn new(invocation_hash: [F; 4], pure_output_hash: [F; 4], effects: Vec<Effect>) -> Self {
        Self {
            invocation_hash,
            pure_output_hash,
            effects,
        }
    }
}

pub struct Session<F: RichField + Extendable<D>, const D: usize> {
    job_hash: [F; 4],
    result: ExecuteResult<F, D>,
    trace_hash: [F; 4],
    error_hash: Option<[F; 4]>,
}

impl<F: RichField + Extendable<D>, const D: usize> Session<F, D> {
    pub fn new(
        job_hash: [F; 4],
        result: ExecuteResult<F, D>,
        trace_hash: [F; 4],
        error_hash: Option<[F; 4]>,
    ) -> Self {
        Self {
            job_hash,
            result,
            trace_hash,
            error_hash,
        }
    }
}
