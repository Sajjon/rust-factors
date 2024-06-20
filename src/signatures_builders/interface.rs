use crate::prelude::*;

#[async_trait::async_trait]
pub trait SignaturesBuilder: Sync {
    fn new(
        user: SigningUser,
        all_factor_sources_in_profile: IndexSet<FactorSource>,
        transactions: IndexSet<TransactionIntent>,
        signing_drivers_context: SigningDriversContext,
    ) -> impl SignaturesBuilder;

    async fn sign(&self) -> Result<SignaturesOutcome>;

    fn invalid_transactions_if_skipped(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<InvalidTransactionIfSkipped>;

    fn transactions_to_sign_with_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<&IntentHash>;

    fn factor_instances_to_sign_with_using_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<&OwnedFactorInstance>;

    fn skipped(&self, skipped_factor_sources: IndexSet<&FactorSource>);
}
