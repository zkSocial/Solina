use crate::QmHashBytes;
use plonky2::{field::extension::Extendable, hash::hash_types::RichField};

pub enum Effect {
    PureEffect,
    ImpureEffect,
    Error,
}

pub struct ExecuteResult {
    invocation_hash: QmHashBytes,
    pure_output_hash: QmHashBytes,
    effects: Vec<Effect>,
}

impl ExecuteResult {
    pub fn new(
        invocation_hash: QmHashBytes,
        pure_output_hash: QmHashBytes,
        effects: Vec<Effect>,
    ) -> Self {
        Self {
            invocation_hash,
            pure_output_hash,
            effects,
        }
    }
}

pub struct Session {
    job_hash: QmHashBytes,
    result: ExecuteResult,
    trace_hash: QmHashBytes,
    error_hash: Option<QmHashBytes>,
}

impl Session {
    pub fn new(
        job_hash: QmHashBytes,
        result: ExecuteResult,
        trace_hash: QmHashBytes,
        error_hash: Option<QmHashBytes>,
    ) -> Self {
        Self {
            job_hash,
            result,
            trace_hash,
            error_hash,
        }
    }
}
