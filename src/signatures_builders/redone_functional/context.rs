use std::ops::Index;

use crate::prelude::*;

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

    fn add_signatures(&self, signatures: IndexSet<SignatureByOwnedFactorForPayload>) {
        todo!()
    }

    async fn do_sign(&self) -> Result<()> {
        let factors_of_kind = self.factors_of_kind.clone();
        for (kind, factor_sources) in factors_of_kind.into_iter() {
            let signing_driver = self
                .signing_drivers_context
                .driver_for_factor_source_kind(kind);

            match signing_driver.concurrency() {
                SigningFactorConcurrency::Serial => {
                    for factor_source in factor_sources.iter() {
                        assert_eq!(factor_source.kind(), kind);

                        let batched_intent_hashes =
                            self.transactions_to_sign_with_factor_source(factor_source);

                        let batched_factor_instances =
                            self.factor_instances_to_sign_with_using_factor_source(factor_source);

                        let sign_with_factor_source_outcome = signing_driver
                            .sign_serial(
                                factor_source,
                                batched_intent_hashes,
                                batched_factor_instances,
                            )
                            .await;

                        match sign_with_factor_source_outcome {
                            SignWithFactorSourceOrSourcesOutcome::Signed(_) => todo!(),
                            SignWithFactorSourceOrSourcesOutcome::Skipped => todo!(),
                            SignWithFactorSourceOrSourcesOutcome::Interrupted(_) => todo!(),
                        }

                        // batched_signatures
                        // self.add_signatures(batched_signatures)
                    }
                }
                SigningFactorConcurrency::Parallel => {
                    let super_batched_intent_hashes = factor_sources
                        .iter()
                        .flat_map(|f| self.transactions_to_sign_with_factor_source(f))
                        .collect::<IndexSet<_>>();

                    let super_batched_factor_instances = factor_sources
                        .iter()
                        .flat_map(|f| self.factor_instances_to_sign_with_using_factor_source(f))
                        .collect::<IndexSet<_>>();

                    let outcome = signing_driver
                        .sign_parallel(
                            factor_sources,
                            super_batched_intent_hashes,
                            super_batched_factor_instances,
                        )
                        .await;

                    // super_batched_signatures
                    // self.add_signatures(super_batched_signatures)
                }
            }
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
}
