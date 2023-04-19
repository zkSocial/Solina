use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    plonk::config::GenericConfig,
};

use crate::{
    expr::{ArrExpr, CompileExpr, Expr},
    functional::Functional,
};

const N: usize = 2;

pub struct InsertFunctional {}

impl<F, C, const D: usize> Functional<F, C, D, N> for InsertFunctional
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    type InputGates = Vec<ArrExpr<F, N>>;
    type OutputGates = ();
    fn call_compile(dag: &mut crate::DAGGates<F, C, D, N>, inputs: Self::InputGates) {
        inputs.iter().for_each(|i| {
            dag.gates.push(Expr::ArrExpr(i.clone()));
            let targets = i.compile(&mut dag.circuit_builder, &mut dag.partial_witness);
        });
    }
}
