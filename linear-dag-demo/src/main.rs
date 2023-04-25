use clap::{self, Parser};
use linear_dag_demo::{moving_average::MovingAverageDAGBuilder, DAGState};
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
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
    let moving_average_builder: DAGState<F, C, D, N> =
        MovingAverageDAGBuilder::initialize_with_values(values);
}
