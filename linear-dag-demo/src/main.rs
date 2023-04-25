use std::{fs::File, io::Write};

use clap::{self, Parser};
use linear_dag_demo::{
    moving_average::{MovingAverageDAGBuilder, MovingAverageFunc},
    Connector, DAGState,
};
use plonky2::{
    field::{
        goldilocks_field::GoldilocksField,
        types::{Field, PrimeField64},
    },
    plonk::config::PoseidonGoldilocksConfig,
};

const D: usize = 2;
const N: usize = 1;
type C = PoseidonGoldilocksConfig;
type F = GoldilocksField;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Initial values, we set it to optional as this might
    #[clap(short, long)]
    values: Vec<u64>,
}

fn main() {
    let cli = Cli::parse();
    let values = cli
        .values
        .iter()
        .map(|x| F::from_canonical_u64(*x))
        .collect();
    let (dag, output): (DAGState<F, C, D, N>, F) =
        MovingAverageDAGBuilder::initialize_with_values(values);

    let proof = dag
        .previous_execution_step_proof()
        .expect("Should have available proof");

    let mut proof_file = File::create("proof_file.txt").expect("Failed to create file");
    let mut output_file = File::create("output_file.txt").expect("Failed to create file");

    proof_file
        .write(proof.proof_with_pis.to_bytes().as_ref())
        .expect("Failed to write proof to file");
    output_file
        .write(&output.to_canonical_u64().to_le_bytes())
        .expect("Failed to write output to file");

    let moving_average_functional = MovingAverageFunc {
        value: Some(F::from_canonical_u64(10)),
    };

    let (dag, output) = dag
        .prove_nth_execution(moving_average_functional)
        .expect("Failed to prove nth step");

    let proof = dag
        .previous_execution_step_proof()
        .expect("Should have available proof");

    proof_file
        .write(proof.proof_with_pis.to_bytes().as_ref())
        .expect("Failed to write proof to file");
    output_file
        .write(&output.to_canonical_u64().to_le_bytes())
        .expect("Failed to write output to file");
}
