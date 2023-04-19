use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    plonk::config::GenericConfig,
};

use crate::DAGGates;

pub trait Functional<F, C, const D: usize, const N: usize>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    type InputGates;
    type OutputGates;

    fn call_compile(dag: &mut DAGGates<F, C, D, N>, inputs: Self::InputGates) -> Self::OutputGates;
}
