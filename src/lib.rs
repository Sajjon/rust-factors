use std::collections::{HashMap, HashSet};

use indexmap::{IndexMap, IndexSet};
use rand::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorSourceID;

#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct FactorSource {
    kind: FactorSourceKind,
    id: FactorSourceID,
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

pub trait IsSignaturesBuilder {
    fn can_skip_factor_sources(&self, factor_source: &FactorSource) -> bool;
    fn skip_factor_sources(&self, factor_source: &FactorSource) -> bool {
        todo!()
    }
    fn has_fulfilled_signatures_requirement(&self) -> bool {
        todo!()
    }
    fn signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        todo!()
    }
    fn append_signature(&self, signature: SignatureByOwnedFactorForPayload) {
        todo!()
    }
}

/// Root Signing Context: Aggregates over multiple Transactions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignaturesBuilderLevel0 {
    /// Abstraction of a user signing, decides for every factor source if
    /// she wants to skip signing with the factor source if she can,
    /// or cancel the whole signing process (context) or otherwise
    /// asynchronously sign.
    user: SigningUser,

    /// Factor sources grouped by kind, sorted according to "signing order",
    /// that is, we want to control which factor source kind users signs with
    /// first, second etc, e.g. typically we prompt user to sign with Ledgers
    /// first, and if a user might lack access to that Ledger device, then it is
    /// best to "fail fast", otherwise we might waste the users time, if she has
    /// e.g. answered security questions and then is asked to sign with a Ledger
    /// she might not have handy at the moment - or might not be in front of a
    /// computer and thus unable to make a connection between the Radix Wallet
    /// and a Ledger device.
    factors_of_kind: IndexMap<FactorSourceKind, IndexSet<FactorSource>>,

    /// Lookup payloads that need to be signed with a factor source by its
    /// FactorSourceID. By "payload" we mean a transaction of a ROLA challenge.
    /// We support signing of multiple transactions in one go, therefore, the
    /// plural form of payload**s**.
    ///
    /// The same FactorSource might be required by many payloads
    /// and per payload might be required by many entities, e.g. transactions
    /// `t0` and `t1`, where
    /// `t0` is signed by accounts: A and B
    /// `t1` is signed by accounts: A, C and D,
    ///
    /// Where A, B, C and D, all use the factor source, e.g. some arculus
    /// card which the user has setup as a factor (source) for all these accounts.
    factor_to_payloads: HashMap<FactorSourceID, IndexSet<IntentHash>>,

    /// Lookup from payload (TXID) to signatures builders
    builders: HashMap<IntentHash, Vec<SignaturesBuilderLevel1>>,
}

impl IsSignaturesBuilder for SignaturesBuilderLevel0 {
    fn can_skip_factor_sources(&self, factor_source: &FactorSource) -> bool {
        let factor_source_id = &factor_source.id;
        let payloads = self
            .factor_to_payloads
            .get(factor_source_id)
            .expect("Should not have irrelevant factor sources.");

        // payloads.iter().map(||)
        todo!()
    }
}

/// `SignaturesBuilderOfEntity`
/// Signatures Builder for an Entity: Aggregates over multiple factor instances.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SignaturesBuilderLevel2 {
    owned_matrix_of_factors: OwnedMatrixOfFactorInstances,
    skipped_factor_source_ids: IndexSet<FactorSourceID>,
    signatures: IndexSet<SignatureByOwnedFactorForPayload>,
}
impl SignaturesBuilderLevel2 {
    fn threshold(&self) -> usize {
        self.owned_matrix_of_factors.matrix.threshold as usize
    }

    fn signed_override_factors(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.signatures
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
}
impl IsSignaturesBuilder for SignaturesBuilderLevel2 {
    fn has_fulfilled_signatures_requirement(&self) -> bool {
        self.has_fulfilled_signatures_requirement_thanks_to_override_factors()
            || self.has_fulfilled_signatures_requirement_thanks_to_threshold_factors()
    }
    fn can_skip_factor_sources(&self, factor_source: &FactorSource) -> bool {
        let id = &factor_source.id;
        if self.skipped_factor_source_ids.contains(id) {
            // Cannot skipped twice. This is a programmer error.
            return false;
        }
        if self.has_fulfilled_signatures_requirement() {
            // We have already fulfilled the signatures requirement => no more
            // signatures are needed
            return true;
        }

        if self.is_override_factor(id) {
            let ids_of_all_override_factors = self.all_override_factor_source_ids();

            let remaining_override_factor_source_ids = ids_of_all_override_factors
                .difference(&self.skipped_factor_source_ids)
                .collect::<IndexSet<_>>();

            // If the remaining override factors is NOT empty, it means that we can sign with any subsequent
            // override factor, thus we can skip this one.
            let can_skip_factor_source = !remaining_override_factor_source_ids.is_empty();
            return can_skip_factor_source;
        } else if self.is_threshold_factor(id) {
            let ids_of_all_threshold_factor_sources = self.all_threshold_factor_source_ids();
            let non_skipped_threshold_factor_source_ids = ids_of_all_threshold_factor_sources
                .difference(&self.skipped_factor_source_ids)
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

/// `SignaturesBuilderForTransaction`
/// Signatures Builder for a Transaction: Aggregates over multiple Entities.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SignaturesBuilderLevel1 {
    /// The payload to sign, the hash of a transaction, also used to identify
    /// the transaction being signed.
    pub intent_hash: IntentHash,

    /// Signature builder for each entity signing this transaction
    pub builders: HashMap<AccountAddressOrIdentityAddress, SignaturesBuilderLevel2>,
}
impl IsSignaturesBuilder for SignaturesBuilderLevel1 {
    fn can_skip_factor_sources(&self, factor_source: &FactorSource) -> bool {
        self.builders
            .values()
            .into_iter()
            .all(|b| b.can_skip_factor_sources(factor_source))
    }
}

impl SignaturesBuilderLevel0 {
    async fn sign_with(&self, factor_source: &FactorSource) {
        todo!()
    }
    pub async fn sign(&self) -> Signatures {
        for (kind, factor_sources) in self.factors_of_kind.iter() {
            for factor_source in factor_sources.iter() {
                assert_eq!(&factor_source.kind, kind);
                let skip = if self.can_skip_factor_sources(factor_source) {
                    let skip_or_sign = self.user.sign_or_skip(factor_source).await;
                    match skip_or_sign {
                        SigningUserInput::Skip => true,
                        SigningUserInput::Sign => false,
                    }
                } else {
                    false
                };
                if skip {
                    continue;
                }
                // Should sign
                self.sign_with(factor_source).await
            }
        }
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
