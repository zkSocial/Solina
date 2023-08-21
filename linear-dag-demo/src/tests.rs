use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    plonk::config::PoseidonGoldilocksConfig,
};

use crate::{
    moving_average::{self, MovingAverageDAGBuilder, MovingAverageFunc},
    Connector, DAGState,
};

type F = GoldilocksField;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;
const N: usize = 1;

#[test]
fn it_works_moving_average() {
    let values = vec![F::ONE, F::from_canonical_u64(2), F::from_canonical_u64(3)];
    let (dag, output): (DAGState<F, C, D, N>, F) =
        MovingAverageDAGBuilder::initialize_with_values(values);

    assert_eq!(output, F::from_canonical_u64(2));

    let moving_average_functional = MovingAverageFunc {
        value: Some(F::from_canonical_u64(7)),
    };

    let (dag, output) = dag
        .prove_nth_execution(moving_average_functional)
        .expect("Failed to prove 2nd step");

    assert_eq!(output, F::from_canonical_u64(4));

    let moving_average_functional = MovingAverageFunc {
        value: Some(F::from_canonical_u64(5)),
    };

    let (_dag, output) = dag
        .prove_nth_execution(moving_average_functional)
        .expect("Failed to prove 3rd step");

    assert_eq!(output, F::from_canonical_u64(5));
}
