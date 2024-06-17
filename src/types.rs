use crate::prelude::*;
use rand::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorSourceID;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorSource {
    pub kind: FactorSourceKind,
    pub id: FactorSourceID,
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub enum FactorSourceKind {
    Device,
    Arculus,
    Ledger,
    OffDeviceMnemonic,
    SecurityQuestions,
}

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorInstance {
    pub factor_source_id: FactorSourceID,
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
    pub matrix: MatrixOfFactorInstances,
    pub address_of_owner: AccountAddressOrIdentityAddress,
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
    pub address_of_owner: AccountAddressOrIdentityAddress,
    pub signature: Signature,
    pub factor_instance: FactorInstance,
}
impl SignatureByOwnedFactorForPayload {
    pub fn factor_source_id(&self) -> &FactorSourceID {
        &self.factor_instance.factor_source_id
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
    all_signatures: IndexSet<SignatureByOwnedFactorForPayload>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SigningUserInput {
    Sign,
    Skip,
}

#[async_trait::async_trait]
pub trait IsSigningUser {
    async fn sign_or_skip(&self, factor_source: &FactorSource) -> SigningUserInput;
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
    async fn sign_or_skip(&self, _factor_source: &FactorSource) -> SigningUserInput {
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
    async fn sign_or_skip(&self, factor_source: &FactorSource) -> SigningUserInput {
        match self {
            SigningUser::Test(test_user) => test_user.sign_or_skip(&factor_source).await,
        }
    }
}