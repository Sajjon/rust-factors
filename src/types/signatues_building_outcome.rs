use std::cmp::Ordering;

use itertools::Itertools;

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignaturesForTransaction {
    pub intent_hash: IntentHash,
    pub signatures: IndexSet<SignatureByOwnedFactor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignaturesBuildingOutcome {
    /// Signatures of all valid transactions
    /// ```ignore
    /// for     each    (valid)transaction
    /// by      every   entities
    /// with    some    factor sources
    /// of      all     factor instances
    /// ```
    pub valid_transactions_signatures: IndexMap<IntentHash, SignaturesForTransaction>,

    /// Collection of invalid transactions for which we failed to "build"
    /// enough signatures for authentication since user  skipped signing them
    /// with the prompted factor sources.
    pub invalid_transactions: IndexSet<InvalidTransactionIfSkipped>,
}

impl SignaturesBuildingOutcome {
    pub fn new(
        valid_transactions_signatures: IndexMap<IntentHash, SignaturesForTransaction>,
        invalid_transactions: IndexSet<InvalidTransactionIfSkipped>,
    ) -> Self {
        Self {
            valid_transactions_signatures,
            invalid_transactions,
        }
    }
}

impl SignaturesBuildingOutcome {
    pub fn signatures_of_all_valid_transactions(
        &self,
    ) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.valid_transactions_signatures
            .clone()
            .into_iter()
            .flat_map(|(_, v)| {
                v.signatures.clone().into_iter().map(|y| {
                    SignatureByOwnedFactorForPayload::new(
                        v.intent_hash,
                        y.owned_factor_instance,
                        y.signature,
                    )
                })
            })
            .collect()
    }
}
