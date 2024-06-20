use std::{
    cell::{Ref, RefCell},
    default,
};

use crate::prelude::*;

#[derive(Clone)]
pub struct PetitionOfTransactionByEntity {
    /// The owner of these factors
    entity: AccountAddressOrIdentityAddress,

    /// Hash of transaction to sign
    intent_hash: IntentHash,

    threshold_factors: RefCell<PetitionWithFactors>,
    override_factors: RefCell<PetitionWithFactors>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum Petition {
    Threshold,
    Override,
}

impl PetitionOfTransactionByEntity {
    fn petition(&self, factor_source: &FactorSource) -> Option<Petition> {
        if self
            .threshold_factors
            .borrow()
            .references_factor_source(factor_source)
        {
            Some(Petition::Threshold)
        } else if self
            .override_factors
            .borrow()
            .references_factor_source(factor_source)
        {
            Some(Petition::Override)
        } else {
            None
        }
    }
}
impl PetitionOfTransactionByEntity {
    pub fn status_if_skipped_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> PetitionForFactorListStatus {
        let simulation = self.clone();
        simulation.skipped(factor_source);
        simulation.status()
    }

    pub fn skipped(&self, factor_source: &FactorSource) {
        let Some(petition) = self.petition(factor_source) else {
            return;
        };
        match petition {
            Petition::Threshold => self.threshold_factors.borrow_mut().skipped(factor_source),
            Petition::Override => self.override_factors.borrow_mut().skipped(factor_source),
        }
    }

    pub fn status(&self) -> PetitionForFactorListStatus {
        use PetitionForFactorListStatus::*;
        use PetitionForFactorListStatusFinished::*;
        let threshold = self.threshold_factors.borrow().status();
        let r#override = self.override_factors.borrow().status();

        match (threshold, r#override) {
            (InProgress, InProgress) => PetitionForFactorListStatus::InProgress,
            (Finished(Fail), InProgress) => PetitionForFactorListStatus::InProgress,
            (InProgress, Finished(Fail)) => PetitionForFactorListStatus::InProgress,
            (Finished(Fail), Finished(Fail)) => PetitionForFactorListStatus::Finished(Fail),
            (Finished(Success), _) => PetitionForFactorListStatus::Finished(Success),
            (_, Finished(Success)) => PetitionForFactorListStatus::Finished(Success),
        }
    }
}

#[derive(Clone)]
pub struct PetitionWithFactors {
    /// Factors to sign with and the required number of them.
    input: PetitionWithFactorsInput,
    state: RefCell<PetitionWithFactorsState>,
}

impl PetitionWithFactors {
    pub fn new(input: PetitionWithFactorsInput) -> Self {
        Self {
            input,
            state: RefCell::new(PetitionWithFactorsState::new()),
        }
    }
    pub fn skipped(&self, factor_source: &FactorSource) {
        let factor_instance = self.expect_reference_to_factor_source(factor_source);
        self.state.borrow_mut().skipped(factor_instance);
    }

    pub fn references_factor_source(&self, factor_source: &FactorSource) -> bool {
        self.reference_to_factor_source(factor_source).is_some()
    }

    fn expect_reference_to_factor_source(&self, factor_source: &FactorSource) -> &FactorInstance {
        self.reference_to_factor_source(factor_source)
            .expect("Programmer error! Factor source not found in factors.")
    }

    fn reference_to_factor_source(&self, factor_source: &FactorSource) -> Option<&FactorInstance> {
        self.input.reference_factor_source(factor_source)
    }
}
#[derive(Clone)]
struct PetitionWithFactorsStateSnapshot {
    /// Factors that have signed.
    signed: IndexSet<SignatureByFactor>,
    /// Factors that user skipped.
    skipped: IndexSet<FactorInstance>,
}
impl PetitionWithFactorsStateSnapshot {
    fn prompted_count(&self) -> i8 {
        self.signed_count() + self.skipped_count()
    }

    fn signed_count(&self) -> i8 {
        self.signed.len() as i8
    }

    fn skipped_count(&self) -> i8 {
        self.skipped.len() as i8
    }
}

pub trait FactorSourceReferencing: std::hash::Hash + PartialEq + Eq + Clone {
    fn factor_source_id(&self) -> FactorSourceID;
}

#[derive(Clone)]
struct PetitionWithFactorsStateFactors<F>
where
    F: FactorSourceReferencing,
{
    /// Factors that have signed or skipped
    factors: RefCell<IndexSet<F>>,
}
impl<F: FactorSourceReferencing> PetitionWithFactorsStateFactors<F> {
    fn new() -> Self {
        Self {
            factors: RefCell::new(IndexSet::new()),
        }
    }

    fn insert(&self, factor: &F) {
        self.factors.borrow_mut().insert(factor.clone());
    }

