use crate::prelude::*;

#[async_trait::async_trait]
pub trait SignaturesBuilder {
    fn new(
        user: SigningUser,
        all_factor_sources_in_profile: IndexSet<FactorSource>,
        transactions: IndexSet<TransactionIntent>,
    ) -> impl SignaturesBuilder;

    async fn sign(&self) -> Result<SignaturesOutcome>;
}
