use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::config::{AlgebraicHasher, GenericConfig},
};

use crate::{
    expr::{ArrExpr, CompileExpr, Expr},
    functional::Functional,
    DAGGates,
};

const N_TRANSFER: usize = 3;
const N_DAG: usize = 2;

pub struct TransferExpr<F: Field> {
    from_expr: ArrExpr<F, N_DAG>,
    to_expr: ArrExpr<F, N_DAG>,
}
pub struct TransferFunc {}

impl<F, C, const D: usize> Functional<F, C, D, N_DAG> for TransferFunc
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    type InputGates = Vec<ArrExpr<F, N_TRANSFER>>;
    type OutputGates = Vec<TransferExpr<F>>;
    fn call_compile(
        dag: &mut DAGGates<F, C, D, N_DAG>,
        inputs: Self::InputGates,
    ) -> Self::OutputGates {
        let mut outputs = vec![];
        inputs.iter().for_each(|transfer_expr| {
            let (transfer_data, transfer_data_targets) = (
                transfer_expr.evaluate(),
                transfer_expr.compile(&mut dag.circuit_builder, &mut dag.partial_witness),
            );
            let from = transfer_data[0];
            let to = transfer_data[1];

            let should_be_from_target = dag.circuit_builder.constant(from);
            let should_be_to_target = dag.circuit_builder.constant(to);

            // enforce transfer account values on the inputs
            dag.circuit_builder
                .connect(should_be_from_target, transfer_data_targets[0]);
            dag.circuit_builder
                .connect(should_be_to_target, transfer_data_targets[1]);

            // TODO: error catch instead of panic
            let from_expr = get_account_data(&dag.gates, from);
            let to_expr = get_account_data(&dag.gates, to);

            let transfer_expr = transfer_circuit_logic(
                dag,
                from_expr,
                to_expr,
                transfer_data[2],
                transfer_data_targets[2],
                should_be_from_target,
                should_be_to_target,
            );
            outputs.push(transfer_expr);
        });
        outputs
    }
}

fn transfer_circuit_logic<F, C, const D: usize>(
    dag: &mut DAGGates<F, C, D, N_DAG>,
    mut from_expr: ArrExpr<F, N_DAG>,
    mut to_expr: ArrExpr<F, N_DAG>,
    transfer_value: F,
    transfer_value_target: Target,
    should_be_from_target: Target,
    should_be_to_target: Target,
) -> TransferExpr<F>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    let [from, from_balance] = from_expr.evaluate();
    let [from_target, from_balance_target] =
        from_expr.compile(&mut dag.circuit_builder, &mut dag.partial_witness);

    let [to, to_balance] = to_expr.evaluate();
    let [to_target, to_balance_target] =
        to_expr.compile(&mut dag.circuit_builder, &mut dag.partial_witness);

    // enforce transfer account values on the DAGState
    dag.circuit_builder
        .connect(should_be_from_target, from_target);
    dag.circuit_builder.connect(should_be_to_target, to_target);

    // TODO: do range check
    let new_from_balance = from_balance.sub(transfer_value);
    let should_be_new_from_balance_target = dag
        .circuit_builder
        .sub(from_balance_target, transfer_value_target);
    from_expr.update([from, new_from_balance]);
    let [new_from_target, new_from_balance_target] =
        from_expr.compile(&mut dag.circuit_builder, &mut dag.partial_witness);

    // enforce that new from target states were updated correctly
    dag.circuit_builder
        .connect(should_be_from_target, new_from_target);
    dag.circuit_builder
        .connect(new_from_balance_target, should_be_new_from_balance_target);

    // TODO: do range check
    let new_to_balance = to_balance.add(transfer_value);
    let should_be_new_to_balance_target = dag
        .circuit_builder
        .add(to_balance_target, transfer_value_target);
    to_expr.update([to, new_to_balance]);
    let [new_to_target, new_to_balance_target] =
        to_expr.compile(&mut dag.circuit_builder, &mut dag.partial_witness);

    // enforce taht new to target states were updated correctly
    dag.circuit_builder
        .connect(should_be_to_target, new_to_target);
    dag.circuit_builder
        .connect(new_to_balance_target, should_be_new_to_balance_target);

    TransferExpr { from_expr, to_expr }
}

fn get_account_data<F, C, const D: usize>(
    state: &[Expr<F, C, N_DAG, D>],
    account: F,
) -> ArrExpr<F, N_DAG>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    let account_data = state
        .into_iter()
        .filter_map(|s| {
            if let Expr::ArrExpr(s) = s {
                if s.evaluate()[0] == account {
                    Some(s.clone())
                } else {
                    None
                }
            } else {
                // return None instead of panicking
                None
            }
        })
        .collect::<Vec<_>>();

    if account_data.len() != 1 {
        panic!(
            "Panic: either non-existing or duplicate account state: {}",
            account
        );
    }
    account_data[0].clone()
}
