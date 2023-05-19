// use plonky2::{
//     field::{extension::Extendable, types::Field},
//     hash::hash_types::RichField,
//     plonk::config::GenericConfig,
// };

// use crate::{
//     expr::{ArrExpr, CompileExpr, Expr},
//     functional::Functional,
//     DAGState,
// };

// const N: usize = 2;

// pub struct InsertFunctional<F: Field> {
//     inputs: Vec<[F; N]>,
// }

// impl<F, C, const D: usize> Functional<F, C, D, N> for InsertFunctional<F>
// where
//     F: Field + RichField + Extendable<D>,
//     C: GenericConfig<D, F = F>,
// {
//     type Inputs = Vec<[F; N]>;
//     type Outputs = ();
//     fn functional_inputs(&self) -> Self::Inputs {
//         self.inputs
//     }
//     fn call_compile(self, dag: &mut DAGState<F, C, D, N>) {
//         self.inputs.iter().for_each(|i| {
//             let expr = ArrExpr::initialize_compile(dag, *i);
//             dag.values.push(Expr::ArrExpr(expr));
//         });
//     }
// }
