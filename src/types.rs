use std::time::SystemTime;

use crate::prelude::*;
use rand::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorSourceID;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorSource {
    pub kind: FactorSourceKind,
    pub last_used: SystemTime,
    pub id: FactorSourceID,
}

impl PartialOrd for FactorSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FactorSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.kind.cmp(&other.kind) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.last_used.cmp(&other.last_used) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        core::cmp::Ordering::Equal
    }
}

impl FactorSource {
    fn sign(&self, _intent_hash: &IntentHash, _factor_instance: &FactorInstance) -> Signature {
        Signature
    }
    pub async fn batch_sign(
        &self,
        intent_hash: &IntentHash,
        owned_instances: impl IntoIterator<Item = OwnedFactorInstance>,
    ) -> IndexSet<SignatureByOwnedFactorForPayload> {
        owned_instances
            .into_iter()
            .map(|oi| {
                let signature = self.sign(intent_hash, &oi.factor_instance);
                SignatureByOwnedFactorForPayload::new(intent_hash.clone(), oi, signature)
            })
            .collect()
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash, PartialOrd, Ord)]
pub enum FactorSourceKind {
    Ledger,
    Arculus,
    SecurityQuestions,
    OffDeviceMnemonic,
    Device,
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorInstance {
    pub factor_source_id: FactorSourceID,
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct OwnedFactorInstance {
    pub factor_instance: FactorInstance,
    pub owner: AccountAddressOrIdentityAddress,
}
impl OwnedFactorInstance {
    pub fn new(factor_instance: FactorInstance, owner: AccountAddressOrIdentityAddress) -> Self {
        Self {
            factor_instance,
            owner,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct Hash;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub enum EntitySecurityState {
    Unsecured(FactorInstance),
    Securified(MatrixOfFactorInstances),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct AccountAddress;

#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct IdentityAddress;

#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash)]
pub enum AccountAddressOrIdentityAddress {
    AccountAddress(AccountAddress),
    IdentityAddress(IdentityAddress),
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct AccountOrPersona {
    pub address: AccountAddressOrIdentityAddress,
    pub security_state: EntitySecurityState,
}

impl From<&AccountOrPersona> for OwnedMatrixOfFactorInstances {
    fn from(value: &AccountOrPersona) -> Self {
        let matrix = match value.security_state.clone() {
            EntitySecurityState::Securified(matrix) => matrix.clone(),
            EntitySecurityState::Unsecured(instance) => MatrixOfFactorInstances::from(instance),
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
            override_factors: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct OwnedMatrixOfFactorInstances {
    pub address_of_owner: AccountAddressOrIdentityAddress,
    pub matrix: MatrixOfFactorInstances,
}
impl OwnedMatrixOfFactorInstances {
    pub fn new(
        address_of_owner: AccountAddressOrIdentityAddress,
        matrix: MatrixOfFactorInstances,
    ) -> Self {
        Self {
            address_of_owner,
            matrix,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct IntentHash {
    hash: Hash,
}
impl IntentHash {
    pub fn hash(&self) -> Hash {
        self.hash.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionIntent {
    pub intent_hash: IntentHash,
    pub entities_requiring_auth: IndexSet<AccountOrPersona>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionsPayloads {
    pub intents: IndexMap<IntentHash, TransactionIntent>,
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct Signature;

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signatures {
    /// **ALL** signatures:
    /// ```ignore
    /// for     each    transaction
    /// by      every   entities
    /// with    some    factor sources
    /// of      all     factor instances
    /// ```
    pub all_signatures: IndexSet<SignatureByOwnedFactorForPayload>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SigningUserInput {
    Sign,
    Skip,
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct InvalidTransactionIfSkipped {
    pub intent_hash: IntentHash,
    pub entities_which_would_fail_auth: Vec<AccountAddressOrIdentityAddress>,
}
impl InvalidTransactionIfSkipped {
    pub fn new(
        intent_hash: IntentHash,
        entities_which_would_fail_auth: Vec<AccountAddressOrIdentityAddress>,
    ) -> Self {
        Self {
            intent_hash,
            entities_which_would_fail_auth,
        }
    }
}

#[async_trait::async_trait]
pub trait IsSigningUser {
    async fn sign_or_skip(
        &self,
        factor_source: &FactorSource,
        invalid_tx_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
    ) -> SigningUserInput;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestSigningUser {
    /// Emulation of a "prudent" user, that signs with all factors sources, i.e.
    /// she never ever "skips" a factor source
    Prudent,

    /// Emulation of a "lazy" user, that skips signing with as many factor
    /// sources as possible.
    Lazy,

    /// Emulation of a "random" user, that skips signing some factor sources
    ///  at random.
    Random,
}

#[async_trait::async_trait]
impl IsSigningUser for TestSigningUser {
    async fn sign_or_skip(
        &self,
        _factor_source: &FactorSource,
        _invalid_tx_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
    ) -> SigningUserInput {
        match self {
            TestSigningUser::Prudent => SigningUserInput::Sign,
            TestSigningUser::Lazy => SigningUserInput::Skip,
            TestSigningUser::Random => {
                let mut rng = rand::thread_rng();
                let num: f64 = rng.gen(); // generates a float between 0 and 1
                if num > 0.5 {
                    SigningUserInput::Skip
                } else {
                    SigningUserInput::Sign
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SigningUser {
    Test(TestSigningUser),
}

#[async_trait::async_trait]
impl IsSigningUser for SigningUser {
    async fn sign_or_skip(
        &self,
        factor_source: &FactorSource,
        invalid_tx_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
    ) -> SigningUserInput {
        match self {
            SigningUser::Test(test_user) => {
                test_user
                    .sign_or_skip(&factor_source, invalid_tx_if_skipped)
                    .await
            }
        }
    }
}
