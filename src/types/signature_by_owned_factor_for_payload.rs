use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct SignatureByOwnedFactorForPayload {
    pub intent_hash: IntentHash,
    pub owned_factor_instance: OwnedFactorInstance,
    pub signature: Signature,
}
impl SignatureByOwnedFactorForPayload {
    pub fn new(
        intent_hash: IntentHash,
        owned_factor_instance: OwnedFactorInstance,
        signature: Signature,
    ) -> Self {
        Self {
            intent_hash,
            owned_factor_instance,
            signature,
        }
    }
    pub fn factor_source_id(&self) -> &FactorSourceID {
        &self.owned_factor_instance.factor_instance.factor_source_id
    }

    pub fn signature_by_owned_factor(&self) -> SignatureByOwnedFactor {
        SignatureByOwnedFactor {
            owned_factor_instance: self.owned_factor_instance.clone(),
            signature: self.signature.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct SignatureByOwnedFactor {
    pub owned_factor_instance: OwnedFactorInstance,
    pub signature: Signature,
}
