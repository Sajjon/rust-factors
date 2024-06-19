use crate::prelude::*;

pub trait IsSignaturesBuilder {
    type InvalidIfSkipped: std::hash::Hash;

    fn skip_status(&self, factor_source: &FactorSource)
        -> SkipFactorStatus<Self::InvalidIfSkipped>;

    fn skip_factor_sources(&self, factor_source: &FactorSource);
    fn has_fulfilled_signatures_requirement(&self) -> bool;
    fn signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload>;
    fn append_signature(&self, signature: SignatureByOwnedFactorForPayload);
}
