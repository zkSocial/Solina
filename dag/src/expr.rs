use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{VerifierCircuitData, VerifierCircuitTarget},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

pub trait CompileExpr<F: Field + RichField + Extendable<D>, const D: usize> {
    type Targets;
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets;
}

#[derive(PartialEq, Eq)]
pub struct FieldExpr<F: Field> {
    element: F,
}

impl<F: Field + RichField + Extendable<D>, const D: usize> CompileExpr<F, D> for FieldExpr<F> {
    type Targets = Target;
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        let target = circuit_builder.add_virtual_target();
        partial_witness.set_target(target, self.element);
        target
    }
}

#[derive(PartialEq, Eq)]
pub struct ArrExpr<F: Field, const N: usize> {
    expr: [F; N],
}

impl<F: Field + RichField + Extendable<D>, const N: usize, const D: usize> CompileExpr<F, D>
    for ArrExpr<F, N>
{
    type Targets = [Target; N];
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        let targets = circuit_builder.add_virtual_target_arr::<N>();
        partial_witness.set_target_arr(targets, self.expr);
        targets
    }
}

#[derive(PartialEq, Eq)]
pub struct HashExpr<F: Field> {
    hash: HashOut<F>,
}

impl<F: Field + RichField + Extendable<D>, const D: usize> CompileExpr<F, D> for HashExpr<F> {
    type Targets = HashOutTarget;
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        let hash_targets = circuit_builder.add_virtual_hash();
        partial_witness.set_hash_target(hash_targets, self.hash);
        hash_targets
    }
}

#[derive(PartialEq, Eq)]
pub struct MultiHashExpr<F: Field> {
    hashes: Vec<HashOut<F>>,
}

impl<F: Field + RichField + Extendable<D>, const D: usize> CompileExpr<F, D> for MultiHashExpr<F> {
    type Targets = Vec<HashOutTarget>;
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        let hash_targets = circuit_builder.add_virtual_hashes(self.hashes.len());
        hash_targets
            .iter()
            .zip(&self.hashes)
            .for_each(|(t, h)| partial_witness.set_hash_target(*t, *h));
        hash_targets
    }
}

#[derive(PartialEq, Eq)]
pub struct BoolExpr {
    b: bool,
}

impl<F: Field + RichField + Extendable<D>, const D: usize> CompileExpr<F, D> for BoolExpr {
    type Targets = BoolTarget;
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        let bool_target = circuit_builder.add_virtual_bool_target_safe();
        partial_witness.set_bool_target(bool_target, self.b);
        bool_target
    }
}

#[derive(PartialEq, Eq)]
pub struct BoolArrExpr<const N: usize> {
    b_arr: [bool; N],
}

impl<F: Field + RichField + Extendable<D>, const D: usize, const N: usize> CompileExpr<F, D>
    for BoolArrExpr<N>
{
    type Targets = [BoolTarget; N];
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        let bool_targets = [0; N].map(|_| circuit_builder.add_virtual_bool_target_safe());
        [0; N]
            .into_iter()
            .for_each(|i| partial_witness.set_bool_target(bool_targets[i], self.b_arr[i]));
        bool_targets
    }
}

pub struct ProofData<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    proof_with_pis: ProofWithPublicInputs<F, C, D>,
    verify_data: VerifierCircuitData<F, C, D>,
}

impl<F, C, const D: usize> CompileExpr<F, D> for ProofData<F, C, D>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    type Targets = (ProofWithPublicInputsTarget<D>, VerifierCircuitTarget);

    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        let proof_with_pis_targets =
            circuit_builder.add_virtual_proof_with_pis(&self.verify_data.common);
        partial_witness.set_proof_with_pis_target(&proof_with_pis_targets, &self.proof_with_pis);
        let verify_target = circuit_builder
            .add_virtual_verifier_data(self.verify_data.common.config.fri_config.cap_height);
        partial_witness.set_verifier_data_target(&verify_target, &self.verify_data.verifier_only);
        (proof_with_pis_targets, verify_target)
    }
}

pub enum Expr<
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const N: usize,
    const D: usize,
> {
    FieldExpr(FieldExpr<F>),
    ArrExpr(ArrExpr<F, N>),
    HashExpr(HashExpr<F>),
    MultiHashExpr(MultiHashExpr<F>),
    BoolExpr(BoolExpr),
    BoolArrExpr(BoolArrExpr<N>),
    ProofData(ProofData<F, C, D>),
}

impl<F, C, const N: usize, const D: usize> PartialEq for Expr<F, C, N, D>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::FieldExpr(f) => {
                if let Self::FieldExpr(other_f) = other {
                    return other_f == f;
                } else {
                    return false;
                }
            }
            Self::ArrExpr(a) => {
                if let Self::ArrExpr(other_a) = other {
                    return other_a == a;
                } else {
                    return false;
                }
            }
            Self::HashExpr(h) => {
                if let Self::HashExpr(other_h) = other {
                    return other_h == h;
                } else {
                    return false;
                }
            }
            Self::MultiHashExpr(h) => {
                if let Self::MultiHashExpr(other_h) = other {
                    return other_h == h;
                } else {
                    return false;
                }
            }
            Self::BoolExpr(b) => {
                if let Self::BoolExpr(other_b) = other {
                    return other_b == b;
                } else {
                    return false;
                }
            }
            Self::BoolArrExpr(b) => {
                if let Self::BoolArrExpr(other_b) = other {
                    return other_b == b;
                } else {
                    return false;
                }
            }
            Self::ProofData(_) => {
                return false;
            }
        }
    }
}

pub enum ExprTargets<const N: usize, const D: usize> {
    FieldTarget(Target),
    ArrTarget([Target; N]),
    HashTarget(HashOutTarget),
    MultiHashTarget(Vec<HashOutTarget>),
    BoolTarget(BoolTarget),
    BoolArrTarget([BoolTarget; N]),
    ProofDataTargets((ProofWithPublicInputsTarget<D>, VerifierCircuitTarget)),
}

impl<F, C, const D: usize, const N: usize> CompileExpr<F, D> for Expr<F, C, N, D>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    type Targets = ExprTargets<N, D>;
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        match self {
            Self::FieldExpr(f) => {
                ExprTargets::FieldTarget(f.compile(circuit_builder, partial_witness))
            }
            Self::ArrExpr(a) => ExprTargets::ArrTarget(a.compile(circuit_builder, partial_witness)),
            Self::HashExpr(h) => {
                ExprTargets::HashTarget(h.compile(circuit_builder, partial_witness))
            }
            Self::MultiHashExpr(h) => {
                ExprTargets::MultiHashTarget(h.compile(circuit_builder, partial_witness))
            }
            Self::BoolExpr(b) => {
                ExprTargets::BoolTarget(b.compile(circuit_builder, partial_witness))
            }
            Self::BoolArrExpr(b) => {
                ExprTargets::BoolArrTarget(b.compile(circuit_builder, partial_witness))
            }
            Self::ProofData(p) => {
                ExprTargets::ProofDataTargets(p.compile(circuit_builder, partial_witness))
            }
        }
    }
}
