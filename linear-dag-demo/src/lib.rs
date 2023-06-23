use functional::Functional;
use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CommonCircuitData, VerifierOnlyCircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
};

use crate::expr::{CompileExpr, Expr};

pub mod expr;
pub mod functional;
pub mod insert_balance;
pub mod moving_average;
mod tests;
pub mod transfer;

#[derive(Clone)]
pub struct ProofData<F, C: GenericConfig<D, F = F>, const D: usize>
where
    F: RichField + Extendable<D>,
{
    pub proof_with_pis: ProofWithPublicInputs<F, C, D>,
    pub common: CommonCircuitData<F, D>,
    pub verifier_only: VerifierOnlyCircuitData<C, D>,
}

struct FillCircuit<F: Field + RichField + Extendable<D>, const D: usize> {
    circuit_builder: CircuitBuilder<F, D>,
    partial_witness: PartialWitness<F>,
}

pub trait Connector<F, C, const D: usize, const N: usize>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    Self: Sized,
{
    fn execution_step(&self) -> usize;
    fn prove_nth_execution<Func: Functional<F, C, D, N>>(
        self,
        functional: Func,
    ) -> Result<(Self, Func::Outputs), anyhow::Error>;
}

pub struct DAGState<F, C: GenericConfig<D, F = F>, const D: usize, const N: usize>
where
    F: Field + RichField + Extendable<D>,
{
    values: Vec<Expr<F, C, N, D>>,
    execution_step: usize,
    previous_execution_step_proof: Option<ProofData<F, C, D>>,
    to_fill_circuit: FillCircuit<F, D>,
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

impl<F, C, const D: usize, const N: usize> DAGState<F, C, D, N>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new(config: Option<CircuitConfig>) -> Self {
        let execution_step = 0;
        let previous_execution_step_proof = None;
        // TODO: should we change to a different default config type ?
        let config = config.unwrap_or(CircuitConfig::standard_recursion_config());

        let circuit_builder = CircuitBuilder::<F, D>::new(config);
        let partial_witness = PartialWitness::<F>::new();
        Self {
            execution_step,
            previous_execution_step_proof,
            to_fill_circuit: FillCircuit {
                circuit_builder,
                partial_witness,
            },
            values: vec![],
        }
    }

    pub fn previous_execution_step_proof(&self) -> Option<ProofData<F, C, D>> {
        self.previous_execution_step_proof.clone()
    }

    fn build_circuit(self) -> Result<ProofData<F, C, D>, anyhow::Error> {
        let circuit_builder = self.to_fill_circuit.circuit_builder;
        let partial_witness = self.to_fill_circuit.partial_witness;

        let circuit_data = circuit_builder.build::<C>();
        let proof_with_pis = circuit_data.prove(partial_witness)?;

        Ok(ProofData {
            proof_with_pis,
            common: circuit_data.common,
            verifier_only: circuit_data.verifier_only,
        })
    }
}

impl<F, C, const D: usize, const N: usize> Connector<F, C, D, N> for DAGState<F, C, D, N>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    fn execution_step(&self) -> usize {
        self.execution_step
    }
    fn prove_nth_execution<Func: Functional<F, C, D, N>>(
        mut self,
        functional: Func,
    ) -> Result<(Self, Func::Outputs), anyhow::Error> {
        let outputs = functional.call_compile(&mut self);
        let config = self.to_fill_circuit.circuit_builder.config.clone();

        // verifies previous execution step (recursively)
        if let Some(ProofData {
            ref common,
            ref verifier_only,
            ref proof_with_pis,
        }) = self.previous_execution_step_proof
        {
            let proof_with_pis_target = self
                .to_fill_circuit
                .circuit_builder
                .add_virtual_proof_with_pis(&common);
            let verifier_target = self
                .to_fill_circuit
                .circuit_builder
                .add_virtual_verifier_data(common.config.fri_config.cap_height);
            let () = self.to_fill_circuit.circuit_builder.verify_proof::<C>(
                &proof_with_pis_target,
                &verifier_target,
                common,
            );
            self.to_fill_circuit
                .partial_witness
                .set_proof_with_pis_target(&proof_with_pis_target, proof_with_pis);
            self.to_fill_circuit
                .partial_witness
                .set_verifier_data_target(&verifier_target, verifier_only);
        }

        // new proof data, for the current execution step
        let execution_step = self.execution_step + 1;
        let previous_values = self.values.clone();
        let proof_data = self.build_circuit()?;

        // Start afresh circuit builder
        let circuit_builder = CircuitBuilder::<F, D>::new(config);
        let partial_witness = PartialWitness::<F>::new();

        let mut dag = Self {
            execution_step: execution_step + 1,
            values: vec![],
            previous_execution_step_proof: Some(proof_data),
            to_fill_circuit: FillCircuit {
                circuit_builder,
                partial_witness,
            },
        };

        // add all necessary targets to the new dag circuit_builder
        dag.values = previous_values
            .iter()
            .map(|e| {
                <Expr<F, C, N, D> as CompileExpr<F, C, D, N>>::initialize_compile(
                    &mut dag,
                    e.evaluate(),
                )
            })
            .collect();

        Ok((dag, outputs))
    }
}
