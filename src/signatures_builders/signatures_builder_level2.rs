use core::panic;
use std::{cell::RefCell, os::macos::raw::stat};

use crate::prelude::*;

/// `SignaturesBuilderOfEntity`
/// Signatures Builder for an Entity: Aggregates over multiple factor instances.
#[derive(Debug)]
pub struct SignaturesBuilderLevel2 {
    owned_matrix_of_factors: OwnedMatrixOfFactorInstances,
    pub skipped_factor_source_ids: RefCell<Vec<FactorSourceID>>,
    pub signatures: RefCell<Vec<SignatureByOwnedFactorForPayload>>,
    pub status: RefCell<SignaturesBuildingStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignaturesBuildingStatus {
    Building,
    /// E.g. failed since too many factor sources were skipped
    Invalid,
    Signed,
}

#[derive(Debug, Clone)]
pub enum SkipFactorStatus<InvalidIfSkipped: std::hash::Hash> {
    StillBuildingCanSkip,
    StillBuildingWillFailIfSkip(IndexSet<InvalidIfSkipped>),
    SignedNoNeedToSign,
    InvalidNoNeedToSign,
}

impl SignaturesBuilderLevel2 {
    pub fn new(owned_matrix_of_factors: OwnedMatrixOfFactorInstances) -> Self {
        Self {
            owned_matrix_of_factors,
            skipped_factor_source_ids: Vec::new().into(),
            signatures: Vec::new().into(),
            status: SignaturesBuildingStatus::Building.into(),
        }
    }
    pub fn new_unsecurified(
        address_of_owner: AccountAddressOrIdentityAddress,
        factor_instance: FactorInstance,
    ) -> Self {
        Self::new(OwnedMatrixOfFactorInstances::new(
            address_of_owner,
            MatrixOfFactorInstances::from(factor_instance),
        ))
    }
    pub fn new_securified(
        address_of_owner: AccountAddressOrIdentityAddress,
        matrix: MatrixOfFactorInstances,
    ) -> Self {
        Self::new(OwnedMatrixOfFactorInstances::new(address_of_owner, matrix))
    }

    pub fn owned_instance_of_factor_source(
        &self,
        factor_source_id: &FactorSourceID,
    ) -> OwnedFactorInstance {
        let factors = if self.is_override_factor(factor_source_id) {
            &self.owned_matrix_of_factors.matrix.override_factors
        } else if self.is_threshold_factor(factor_source_id) {
            &self.owned_matrix_of_factors.matrix.threshold_factors
        } else {
            panic!("MUST be either threshold or override")
        };

        let instance = factors
            .into_iter()
            .find(|fi| &fi.factor_source_id == factor_source_id)
            .unwrap();

        return OwnedFactorInstance::new(
            instance.clone(),
            self.owned_matrix_of_factors.address_of_owner.clone(),
        );
    }
}
impl SignaturesBuilderLevel2 {
    fn threshold(&self) -> usize {
        self.owned_matrix_of_factors.matrix.threshold as usize
    }

    fn signed_override_factors(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.signatures
            .borrow()
            .iter()
            .filter(|s| {
                self.all_override_factor_source_ids()
                    .contains(s.factor_source_id())
            })
            .cloned()
            .collect::<IndexSet<SignatureByOwnedFactorForPayload>>()
    }

    pub fn signed_threshold_factors(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.signatures
            .borrow()
            .iter()
            .filter(|s| {
                self.all_threshold_factor_source_ids()
                    .contains(s.factor_source_id())
            })
            .cloned()
            .collect::<IndexSet<SignatureByOwnedFactorForPayload>>()
    }

    fn has_fulfilled_signatures_requirement_thanks_to_override_factors(&self) -> bool {
        !self.signed_override_factors().is_empty()
    }

    fn has_fulfilled_signatures_requirement_thanks_to_threshold_factors(&self) -> bool {
        if self.threshold() == 0 {
            return false; // corner case
        }
        self.signed_threshold_factors().len() >= self.threshold()
    }

