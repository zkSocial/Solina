use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{
            CommonCircuitData, VerifierCircuitData, VerifierCircuitTarget, VerifierOnlyCircuitData,
        },
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

pub trait CompileExpr<F: Field + RichField + Extendable<D>, const D: usize> {
    type Targets;
    type Inner;
    fn evaluate(&self) -> Self::Inner;
    fn update(&mut self, new_val: Self::Inner);
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets;
}

#[derive(Clone, PartialEq, Eq)]
pub struct FieldExpr<F: Field> {
    element: F,
}

impl<F: Field + RichField + Extendable<D>, const D: usize> CompileExpr<F, D> for FieldExpr<F> {
    type Targets = Target;
    type Inner = F;
    fn evaluate(&self) -> Self::Inner {
        self.element
    }
    fn update(&mut self, new_val: Self::Inner) {
        self.element = new_val;
    }
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

#[derive(Clone, PartialEq, Eq)]
pub struct ArrExpr<F: Field, const N: usize> {
    expr: [F; N],
}

impl<F: Field + RichField + Extendable<D>, const N: usize, const D: usize> CompileExpr<F, D>
    for ArrExpr<F, N>
{
    type Targets = [Target; N];
    type Inner = [F; N];
    fn evaluate(&self) -> Self::Inner {
        self.expr
    }
    fn update(&mut self, new_val: Self::Inner) {
        self.expr = new_val;
    }
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

#[derive(Clone, PartialEq, Eq)]
pub struct HashExpr<F: Field> {
    hash: HashOut<F>,
}

impl<F: Field + RichField + Extendable<D>, const D: usize> CompileExpr<F, D> for HashExpr<F> {
    type Targets = HashOutTarget;
    type Inner = HashOut<F>;
    fn evaluate(&self) -> Self::Inner {
        self.hash
    }
    fn update(&mut self, new_val: Self::Inner) {
        self.hash = new_val;
    }
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

#[derive(Clone, PartialEq, Eq)]
pub struct MultiHashExpr<F: Field> {
    hashes: Vec<HashOut<F>>,
}

impl<F: Field + RichField + Extendable<D>, const D: usize> CompileExpr<F, D> for MultiHashExpr<F> {
    type Targets = Vec<HashOutTarget>;
    type Inner = Vec<HashOut<F>>;
    fn evaluate(&self) -> Self::Inner {
        self.hashes.clone()
    }
    fn update(&mut self, new_val: Self::Inner) {
        self.hashes = new_val;
    }
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

#[derive(Clone, PartialEq, Eq)]
pub struct BoolExpr {
    b: bool,
}

impl<F: Field + RichField + Extendable<D>, const D: usize> CompileExpr<F, D> for BoolExpr {
    type Targets = BoolTarget;
    type Inner = bool;
    fn evaluate(&self) -> Self::Inner {
        self.b
    }
    fn update(&mut self, new_val: Self::Inner) {
        self.b = new_val;
    }
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

#[derive(Clone, PartialEq, Eq)]
pub struct BoolArrExpr<const N: usize> {
    b_arr: [bool; N],
}

impl<F: Field + RichField + Extendable<D>, const D: usize, const N: usize> CompileExpr<F, D>
    for BoolArrExpr<N>
{
    type Targets = [BoolTarget; N];
    type Inner = [bool; N];
    fn evaluate(&self) -> Self::Inner {
        self.b_arr
    }
    fn update(&mut self, new_val: Self::Inner) {
        self.b_arr = new_val;
    }
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

#[derive(Clone, PartialEq, Eq)]
pub struct ProofData<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    proof_with_pis: ProofWithPublicInputs<F, C, D>,
    common_circuit_data: CommonCircuitData<F, D>,
    verifier_only_data: VerifierOnlyCircuitData<C, D>,
}

impl<F, C, const D: usize> CompileExpr<F, D> for ProofData<F, C, D>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    type Targets = (ProofWithPublicInputsTarget<D>, VerifierCircuitTarget);
    type Inner = (CommonCircuitData<F, D>, VerifierOnlyCircuitData<C, D>);
    fn evaluate(&self) -> Self::Inner {
        (
            self.common_circuit_data.clone(),
            self.verifier_only_data.clone(),
        )
    }
    fn update(&mut self, new_val: Self::Inner) {
        unimplemented!("Update is not implemented for recursive proofs");
    }
    fn compile(
        &self,
        circuit_builder: &mut CircuitBuilder<F, D>,
        partial_witness: &mut PartialWitness<F>,
    ) -> Self::Targets {
        let proof_with_pis_targets =
            circuit_builder.add_virtual_proof_with_pis(&self.common_circuit_data);
        partial_witness.set_proof_with_pis_target(&proof_with_pis_targets, &self.proof_with_pis);
        let verify_target = circuit_builder
            .add_virtual_verifier_data(self.common_circuit_data.config.fri_config.cap_height);
        partial_witness.set_verifier_data_target(&verify_target, &self.verifier_only_data);
        (proof_with_pis_targets, verify_target)
    }
}

#[derive(Clone, PartialEq, Eq)]
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

pub enum ExprTargets<const N: usize, const D: usize> {
    FieldTarget(Target),
    ArrTarget([Target; N]),
    HashTarget(HashOutTarget),
    MultiHashTarget(Vec<HashOutTarget>),
    BoolTarget(BoolTarget),
    BoolArrTarget([BoolTarget; N]),
    ProofDataTargets((ProofWithPublicInputsTarget<D>, VerifierCircuitTarget)),
}

pub enum ExprValue<
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const N: usize,
    const D: usize,
> {
    Field(F),
    Arr([F; N]),
    Hash(HashOut<F>),
    MultiHash(Vec<HashOut<F>>),
    Bool(bool),
    BoolArr([bool; N]),
    ProofData((CommonCircuitData<F, D>, VerifierOnlyCircuitData<C, D>)),
}

impl<F, C, const D: usize, const N: usize> CompileExpr<F, D> for Expr<F, C, N, D>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    type Targets = ExprTargets<N, D>;
    type Inner = ExprValue<F, C, N, D>;
    fn evaluate(&self) -> Self::Inner {
        match self {
            Expr::FieldExpr(f) => ExprValue::Field(f.element),
            Expr::ArrExpr(a) => ExprValue::Arr(a.expr),
            Expr::HashExpr(h) => ExprValue::Hash(h.hash),
            Expr::MultiHashExpr(h) => ExprValue::MultiHash(h.hashes.clone()),
            Expr::BoolExpr(b) => ExprValue::Bool(b.b),
            Expr::BoolArrExpr(b) => ExprValue::BoolArr(b.b_arr),
            Expr::ProofData(p) => {
                ExprValue::ProofData((p.common_circuit_data.clone(), p.verifier_only_data.clone()))
            }
        }
    }
    fn update(&mut self, new_val: Self::Inner) {
        match self {
            Expr::FieldExpr(f) => {
                if let ExprValue::Field(other_f) = new_val {
                    f.update(other_f);
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::ArrExpr(a) => {
                if let ExprValue::Arr(other_a) = new_val {
                    a.update(other_a);
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::HashExpr(h) => {
                if let ExprValue::Hash(other_h) = new_val {
                    h.update(other_h);
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::MultiHashExpr(h) => {
                if let ExprValue::MultiHash(other_h) = new_val {
                    h.update(other_h);
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::BoolExpr(b) => {
                if let ExprValue::Bool(other_b) = new_val {
                    <BoolExpr as CompileExpr<F, D>>::update(b, other_b);
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::BoolArrExpr(b) => {
                if let ExprValue::BoolArr(other_b) = new_val {
                    <BoolArrExpr<N> as CompileExpr<F, D>>::update(b, other_b);
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::ProofData(..) => {
                unimplemented!("Cannot update proof data")
            }
        }
    }
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
