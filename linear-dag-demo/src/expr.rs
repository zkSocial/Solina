use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{
        target::{BoolTarget, Target},
        witness::WitnessWrite,
    },
    plonk::{
        circuit_data::{CommonCircuitData, VerifierCircuitTarget, VerifierOnlyCircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

use crate::DAGState;

pub trait CompileExpr<
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const N: usize,
> where
    Self: Sized,
{
    type Targets;
    type Inner;
    fn evaluate(&self) -> Self::Inner;
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self;
    fn update_compile(
        &mut self,
        dag: &mut DAGState<F, C, D, N>,
        new_val: Self::Inner,
    ) -> Self::Targets;
    fn targets(&self) -> Self::Targets;
}

#[derive(Clone)]
pub struct FieldExpr<F: Field> {
    element: F,
    target: Target,
}

impl<
        F: Field + RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const N: usize,
    > CompileExpr<F, C, D, N> for FieldExpr<F>
{
    type Targets = Target;
    type Inner = F;
    fn evaluate(&self) -> Self::Inner {
        self.element
    }
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self {
        let target = dag.to_fill_circuit.circuit_builder.add_virtual_target();
        dag.to_fill_circuit.partial_witness.set_target(target, val);
        let expr = Self {
            element: val,
            target,
        };
        expr
    }
    fn update_compile(
        &mut self,
        dag: &mut DAGState<F, C, D, N>,
        new_val: Self::Inner,
    ) -> Self::Targets {
        let target = dag.to_fill_circuit.circuit_builder.add_virtual_target();
        dag.to_fill_circuit
            .partial_witness
            .set_target(target, new_val);
        self.element = new_val;
        target
    }
    fn targets(&self) -> Self::Targets {
        self.target
    }
}

#[derive(Clone)]
pub struct ArrExpr<F: Field, const N: usize> {
    expr: [F; N],
    targets: [Target; N],
}

impl<
        F: Field + RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const N: usize,
        const D: usize,
    > CompileExpr<F, C, D, N> for ArrExpr<F, N>
{
    type Targets = [Target; N];
    type Inner = [F; N];
    fn evaluate(&self) -> Self::Inner {
        self.expr
    }
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self {
        let targets = dag
            .to_fill_circuit
            .circuit_builder
            .add_virtual_target_arr::<N>();
        dag.to_fill_circuit
            .partial_witness
            .set_target_arr(targets, val);
        let expr = Self { expr: val, targets };
        expr
    }
    fn update_compile(
        &mut self,
        dag: &mut DAGState<F, C, D, N>,
        new_val: Self::Inner,
    ) -> Self::Targets {
        let targets = dag
            .to_fill_circuit
            .circuit_builder
            .add_virtual_target_arr::<N>();
        dag.to_fill_circuit
            .partial_witness
            .set_target_arr(targets, self.expr);
        self.expr = new_val;
        targets
    }
    fn targets(&self) -> Self::Targets {
        self.targets
    }
}

#[derive(Clone)]
pub struct HashExpr<F: Field> {
    hash: HashOut<F>,
    hash_out_target: HashOutTarget,
}

impl<
        F: Field + RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const N: usize,
    > CompileExpr<F, C, D, N> for HashExpr<F>
{
    type Targets = HashOutTarget;
    type Inner = HashOut<F>;
    fn evaluate(&self) -> Self::Inner {
        self.hash
    }
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self {
        let hash_out_target = dag.to_fill_circuit.circuit_builder.add_virtual_hash();
        dag.to_fill_circuit
            .partial_witness
            .set_hash_target(hash_out_target, val);
        let expr = Self {
            hash: val,
            hash_out_target,
        };
        expr
    }
    fn update_compile(
        &mut self,
        dag: &mut DAGState<F, C, D, N>,
        new_val: Self::Inner,
    ) -> Self::Targets {
        let hash_targets = dag.to_fill_circuit.circuit_builder.add_virtual_hash();
        dag.to_fill_circuit
            .partial_witness
            .set_hash_target(hash_targets, self.hash);
        self.hash = new_val;
        hash_targets
    }
    fn targets(&self) -> Self::Targets {
        self.hash_out_target
    }
}

#[derive(Clone)]
pub struct MultiHashExpr<F: Field> {
    hashes: Vec<HashOut<F>>,
    hash_out_targets: Vec<HashOutTarget>,
}

impl<
        F: Field + RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const N: usize,
    > CompileExpr<F, C, D, N> for MultiHashExpr<F>
{
    type Targets = Vec<HashOutTarget>;
    type Inner = Vec<HashOut<F>>;
    fn evaluate(&self) -> Self::Inner {
        self.hashes.clone()
    }
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self {
        let hash_out_targets = dag
            .to_fill_circuit
            .circuit_builder
            .add_virtual_hashes(val.len());
        hash_out_targets
            .iter()
            .zip(&val)
            .for_each(|(t, h)| dag.to_fill_circuit.partial_witness.set_hash_target(*t, *h));
        let expr = Self {
            hashes: val,
            hash_out_targets,
        };
        expr
    }
    fn update_compile(
        &mut self,
        dag: &mut DAGState<F, C, D, N>,
        new_val: Self::Inner,
    ) -> Self::Targets {
        let hash_targets = dag
            .to_fill_circuit
            .circuit_builder
            .add_virtual_hashes(new_val.len());
        hash_targets
            .iter()
            .zip(&new_val)
            .for_each(|(t, h)| dag.to_fill_circuit.partial_witness.set_hash_target(*t, *h));
        self.hashes = new_val;
        hash_targets
    }
    fn targets(&self) -> Self::Targets {
        self.hash_out_targets.clone()
    }
}

#[derive(Clone)]
pub struct BoolExpr {
    b: bool,
    bool_target: BoolTarget,
}

impl<
        F: Field + RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const N: usize,
    > CompileExpr<F, C, D, N> for BoolExpr
{
    type Targets = BoolTarget;
    type Inner = bool;
    fn evaluate(&self) -> Self::Inner {
        self.b
    }
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self {
        let bool_target = dag
            .to_fill_circuit
            .circuit_builder
            .add_virtual_bool_target_safe();
        dag.to_fill_circuit
            .partial_witness
            .set_bool_target(bool_target, val);
        let expr = Self {
            b: val,
            bool_target,
        };
        expr
    }
    fn update_compile(
        &mut self,
        dag: &mut DAGState<F, C, D, N>,
        new_val: Self::Inner,
    ) -> Self::Targets {
        let bool_target = dag
            .to_fill_circuit
            .circuit_builder
            .add_virtual_bool_target_safe();
        dag.to_fill_circuit
            .partial_witness
            .set_bool_target(bool_target, new_val);
        self.b = new_val;
        bool_target
    }
    fn targets(&self) -> Self::Targets {
        self.bool_target
    }
}

#[derive(Clone)]
pub struct BoolArrExpr<const N: usize> {
    b_arr: [bool; N],
    bool_targets: [BoolTarget; N],
}

impl<
        F: Field + RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const N: usize,
    > CompileExpr<F, C, D, N> for BoolArrExpr<N>
{
    type Targets = [BoolTarget; N];
    type Inner = [bool; N];
    fn evaluate(&self) -> Self::Inner {
        self.b_arr
    }
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self {
        let bool_targets = [0; N].map(|_| {
            dag.to_fill_circuit
                .circuit_builder
                .add_virtual_bool_target_safe()
        });
        [0; N].into_iter().for_each(|i| {
            dag.to_fill_circuit
                .partial_witness
                .set_bool_target(bool_targets[i], val[i])
        });
        let expr = Self {
            b_arr: val,
            bool_targets,
        };
        expr
    }
    fn update_compile(
        &mut self,
        dag: &mut DAGState<F, C, D, N>,
        new_val: Self::Inner,
    ) -> Self::Targets {
        let bool_targets = [0; N].map(|_| {
            dag.to_fill_circuit
                .circuit_builder
                .add_virtual_bool_target_safe()
        });
        [0; N].into_iter().for_each(|i| {
            dag.to_fill_circuit
                .partial_witness
                .set_bool_target(bool_targets[i], new_val[i])
        });
        self.b_arr = new_val;
        bool_targets
    }
    fn targets(&self) -> Self::Targets {
        self.bool_targets
    }
}

#[derive(Clone)]
pub struct ProofExpr<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    proof_with_pis: ProofWithPublicInputs<F, C, D>,
    common_circuit_data: CommonCircuitData<F, D>,
    verifier_only_data: VerifierOnlyCircuitData<C, D>,
    proof_with_pis_targets: ProofWithPublicInputsTarget<D>,
    verify_target: VerifierCircuitTarget,
}

impl<F, C, const D: usize, const N: usize> CompileExpr<F, C, D, N> for ProofExpr<F, C, D>
where
    F: Field + RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    type Targets = (ProofWithPublicInputsTarget<D>, VerifierCircuitTarget);
    type Inner = (
        ProofWithPublicInputs<F, C, D>,
        CommonCircuitData<F, D>,
        VerifierOnlyCircuitData<C, D>,
    );
    fn evaluate(&self) -> Self::Inner {
        (
            self.proof_with_pis.clone(),
            self.common_circuit_data.clone(),
            self.verifier_only_data.clone(),
        )
    }
    fn update_compile(
        &mut self,
        _dag: &mut DAGState<F, C, D, N>,
        _new_val: Self::Inner,
    ) -> Self::Targets {
        unimplemented!("Update is not implemented for recursive proofs");
    }
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self {
        let (proof_with_pis, common_circuit_data, verifier_only_data) = val;
        let proof_with_pis_targets = dag
            .to_fill_circuit
            .circuit_builder
            .add_virtual_proof_with_pis(&common_circuit_data);
        dag.to_fill_circuit
            .partial_witness
            .set_proof_with_pis_target(&proof_with_pis_targets, &proof_with_pis);
        let verify_target = dag
            .to_fill_circuit
            .circuit_builder
            .add_virtual_verifier_data(common_circuit_data.config.fri_config.cap_height);
        dag.to_fill_circuit
            .partial_witness
            .set_verifier_data_target(&verify_target, &verifier_only_data);
        let expr = Self {
            proof_with_pis,
            common_circuit_data,
            verifier_only_data,
            proof_with_pis_targets,
            verify_target,
        };
        expr
    }
    fn targets(&self) -> Self::Targets {
        (
            self.proof_with_pis_targets.clone(),
            self.verify_target.clone(),
        )
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
    ProofExpr(ProofExpr<F, C, D>),
}

pub enum ExprTargets<const N: usize, const D: usize> {
    FieldTarget(Target),
    ArrTarget([Target; N]),
    HashTarget(HashOutTarget),
    MultiHashTarget(Vec<HashOutTarget>),
    BoolTarget(BoolTarget),
    BoolArrTarget([BoolTarget; N]),
    ProofExprTargets((ProofWithPublicInputsTarget<D>, VerifierCircuitTarget)),
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
    ProofExpr(
        (
            ProofWithPublicInputs<F, C, D>,
            CommonCircuitData<F, D>,
            VerifierOnlyCircuitData<C, D>,
        ),
    ),
}

impl<F, C, const D: usize, const N: usize> CompileExpr<F, C, D, N> for Expr<F, C, N, D>
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
            Expr::ProofExpr(p) => ExprValue::ProofExpr((
                p.proof_with_pis.clone(),
                p.common_circuit_data.clone(),
                p.verifier_only_data.clone(),
            )),
        }
    }
    fn initialize_compile(dag: &mut DAGState<F, C, D, N>, val: Self::Inner) -> Self {
        match val {
            ExprValue::Field(f) => {
                let expr = FieldExpr::initialize_compile(dag, f);
                Expr::FieldExpr(expr)
            }
            ExprValue::Arr(a) => {
                let expr = ArrExpr::initialize_compile(dag, a);
                Expr::ArrExpr(expr)
            }
            ExprValue::Hash(h) => {
                let expr = HashExpr::initialize_compile(dag, h);
                Expr::HashExpr(expr)
            }
            ExprValue::MultiHash(h) => {
                let expr = MultiHashExpr::initialize_compile(dag, h);
                Expr::MultiHashExpr(expr)
            }
            ExprValue::Bool(b) => {
                let expr = BoolExpr::initialize_compile(dag, b);
                Expr::BoolExpr(expr)
            }
            ExprValue::BoolArr(b) => {
                let expr = BoolArrExpr::initialize_compile(dag, b);
                Expr::BoolArrExpr(expr)
            }
            ExprValue::ProofExpr(p) => {
                let expr = ProofExpr::initialize_compile(dag, p);
                Expr::ProofExpr(expr)
            }
        }
    }
    fn update_compile(
        &mut self,
        dag: &mut DAGState<F, C, D, N>,
        new_val: Self::Inner,
    ) -> Self::Targets {
        match self {
            Expr::FieldExpr(f) => {
                if let ExprValue::Field(other_f) = new_val {
                    ExprTargets::FieldTarget(f.update_compile(dag, other_f))
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::ArrExpr(a) => {
                if let ExprValue::Arr(other_a) = new_val {
                    ExprTargets::ArrTarget(a.update_compile(dag, other_a))
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::HashExpr(h) => {
                if let ExprValue::Hash(other_h) = new_val {
                    ExprTargets::HashTarget(h.update_compile(dag, other_h))
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::MultiHashExpr(h) => {
                if let ExprValue::MultiHash(other_h) = new_val {
                    ExprTargets::MultiHashTarget(h.update_compile(dag, other_h))
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::BoolExpr(b) => {
                if let ExprValue::Bool(other_b) = new_val {
                    ExprTargets::BoolTarget(<BoolExpr as CompileExpr<F, C, D, N>>::update_compile(
                        b, dag, other_b,
                    ))
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::BoolArrExpr(b) => {
                if let ExprValue::BoolArr(other_b) = new_val {
                    ExprTargets::BoolArrTarget(
                        <BoolArrExpr<N> as CompileExpr<F, C, D, N>>::update_compile(
                            b, dag, other_b,
                        ),
                    )
                } else {
                    panic!("Invalid value to be updated on");
                }
            }
            Expr::ProofExpr(..) => {
                unimplemented!("Cannot update proof data")
            }
        }
    }
    fn targets(&self) -> Self::Targets {
        match self {
            Self::FieldExpr(i) => {
                ExprTargets::FieldTarget(<FieldExpr<F> as CompileExpr<F, C, D, N>>::targets(i))
            }
            Self::ArrExpr(i) => {
                ExprTargets::ArrTarget(<ArrExpr<F, N> as CompileExpr<F, C, D, N>>::targets(i))
            }
            Self::HashExpr(i) => {
                ExprTargets::HashTarget(<HashExpr<F> as CompileExpr<F, C, D, N>>::targets(i))
            }
            Self::MultiHashExpr(i) => ExprTargets::MultiHashTarget(
                <MultiHashExpr<F> as CompileExpr<F, C, D, N>>::targets(i),
            ),
            Self::BoolExpr(i) => {
                ExprTargets::BoolTarget(<BoolExpr as CompileExpr<F, C, D, N>>::targets(i))
            }
            Self::BoolArrExpr(i) => {
                ExprTargets::BoolArrTarget(<BoolArrExpr<N> as CompileExpr<F, C, D, N>>::targets(i))
            }
            Self::ProofExpr(i) => ExprTargets::ProofExprTargets(
                <ProofExpr<F, C, D> as CompileExpr<F, C, D, N>>::targets(i),
            ),
        }
    }
}

impl<F: Field> PartialEq for FieldExpr<F> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

impl<F: Field, const N: usize> PartialEq for ArrExpr<F, N> {
    fn eq(&self, other: &Self) -> bool {
        self.expr == other.expr
    }
}

impl PartialEq for BoolExpr {
    fn eq(&self, other: &Self) -> bool {
        self.b == other.b
    }
}

impl<const N: usize> PartialEq for BoolArrExpr<N> {
    fn eq(&self, other: &Self) -> bool {
        self.b_arr == other.b_arr
    }
}

impl<F: Field> PartialEq for HashExpr<F> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<F: Field> PartialEq for MultiHashExpr<F> {
    fn eq(&self, other: &Self) -> bool {
        self.hashes == other.hashes
    }
}

impl<F: Field + RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> PartialEq
    for ProofExpr<F, C, D>
{
    fn eq(&self, other: &Self) -> bool {
        self.proof_with_pis == other.proof_with_pis
            && self.common_circuit_data == other.common_circuit_data
            && self.verifier_only_data == other.verifier_only_data
    }
}

impl<F: Field> Eq for FieldExpr<F> {}

impl<F: Field, const N: usize> Eq for ArrExpr<F, N> {}

impl Eq for BoolExpr {}

impl<const N: usize> Eq for BoolArrExpr<N> {}

impl<F: Field> Eq for HashExpr<F> {}

impl<F: Field> Eq for MultiHashExpr<F> {}

impl<F: Field + RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> Eq
    for ProofExpr<F, C, D>
{
}
