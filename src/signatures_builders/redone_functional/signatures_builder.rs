use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningInputForFactorSource {
    pub factor_source: FactorSource,
    pub intent_hashes: IndexSet<IntentHash>,
    pub factor_instances: IndexSet<OwnedFactorInstance>,
    pub transactions_which_would_fail_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
}

pub struct SignaturesBuilder {
    signing_drivers_context: SigningDriversContext,
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
}

impl SignaturesBuilder {
    /// If all transactions already would fail, or if all transactions already are done, then
    /// no point in continuing.
    fn continue_if_necessary(&self) -> Result<()> {
        todo!()
    }

    async fn do_sign(&self) -> Result<()> {
        let factors_of_kind = self.factors_of_kind.clone();
        for (kind, factor_sources) in factors_of_kind.into_iter() {
            let signing_driver = self
                .signing_drivers_context
                .driver_for_factor_source_kind(kind);

            signing_driver.sign(kind, factor_sources, self).await;

            self.continue_if_necessary()?;
        }
        Ok(())
    }
}

impl SignaturesBuilder {
    /// Used by "Parallel" SigningDrivers
    pub(super) fn input_per_factors_source(
        &self,
        factor_sources: IndexSet<FactorSource>,
    ) -> IndexMap<FactorSource, SigningInputForFactorSource> {
        todo!()
    }
    /// Used by "Serial" SigningDrivers
    pub(super) fn input_for_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> SigningInputForFactorSource {
        todo!()
    }

    pub(super) fn process(&self, outcome: SignWithFactorSourceOrSourcesOutcome) {
        todo!()
    }
}

impl SignaturesBuilder {
    pub fn new(
        user: SigningUser,
        all_factor_sources_in_profile: IndexSet<FactorSource>,
        transactions: IndexSet<TransactionIntent>,
        signing_drivers_context: SigningDriversContext,
    ) -> Self {
        Self {
            signing_drivers_context,
            factors_of_kind: IndexMap::new(),
        }
    }

    pub async fn sign(&self) -> Result<SignaturesOutcome> {
        self.do_sign().await?;
        let outcome = SignaturesOutcome::new(
            MaybeSignedTransactions::new(IndexMap::new()),
            MaybeSignedTransactions::new(IndexMap::new()),
        );
        Ok(outcome)
    }
}
