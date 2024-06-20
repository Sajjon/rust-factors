use std::ops::Index;

use crate::{prelude::*, signatures_builders::redone_functional::sign_with_factors};

pub struct Context {
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

impl Context {
    fn add_signatures(&self, signatures: IndexSet<SignatureByOwnedFactorForPayload>) {
        todo!()
    }

    async fn do_sign(&self) -> Result<()> {
        let factors_of_kind = self.factors_of_kind.clone();
        for (kind, factor_sources) in factors_of_kind.into_iter() {
            let signing_driver = self
                .signing_drivers_context
                .driver_for_factor_source_kind(kind);

            let super_mega_batched_signatures =
                signing_driver.sign(kind, factor_sources, self).await?;

            self.add_signatures(super_mega_batched_signatures);
        }
        todo!()
    }
}

#[async_trait::async_trait]
impl SignaturesBuilder for Context {
    fn new(
        user: SigningUser,
        all_factor_sources_in_profile: IndexSet<FactorSource>,
        transactions: IndexSet<TransactionIntent>,
        signing_drivers_context: SigningDriversContext,
    ) -> impl SignaturesBuilder {
        Context {
            signing_drivers_context,
            factors_of_kind: IndexMap::new(),
        }
    }

    async fn sign(&self) -> Result<SignaturesOutcome> {
        self.do_sign().await?;
        let outcome = SignaturesOutcome::new(
            MaybeSignedTransactions::new(IndexMap::new()),
            MaybeSignedTransactions::new(IndexMap::new()),
        );
        Ok(outcome)
    }

    fn invalid_transactions_if_skipped(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<InvalidTransactionIfSkipped> {
        todo!()
    }

    fn transactions_to_sign_with_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<&IntentHash> {
        todo!()
    }

    fn factor_instances_to_sign_with_using_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<&OwnedFactorInstance> {
        todo!()
    }

    fn skipped(&self, skipped_factor_sources: IndexSet<&FactorSource>) {
        todo!()
    }
   
}
