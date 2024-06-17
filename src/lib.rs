use indexmap::{IndexMap, IndexSet};
use std::collections::hash_set::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorInstance;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct Hash;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub enum EntitySecurityState {
    Unsecured(FactorInstance),
    Securified(MatrixOfFactorInstances)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct AccountAddress;

#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct IdentityAddress;


#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash)]
pub enum AccountAddressOrIdentityAddress {
    AccountAddress(AccountAddress),
    IdentityAddress(IdentityAddress)
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct AccountOrPersona {
    pub address: AccountAddressOrIdentityAddress,
    pub security_state: EntitySecurityState
}

impl From<&AccountOrPersona> for OwnedMatrixOfFactorInstances {
fn from(value: &AccountOrPersona) -> Self {
    let matrix = match value.security_state.clone() {
        EntitySecurityState::Securified(matrix) => matrix.clone(),
        EntitySecurityState::Unsecured(instance) => MatrixOfFactorInstances::from(instance)
    };
    OwnedMatrixOfFactorInstances {
        address_of_owner: value.address,
        matrix,
    }
}
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct MatrixOfFactorInstances {
    pub threshold_factors: Vec<FactorInstance>,
    pub threshold: u8,
    pub override_factors: Vec<FactorInstance>,
}

/// For unsecurified entities we map single factor -> single threshold factor.
/// Which is used by ROLA.
impl From<FactorInstance> for MatrixOfFactorInstances {
    fn from(value: FactorInstance) -> Self {
        Self {
            threshold: 1,
            threshold_factors: vec![value],
            override_factors: Vec::new()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct OwnedMatrixOfFactorInstances {
    pub matrix: MatrixOfFactorInstances,
    pub address_of_owner: AccountAddressOrIdentityAddress
}


pub trait PayloadsToSign {
    type PayloadID;
    fn hash_to_sign_for_payload(&self, id: &Self::PayloadID) -> Hash;
}
pub trait OwnedMatrixOfFactorsToSignWithPerPayload: PayloadsToSign {
    fn owned_matrix_factors_to_sign_with_of_payload(&self, id: &Self::PayloadID) -> IndexSet<OwnedMatrixOfFactorInstances>;
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct IntentHash {
    hash: Hash
}
impl IntentHash {
    pub fn hash(&self) -> Hash {
        self.hash.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionIntent {
    pub intent_hash: IntentHash,
    pub entities_requiring_auth: IndexSet<AccountOrPersona>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionsPayloads {
     pub intents: IndexMap<IntentHash, TransactionIntent>
}
impl PayloadsToSign for TransactionsPayloads {
    type PayloadID = IntentHash;

    fn hash_to_sign_for_payload(&self, id: &Self::PayloadID) -> Hash {
        self.intents.get(id).map(|x| x.intent_hash.clone().hash).expect("hash for payload")
    }
}

impl OwnedMatrixOfFactorsToSignWithPerPayload for TransactionsPayloads {
    fn owned_matrix_factors_to_sign_with_of_payload(&self, id: &Self::PayloadID) -> IndexSet<OwnedMatrixOfFactorInstances> {
        self.intents.get(id).map(|x| x.entities_requiring_auth.iter().map(OwnedMatrixOfFactorInstances::from).collect::<IndexSet<_>>()).expect("owned matrix of factors to sign")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RolaPayload {
    origin: String,
    challenge: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigningOfRolaPayload {
    payload: RolaPayload,
    rola_factor: FactorInstance,
    address_of_owner: AccountAddressOrIdentityAddress,
}
impl RolaPayload {
    fn hash_to_sign(&self) -> Hash {
        Hash
    }
}
impl PayloadsToSign for SigningOfRolaPayload {
    type PayloadID = Hash;

    fn hash_to_sign_for_payload(&self, id: &Self::PayloadID) -> Hash {
        let hash =  self.payload.hash_to_sign();
       assert_eq!(id, &hash);
       hash
    }
}

impl OwnedMatrixOfFactorsToSignWithPerPayload for SigningOfRolaPayload {
    fn owned_matrix_factors_to_sign_with_of_payload(&self, id: &Self::PayloadID) -> IndexSet<OwnedMatrixOfFactorInstances> {
        let hash =  self.payload.hash_to_sign();
        assert_eq!(id, &hash);
        IndexSet::from_iter([
            OwnedMatrixOfFactorInstances {
                address_of_owner: self.address_of_owner,
                matrix: MatrixOfFactorInstances::from(self.rola_factor.clone())
            }
        ])
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SigningContextMode {
    Transactions {
        transactions_payloads: TransactionsPayloads
    },
    Rola {
        rola_payload: RolaPayload
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigningContext {
    mode: SigningContextMode
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct Signature;


#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub enum PayloadID {
    Transaction(IntentHash),
    RolaPayload(Hash)
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct SignatureByOwnedFactorForPayload {
	pub payload_id: PayloadID,
    pub hash: Hash,
	pub address_of_owner: AccountAddressOrIdentityAddress,
	pub signature: Signature,
	pub factor_instance: FactorInstance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signatures {
    mode: SigningContextMode,
    /// *ALL* signatures of all transaction for all entities by all factor sources
    /// for all factor instances.
    all_signatures: IndexSet<SignatureByOwnedFactorForPayload>
}

impl SigningContext {
    pub fn sign(&self) -> Signatures {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_() {
        assert_eq!(1, 1);
    }
}