    fn snapshot(&self) -> IndexSet<F> {
        self.factors.borrow().clone()
    }
    fn references_factor_source_by_id(&self, factor_source_id: FactorSourceID) -> bool {
        self.factors
            .borrow()
            .iter()
            .any(|sf| sf.factor_source_id() == factor_source_id)
    }
}

#[derive(Clone)]
struct PetitionWithFactorsState {
    /// Factors that have signed.
    signed: RefCell<PetitionWithFactorsStateFactors<SignatureByFactor>>,
    /// Factors that user skipped.
    skipped: RefCell<PetitionWithFactorsStateFactors<FactorInstance>>,
}
impl PetitionWithFactorsState {
    fn assert_not_referencing_factor_source(&self, factor_source_id: FactorSourceID) {
        assert!(
            self.references_factor_source_by_id(factor_source_id),
            "Programmer error! Factor source already used, should only be referenced once."
        );
    }
    fn skipped(&self, factor_instance: &FactorInstance) {
        self.assert_not_referencing_factor_source(factor_instance.factor_source_id);
        self.skipped.borrow_mut().insert(factor_instance)
    }

    fn new() -> Self {
        Self {
            signed: RefCell::new(PetitionWithFactorsStateFactors::<_>::new()),
            skipped: RefCell::new(PetitionWithFactorsStateFactors::<_>::new()),
        }
    }
    fn snapshot(&self) -> PetitionWithFactorsStateSnapshot {
        PetitionWithFactorsStateSnapshot {
            signed: self.signed.borrow().snapshot(),
            skipped: self.skipped.borrow().snapshot(),
        }
    }
    fn references_factor_source_by_id(&self, factor_source_id: FactorSourceID) -> bool {
        self.signed
            .borrow()
            .references_factor_source_by_id(factor_source_id)
            || self
                .skipped
                .borrow()
                .references_factor_source_by_id(factor_source_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
struct SignatureByFactor {
    signature: Signature,
    factor: FactorInstance,
}
impl FactorSourceReferencing for SignatureByFactor {
    fn factor_source_id(&self) -> FactorSourceID {
        self.factor.factor_source_id
    }
}

#[derive(Clone)]
struct PetitionWithFactorsInput {
    /// Factors to sign with.
    factors: IndexSet<FactorInstance>,

    /// Number of required factors to sign with.
    required: i8,
}

impl PetitionWithFactorsInput {
    pub fn reference_factor_source(&self, factor_source: &FactorSource) -> Option<&FactorInstance> {
        self.factors
            .iter()
            .find(|f| f.factor_source_id == factor_source.id)
    }
    pub fn references_factor_source(&self, factor_source: &FactorSource) -> bool {
        self.reference_factor_source(factor_source).is_some()
    }

    fn factors_count(&self) -> i8 {
        self.factors.len() as i8
    }

    fn remaining_factors_until_success(&self, snapshot: PetitionWithFactorsStateSnapshot) -> i8 {
        self.required - snapshot.signed_count()
    }

    fn is_fulfilled_by(&self, snapshot: PetitionWithFactorsStateSnapshot) -> bool {
        self.remaining_factors_until_success(snapshot) <= 0
    }

    fn factors_left_to_prompt(&self, snapshot: PetitionWithFactorsStateSnapshot) -> i8 {
        self.factors_count() - snapshot.prompted_count()
    }

    fn is_failure_with(&self, snapshot: PetitionWithFactorsStateSnapshot) -> bool {
        self.factors_left_to_prompt(snapshot) < self.required
    }
}

pub enum PetitionForFactorListStatus {
    /// In progress, still gathering signatures
    InProgress,

    Finished(PetitionForFactorListStatusFinished),
}

pub enum PetitionForFactorListStatusFinished {
    Success,
    Fail,
}

impl PetitionWithFactors {
    fn state_snapshot(&self) -> PetitionWithFactorsStateSnapshot {
        self.state.borrow().snapshot()
    }

    fn is_finished_successfully(&self) -> bool {
        self.input.is_fulfilled_by(self.state_snapshot())
    }

    fn is_finished_with_fail(&self) -> bool {
        self.input.is_failure_with(self.state_snapshot())
    }

    fn finished_with(&self) -> Option<PetitionForFactorListStatusFinished> {
        if self.is_finished_successfully() {
            Some(PetitionForFactorListStatusFinished::Success)
        } else if self.is_finished_with_fail() {
            Some(PetitionForFactorListStatusFinished::Fail)
        } else {
            None
        }
    }
}

impl PetitionWithFactors {
    pub fn status(&self) -> PetitionForFactorListStatus {
        if let Some(finished_state) = self.finished_with() {
            return PetitionForFactorListStatus::Finished(finished_state);
        }
        PetitionForFactorListStatus::InProgress
    }
}
