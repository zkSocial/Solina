use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    plonk::config::GenericConfig,
};

use crate::DAGState;

pub trait Functional<F, C, const D: usize, const N: usize>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    type Inputs;
    type Outputs;

    fn functional_inputs(&self) -> &Self::Inputs;
    fn call_compile(self, dag: &mut DAGState<F, C, D, N>) -> Self::Outputs;
}
