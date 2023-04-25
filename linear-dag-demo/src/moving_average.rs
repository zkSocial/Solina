use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    iop::witness::PartialWitness,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::{AlgebraicHasher, GenericConfig},
    },
};

use crate::{
    expr::{CompileExpr, Expr, FieldExpr},
    functional::Functional,
    Connector, DAGState,
};

pub struct MovingAverageFunc<F: Field> {
    pub value: Option<F>,
}

impl<F, C, const D: usize, const N: usize> Functional<F, C, D, N> for MovingAverageFunc<F>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    type Inputs = F;
    type Outputs = F;

    fn functional_inputs(&self) -> Self::Inputs {
        self.value.unwrap_or(F::ZERO)
    }

    fn call_compile(self, dag: &mut DAGState<F, C, D, N>) -> F {
        let mut sum = F::ZERO;
        let mut sum_target = dag.to_fill_circuit.circuit_builder.zero();
        let mut size = 0;
        let mut size_target = dag.to_fill_circuit.circuit_builder.zero();

        let one_target = dag.to_fill_circuit.circuit_builder.one();

        if dag.execution_step > 0 {
            let input = self
                .value
                .expect("Value should be provided at this point of execution > 0");
            let input_expr = FieldExpr::initialize_compile(dag, input);

            let _ = dag.values.remove(0);
            dag.values.push(Expr::FieldExpr(input_expr));
        }
        for expr in dag.values.iter() {
            if let Expr::FieldExpr(expr) = expr {
                sum_target = dag.to_fill_circuit.circuit_builder.add(
                    sum_target,
                    <FieldExpr<F> as CompileExpr<F, C, D, N>>::targets(&expr),
                );
                sum += <FieldExpr<F> as CompileExpr<F, C, D, N>>::evaluate(&expr);
                size += 1;
                size_target = dag
                    .to_fill_circuit
                    .circuit_builder
                    .add(size_target, one_target);
            } else {
                panic!("Invalid moving average expression type");
            }
        }

        let moving_average = sum / F::from_canonical_usize(size);
        let _moving_average_target = dag
            .to_fill_circuit
            .circuit_builder
            .div(sum_target, size_target);

        moving_average
    }
}

pub struct MovingAverageDAGBuilder {}

impl MovingAverageDAGBuilder {
    pub fn initialize_with_values<F, C, const D: usize, const N: usize>(
        values: Vec<F>,
    ) -> (DAGState<F, C, D, N>, F)
    where
        F: Field + RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        C::Hasher: AlgebraicHasher<F>,
    {
        let config = CircuitConfig::standard_recursion_config();
        let circuit_builder = CircuitBuilder::<F, D>::new(config);
        let partial_witness = PartialWitness::<F>::new();

        let mut dag = DAGState {
            execution_step: 0,
            values: vec![],
            previous_execution_step_proof: None,
            to_fill_circuit: crate::FillCircuit {
                circuit_builder,
                partial_witness,
            },
        };

        if values.len() == 0 {
            panic!("Unable to define DAG State for an empty moving average");
        }
        let values = values
            .iter()
            .map(|val| Expr::FieldExpr(FieldExpr::initialize_compile(&mut dag, *val)))
            .collect();
        dag.values = values;

        let moving_average_functional = MovingAverageFunc { value: None };
        let (dag, output) = dag
            .prove_nth_execution::<MovingAverageFunc<F>>(moving_average_functional)
            .expect("Proving execution at first step failed");

        (dag, output)
    }
}