    fn is_override_factor(&self, id: &FactorSourceID) -> bool {
        self.all_override_factor_source_ids().contains(id)
    }

    fn is_threshold_factor(&self, id: &FactorSourceID) -> bool {
        self.all_threshold_factor_source_ids().contains(id)
    }

    fn all_override_factor_source_ids(&self) -> IndexSet<FactorSourceID> {
        IndexSet::from_iter(
            self.owned_matrix_of_factors
                .matrix
                .override_factors
                .clone()
                .into_iter()
                .map(|f| f.factor_source_id),
        )
    }

    fn ids_of_factor_sources_signed_with(&self) -> IndexSet<FactorSourceID> {
        self.signatures
            .borrow()
            .clone()
            .into_iter()
            .map(|s| s.factor_source_id().clone())
            .collect::<IndexSet<_>>()
    }

    pub fn ids_of_skipped_factor_sources(&self) -> IndexSet<FactorSourceID> {
        self.skipped_factor_source_ids
            .borrow()
            .clone()
            .into_iter()
            .collect::<IndexSet<_>>()
    }

    fn ids_of_skipped_threshold_factor_sources(&self) -> IndexSet<FactorSourceID> {
        let threshold_factors = self.all_threshold_factor_source_ids();
        self.ids_of_skipped_factor_sources()
            .intersection(&threshold_factors)
            .into_iter()
            .map(|x| x.clone())
            .collect::<IndexSet<_>>()
    }

    fn ids_of_skipped_override_factor_sources(&self) -> IndexSet<FactorSourceID> {
        let override_factors = self.all_override_factor_source_ids();
        self.ids_of_skipped_factor_sources()
            .intersection(&override_factors)
            .into_iter()
            .map(|x| x.clone())
            .collect::<IndexSet<_>>()
    }

    fn ids_of_signed_override_factor_sources(&self) -> IndexSet<FactorSourceID> {
        let override_factors = self.all_override_factor_source_ids();
        let ids_of_signed = self.ids_of_factor_sources_signed_with();
        ids_of_signed
            .intersection(&override_factors)
            .into_iter()
            .map(|x| x.clone())
            .collect::<IndexSet<_>>()
    }

    fn ids_of_signed_threshold_factor_sources(&self) -> IndexSet<FactorSourceID> {
        let threshold_factors = self.all_threshold_factor_source_ids();
        let ids_of_signed = self.ids_of_factor_sources_signed_with();
        ids_of_signed
            .intersection(&threshold_factors)
            .into_iter()
            .map(|x| x.clone())
            .collect::<IndexSet<_>>()
    }

    /// "done" is either "skipped" or "has signed with"
    fn ids_of_done_threshold_factors(&self) -> IndexSet<FactorSourceID> {
        let skipped = self.ids_of_skipped_threshold_factor_sources();
        let signed = self.ids_of_signed_threshold_factor_sources();
        skipped
            .union(&signed)
            .into_iter()
            .map(|x| x.clone())
            .collect::<IndexSet<_>>()
    }

    /// "done" is either "skipped" or "has signed with"
    fn ids_of_done_override_factors(&self) -> IndexSet<FactorSourceID> {
        let skipped = self.ids_of_skipped_override_factor_sources();
        let signed = self.ids_of_signed_override_factor_sources();
        skipped
            .union(&signed)
            .into_iter()
            .map(|x| x.clone())
            .collect::<IndexSet<_>>()
    }

    fn ids_of_remaining_threshold_factors(&self) -> IndexSet<FactorSourceID> {
        let all = self.all_threshold_factor_source_ids();
        let done = self.ids_of_done_threshold_factors();
        all.difference(&done)
            .into_iter()
            .map(|x| x.clone())
            .collect::<IndexSet<_>>()
    }

    fn ids_of_remaining_override_factors(&self) -> IndexSet<FactorSourceID> {
        let all = self.all_override_factor_source_ids();
        let done = self.ids_of_done_override_factors();
        all.difference(&done)
            .into_iter()
            .map(|x| x.clone())
            .collect::<IndexSet<_>>()
    }

