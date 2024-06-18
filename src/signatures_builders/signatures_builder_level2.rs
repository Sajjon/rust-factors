use std::{borrow::Borrow, cell::RefCell};

use crate::prelude::*;

/// `SignaturesBuilderOfEntity`
/// Signatures Builder for an Entity: Aggregates over multiple factor instances.
pub struct SignaturesBuilderLevel2 {
    owned_matrix_of_factors: OwnedMatrixOfFactorInstances,
    skipped_factor_source_ids: RefCell<Vec<FactorSourceID>>,
    signatures: RefCell<Vec<SignatureByOwnedFactorForPayload>>,
}

impl SignaturesBuilderLevel2 {
    pub fn new(owned_matrix_of_factors: OwnedMatrixOfFactorInstances) -> Self {
        Self {
            owned_matrix_of_factors,
            skipped_factor_source_ids: Vec::new().into(),
            signatures: Vec::new().into(),
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
            self.owned_matrix_of_factors.address_of_owner,
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

    fn signed_threshold_factors(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
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

    fn can_skip_factor_source(&self, factor_source: &FactorSource) -> bool {
        let id = &factor_source.id;
        if self.skipped_factor_source_ids.borrow().contains(id) {
            // Cannot skipped twice. This is a programmer error.
            return false;
        }
        if self.has_fulfilled_signatures_requirement() {
            // We have already fulfilled the signatures requirement => no more
            // signatures are needed
            return true;
        }

        let skipped =
            IndexSet::<FactorSourceID>::from_iter(self.skipped_factor_source_ids.borrow().clone());
        if self.is_override_factor(id) {
            let ids_of_all_override_factors = self.all_override_factor_source_ids();
            let remaining_override_factor_source_ids = ids_of_all_override_factors
                .difference(&skipped)
                .collect::<IndexSet<_>>();

            // If the remaining override factors is NOT empty, it means that we can sign with any subsequent
            // override factor, thus we can skip this one.
            let can_skip_factor_source = !remaining_override_factor_source_ids.is_empty();
            return can_skip_factor_source;
        } else if self.is_threshold_factor(id) {
            let ids_of_all_threshold_factor_sources = self.all_threshold_factor_source_ids();
            let non_skipped_threshold_factor_source_ids = ids_of_all_threshold_factor_sources
                .difference(&skipped)
                .collect::<IndexSet<_>>();

            // We have not skipped this (`id`) yet, if we would skip it we would at least have
            // `nonSkippedThresholdFactorSourceIDs == securifiedEntityControl.threshold`,
            // since we use `>` below.
            let can_skip_factor_source =
                non_skipped_threshold_factor_source_ids.len() > self.threshold();
            return can_skip_factor_source;
        } else {
            panic!("MUST be in either overrideFactors OR in thresholdFactors (and was not in overrideFactors...)")
        }
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
    fn invalid_if_skip_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<Self::InvalidIfSkipped> {
        if self.can_skip_factor_source(factor_source) {
            IndexSet::new()
        } else {
            IndexSet::from_iter([self.owned_matrix_of_factors.address_of_owner])
        }
    }

    fn skip_factor_sources(&self, factor_source: &FactorSource) {
        let id = factor_source.id;
        assert!(self.can_skip_factor_source(factor_source));
        assert!(!self.skipped_factor_source_ids.borrow().contains(&id));
        self.skipped_factor_source_ids.borrow_mut().push(id);
    }

    fn append_signature(&self, signature: SignatureByOwnedFactorForPayload) {
        assert_eq!(
            signature.owned_factor_instance.owner,
            self.owned_matrix_of_factors.address_of_owner
        );
        assert!(!self.signatures.borrow().contains(&signature));
        self.signatures.borrow_mut().push(signature);
    }
}
