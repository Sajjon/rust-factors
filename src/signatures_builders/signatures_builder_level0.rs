use crate::prelude::*;

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
    builders_level_0: HashMap<IntentHash, Vec<SignaturesBuilderLevel1>>,
}

impl IsSignaturesBuilder for SignaturesBuilderLevel0 {
    fn can_skip_factor_source(&self, factor_source: &FactorSource) -> bool {
        self.builders_level_0.values().all(|builders_level_1| {
            builders_level_1
                .iter()
                .all(|b| b.can_skip_factor_source(factor_source))
        })
    }

    fn signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.builders_level_0
            .values()
            .into_iter()
            .flat_map(|builders_level_1| builders_level_1.iter().flat_map(|b| b.signatures()))
            .collect()
    }

    fn skip_factor_sources(&mut self, factor_source: &FactorSource) {
        self.builders_level_0
            .values_mut()
            .for_each(|builders_level_0| {
                builders_level_0
                    .iter_mut()
                    .for_each(|b| b.skip_factor_sources(factor_source))
            })
    }

    fn has_fulfilled_signatures_requirement(&self) -> bool {
        self.builders_level_0.values().all(|builders_level_1| {
            builders_level_1
                .iter()
                .all(|b| b.has_fulfilled_signatures_requirement())
        })
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
                let skip = if self.can_skip_factor_source(factor_source) {
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