    fn all_threshold_factor_source_ids(&self) -> IndexSet<FactorSourceID> {
        IndexSet::from_iter(
            self.owned_matrix_of_factors
                .matrix
                .threshold_factors
                .clone()
                .into_iter()
                .map(|f| f.factor_source_id),
        )
    }

    fn is_unable_to_complete_signing_using_override_factors(&self) -> bool {
        self.ids_of_remaining_override_factors().is_empty()
            && self.signed_override_factors().is_empty()
    }
    fn is_unable_to_complete_signing_using_threshold_factors(&self) -> bool {
        if self.has_fulfilled_signatures_requirement_thanks_to_threshold_factors() {
            return false;
        }
        let remaining_to_eval = self.ids_of_remaining_threshold_factors().len() as i32;
        let signed = self.ids_of_signed_threshold_factor_sources().len() as i32;
        let required = self.threshold() as i32;
        let additional_required = required - signed;
        let is_unable = remaining_to_eval < additional_required;
        is_unable
    }
    fn is_invalid(&self) -> bool {
        self.is_unable_to_complete_signing_using_override_factors()
            && self.is_unable_to_complete_signing_using_threshold_factors()
    }

    /// Calculates the status, potentially a new status, this is use by `update_status`
    /// internally, which will assert that the status transition is valid.
    fn calculate_status(&self) -> SignaturesBuildingStatus {
        let is_signed = self.has_fulfilled_signatures_requirement();
        let is_invalid = self.is_invalid();
        assert!(
            !(is_signed && is_invalid),
            "Cannot be both invalid and signed, this is a programmer error"
        );
        if is_signed {
            SignaturesBuildingStatus::Signed
        } else if is_invalid {
            SignaturesBuildingStatus::Invalid
        } else {
            SignaturesBuildingStatus::Building
        }
    }

    /// Mutates self (with interior mutability) and returns the updates status.
    fn update_status(&self) -> SignaturesBuildingStatus {
        use SignaturesBuildingStatus::*;
        let current = self.status.borrow().clone();
        let new = self.calculate_status();

        let valid = match (current, new) {
            (Building, Signed) | (Building, Invalid) | (Signed, Signed) => true,
            _ => false,
        };
        if !valid {
            panic!("Invalid status transition from {:?} to {:?}", current, new);
        }

        *self.status.borrow_mut() = new;
        return new;
    }
}

impl IsSignaturesBuilder for SignaturesBuilderLevel2 {
    fn has_fulfilled_signatures_requirement(&self) -> bool {
        self.has_fulfilled_signatures_requirement_thanks_to_override_factors()
            || self.has_fulfilled_signatures_requirement_thanks_to_threshold_factors()
    }

    fn signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        IndexSet::from_iter(self.signatures.borrow().clone())
    }

    type InvalidIfSkipped = AccountAddressOrIdentityAddress;

