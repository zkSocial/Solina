use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};

use crate::expr::Expr;

pub mod config;
pub mod expr;
pub mod functional;
pub mod input;
pub mod job;
pub mod session;
pub mod transfer;

pub(crate) const U64_BYTES_LEN: usize = 8;
pub(crate) type QmHashBytes = [u8; 32];

pub trait Connector<F, C, const D: usize, const N: usize>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    type InputGates;
    type OutputGates;

    fn connect_input_output(&mut self);
}

pub struct DAGGates<F, C: GenericConfig<D, F = F>, const D: usize, const N: usize>
where
    F: Field + RichField + Extendable<D>,
{
    circuit_builder: CircuitBuilder<F, D>,
    partial_witness: PartialWitness<F>,
    gates: Vec<Expr<F, C, N, D>>,
}

pub enum DAGKey<'a, F, C, const D: usize, const N: usize>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    Exist(&'a Expr<F, C, N, D>),
    DoesNotExist,
    Index(usize),
}

impl<F, C, const D: usize, const N: usize> DAGGates<F, C, D, N>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub fn new() -> Self {
        let config = CircuitConfig::standard_recursion_config();
        let circuit_builder = CircuitBuilder::<F, D>::new(config);
        let partial_witness = PartialWitness::<F>::new();
        Self {
            circuit_builder,
            partial_witness,
            gates: vec![],
        }
    }

    pub fn read_gate(&self, index: usize) -> DAGKey<F, C, D, N> {
        if index >= self.gates.len() {
            return DAGKey::DoesNotExist;
        }
        let value = &self.gates[index];
        DAGKey::Exist(value)
    }

    pub fn add_gate(&mut self, value: Expr<F, C, N, D>) -> DAGKey<F, C, D, N> {
        let index = if let Some(index) = self.gates.iter().position(|x| x == &value) {
            self.gates[index] = value;
            index
        } else {
            self.gates.push(value);
            self.gates.len() - 1
        };
        DAGKey::Index(index)
    }
}

impl<F, C, const D: usize, const N: usize> Connector<F, C, D, N> for DAGGates<F, C, D, N>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    type InputGates = Expr<F, C, N, D>;
    type OutputGates = Expr<F, C, N, D>;

    fn connect_input_output(&mut self) {}
}
