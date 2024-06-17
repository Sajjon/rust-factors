use crate::prelude::*;

pub trait IsSignaturesBuilder {
    fn can_skip_factor_source(&self, factor_source: &FactorSource) -> bool;
    fn skip_factor_sources(&mut self, factor_source: &FactorSource);
    fn has_fulfilled_signatures_requirement(&self) -> bool {
        todo!()
    }
    fn signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        todo!()
    }
    fn append_signature(&mut self, signature: SignatureByOwnedFactorForPayload) {
        todo!()
    }
}
