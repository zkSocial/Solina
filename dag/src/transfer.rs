use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    plonk::config::GenericConfig,
};

use crate::{expr::Expr, functional::Functional, DAGGates};

pub struct TransferFunc {}

impl<F, C, const D: usize, const N: usize> Functional<F, C, D, N> for TransferFunc
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    type InputGates = Vec<(usize, Expr<F, C, N, D>)>;
    type OutputGates = Vec<(usize, Expr<F, C, N, D>)>;
    fn call_compile(
        dag: &mut DAGGates<F, C, D, N>,
        inputs: Self::InputGates,
        outputs: Self::OutputGates,
    ) {
        inputs.iter().for_each(|(i, e)| {
            let expr = &dag.gates[*i];
            match expr {
                Expr::FieldExpr(f) => {}
                _ => panic!("Invalid Expression"),
            }
        });
    }
}
