use std::{
    cell::{Ref, RefCell},
    default,
};

use crate::prelude::*;

pub struct PetitionOfTransactionByEntity {
    /// The owner of these factors
    entity: AccountAddressOrIdentityAddress,

    /// Hash of transaction to sign
    intent_hash: IntentHash,

    threshold_factors: PetitionWithFactors,

    override_factors: PetitionWithFactors,
}

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
}

struct PetitionWithFactorsStateSnapshot {
    /// Factors that have signed.
    signed: IndexSet<SignatureByFactor>,
    /// Factors that user skipped.
    skipped: IndexSet<FactorInstance>,
}
impl PetitionWithFactorsStateSnapshot {
    // fn prompted(&self) -> IndexSet<FactorInstance> {
    //     let mut prompted = self
    //         .signed
    //         .iter()
    //         .map(|x| x.factor.clone())
    //         .collect::<IndexSet<_>>();
    //     prompted.extend(self.skipped.clone());
    //     prompted
    // }

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

struct PetitionWithFactorsState {
    /// Factors that have signed.
    signed: RefCell<IndexSet<SignatureByFactor>>,
    /// Factors that user skipped.
    skipped: RefCell<IndexSet<FactorInstance>>,
}
impl PetitionWithFactorsState {
    fn new() -> Self {
        Self {
            signed: RefCell::new(IndexSet::new()),
            skipped: RefCell::new(IndexSet::new()),
        }
    }
    fn snapshot(&self) -> PetitionWithFactorsStateSnapshot {
        PetitionWithFactorsStateSnapshot {
            signed: self.signed.borrow().clone(),
            skipped: self.skipped.borrow().clone(),
        }
    }
}

#[derive(Clone, Debug)]
struct SignatureByFactor {
    signature: Signature,
    factor: FactorInstance,
}

struct PetitionWithFactorsInput {
    /// Factors to sign with.
    factors: IndexSet<FactorInstance>,

    /// Number of required factors to sign with.
    required: i8,
}
impl PetitionWithFactorsInput {
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
