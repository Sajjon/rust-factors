use crate::prelude::*;

pub struct Context;

#[async_trait::async_trait]
impl SignaturesBuilder for Context {
    fn new(
        user: SigningUser,
        all_factor_sources_in_profile: IndexSet<FactorSource>,
        transactions: IndexSet<TransactionIntent>,
    ) -> impl SignaturesBuilder {
        Context
    }

    async fn sign(&self) -> SignaturesOutcome {
        SignaturesOutcome::new(
            MaybeSignedTransactions::new(IndexMap::new()),
            MaybeSignedTransactions::new(IndexMap::new()),
        )
    }
}
