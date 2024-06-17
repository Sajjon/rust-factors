use crate::prelude::*;

/// `SignaturesBuilderForTransaction`
/// Signatures Builder for a Transaction: Aggregates over multiple Entities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignaturesBuilderLevel1 {
    /// The payload to sign, the hash of a transaction, also used to identify
    /// the transaction being signed.
    pub intent_hash: IntentHash,

    /// Signature builder for each entity signing this transaction
    pub builders: HashMap<AccountAddressOrIdentityAddress, SignaturesBuilderLevel2>,
}

impl SignaturesBuilderLevel1 {
    pub fn new(
        intent_hash: IntentHash,
        builders: HashMap<AccountAddressOrIdentityAddress, SignaturesBuilderLevel2>,
    ) -> Self {
        Self {
            intent_hash,
            builders,
        }
    }
}

impl IsSignaturesBuilder for SignaturesBuilderLevel1 {
    type InvalidIfSkipped = InvalidTransactionIfSkipped;
    fn invalid_if_skip_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<Self::InvalidIfSkipped> {
        let addresses = self
            .builders
            .values()
            .into_iter()
            .flat_map(|b| b.invalid_if_skip_factor_source(factor_source))
            .collect::<Vec<AccountAddressOrIdentityAddress>>();

        IndexSet::from_iter([InvalidTransactionIfSkipped::new(
            self.intent_hash.clone(),
            addresses,
        )])
    }

    fn skip_factor_sources(&mut self, factor_source: &FactorSource) {
        self.builders
            .values_mut()
            .for_each(|b| b.skip_factor_sources(factor_source))
    }

    fn has_fulfilled_signatures_requirement(&self) -> bool {
        self.builders
            .values()
            .into_iter()
            .all(|b| b.has_fulfilled_signatures_requirement())
    }

    fn signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.builders
            .values()
            .into_iter()
            .flat_map(|b| b.signatures())
            .collect()
    }

    fn append_signature(&mut self, signature: SignatureByOwnedFactorForPayload) {
        self.builders
            .get_mut(&signature.owned_factor_instance.owner)
            .unwrap()
            .append_signature(signature)
    }
}
