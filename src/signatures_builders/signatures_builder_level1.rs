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

impl IsSignaturesBuilder for SignaturesBuilderLevel1 {
    fn can_skip_factor_source(&self, factor_source: &FactorSource) -> bool {
        self.builders
            .values()
            .into_iter()
            .all(|b| b.can_skip_factor_source(factor_source))
    }

    fn skip_factor_sources(&mut self, factor_source: &FactorSource) {
        self.builders
            .values_mut()
            .for_each(|b| b.skip_factor_sources(factor_source))
    }
}