    fn skip_status(
        &self,
        factor_source: &FactorSource,
    ) -> SkipFactorStatus<Self::InvalidIfSkipped> {
        let id = &factor_source.id;
        if self.skipped_factor_source_ids.borrow().contains(id) {
            panic!("Cannot skipped twice. This is a programmer error");
        }

        let status = self.status.borrow().clone();
        match status {
            SignaturesBuildingStatus::Building => {}
            SignaturesBuildingStatus::Invalid => {
                return SkipFactorStatus::InvalidNoNeedToSign;
            }
            SignaturesBuildingStatus::Signed => {
                return SkipFactorStatus::SignedNoNeedToSign;
            }
        }

        println!("\n\n✨✨✨✨✨✨\n\n");

        println!(
            "🐙 all_threshold_factor_source_ids: {}",
            self.all_threshold_factor_source_ids().len()
        );
        println!(
            "🐙 ids_of_remaining_threshold_factors: {}",
            self.ids_of_remaining_threshold_factors().len()
        );
        println!(
            "🐙 ids_of_done_threshold_factors: {}",
            self.ids_of_done_threshold_factors().len()
        );
        println!(
            "🐙 ids_of_signed_threshold_factor_sources: {}",
            self.ids_of_signed_threshold_factor_sources().len()
        );
        println!(
            "🐙 ids_of_skipped_threshold_factor_sources: {}",
            self.ids_of_skipped_threshold_factor_sources().len()
        );
        println!(
            "🐙 ids_of_skipped_factor_sources: {}",
            self.ids_of_skipped_factor_sources().len()
        );
        println!(
            "🐙 ids_of_factor_sources_signed_with: {}",
            self.ids_of_factor_sources_signed_with().len()
        );
        println!(
            "🐙 all_override_factor_source_ids: {}",
            self.all_override_factor_source_ids().len()
        );

        if self.is_override_factor(id) {
            let number_of_remaining_override_factors_to_eval_including_this =
                self.ids_of_remaining_override_factors().len() as i32;

            let can_skip_factor_source =
                number_of_remaining_override_factors_to_eval_including_this > 1;

            if can_skip_factor_source {
                SkipFactorStatus::<_>::StillBuildingCanSkip
            } else {
                SkipFactorStatus::<_>::StillBuildingWillFailIfSkip(IndexSet::from_iter([self
                    .owned_matrix_of_factors
                    .address_of_owner
                    .clone()]))
            }
        } else if self.is_threshold_factor(id) {
            let number_of_additionally_required_threshold_factors_to_sign = self.threshold() as i32
                - self.ids_of_signed_threshold_factor_sources().len() as i32;

            let number_of_remaining_threshold_factors_to_eval_including_this =
                self.ids_of_remaining_threshold_factors().len() as i32;

            let delta = number_of_remaining_threshold_factors_to_eval_including_this
                - number_of_additionally_required_threshold_factors_to_sign;
            let can_skip_factor_source = delta > 0;

            if can_skip_factor_source {
                SkipFactorStatus::<_>::StillBuildingCanSkip
            } else {
                SkipFactorStatus::<_>::StillBuildingWillFailIfSkip(IndexSet::from_iter([self
                    .owned_matrix_of_factors
                    .address_of_owner
                    .clone()]))
            }
        } else {
            panic!("MUST be in either overrideFactors OR in thresholdFactors (and was not in overrideFactors...)")
        }
    }

    fn skip_factor_sources(&self, factor_source: &FactorSource) {
        {
            let id = factor_source.id;
            let skip_status = self.skip_status(factor_source);
            match skip_status {
                SkipFactorStatus::InvalidNoNeedToSign => {
                    panic!("Should not have skipped a factor source with SkipFactorStatus::InvalidNoNeedToSign.")
                }
                SkipFactorStatus::StillBuildingCanSkip => {}
                SkipFactorStatus::StillBuildingWillFailIfSkip(_) => {
                    panic!("Should not have skipped a factor source with SkipFactorStatus::StillBuildingWillFailIfSkip.")
                }
                SkipFactorStatus::SignedNoNeedToSign => {
                    panic!("Should not have skipped a factor source with SkipFactorStatus::SignedNoNeedToSign.")
                }
            }
            assert!(!self.skipped_factor_source_ids.borrow().contains(&id));
            self.skipped_factor_source_ids.borrow_mut().push(id);
        }

        {
            assert!(!self.skipped_factor_source_ids.borrow().is_empty())
        }

        self.update_status();
    }

    fn append_signature(&self, signature: SignatureByOwnedFactorForPayload) {
        {
            assert_eq!(
                signature.owned_factor_instance.owner,
                self.owned_matrix_of_factors.address_of_owner
            );
            assert!(!self.signatures.borrow().contains(&signature));
            self.signatures.borrow_mut().push(signature);
        }
        {
            assert!(!self.signatures.borrow().is_empty())
        }

        self.update_status();
    }
}
