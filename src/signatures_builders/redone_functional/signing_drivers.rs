use crate::prelude::*;

/// If a kind of factor source can be used in a parallel or serial manner.
pub enum SigningFactorConcurrency {
    /// Arculus, Ledger etc can only be used in a serial manner, since its
    /// impractical or otherwise infeasible to put multiple Arculus cards
    /// against the phones NFC reader at once.
    ///
    /// Neither is it likely that the user can use more than two hands to
    /// review and approve transactions on multiple Ledger devices, unless
    /// she is Shiva.
    Serial,

    /// DeviceFactorSource can be used in parallel, since we can read
    /// multiple mnemonics from host at once and sign with them in
    /// the same scope / function.
    Parallel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignWithFactorSourceOrSourcesOutcome {
    Signed(IndexSet<SignatureByOwnedFactorForPayload>),
    Skipped,
    Interrupted(SignWithFactorSourceOrSourcesInterruption),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignWithFactorSourceOrSourcesInterruption {
    /// Timeout
    Timeout,
    /// User aborted
    UserAborted,
    /// Something went wrong.
    Failed,
}

#[async_trait::async_trait]
pub trait IsSigningDriver {
    /// The factor source kind of this signing driver.
    fn factor_source_kind(&self) -> FactorSourceKind;

    async fn sign<S: SignaturesBuilder>(
        &self,
        kind: FactorSourceKind,
        factor_sources: IndexSet<FactorSource>,
        signatures_builder: &S,
    ) -> Result<IndexSet<SignatureByOwnedFactorForPayload>>;
}

pub struct SigningDriverDeviceFactorSource;

impl SigningDriverDeviceFactorSource {
    fn factor_source_kind(&self) -> FactorSourceKind {
        FactorSourceKind::Device
    }

    async fn sign_parallel(
        &self,
        factor_sources: IndexSet<FactorSource>,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> SignWithFactorSourceOrSourcesOutcome {
        todo!()
    }

    async fn prompt_user_if_retry_with(
        &self,
        factor_sources: IndexSet<FactorSource>,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> bool {
        todo!()
    }
}

pub struct SigningDriverSerial {
    kind: FactorSourceKind,
}
impl SigningDriverSerial {
    pub fn new(kind: FactorSourceKind) -> Self {
        Self { kind }
    }
}

impl SigningDriverSerial {
    fn factor_source_kind(&self) -> FactorSourceKind {
        self.kind
    }

    async fn sign_serial(
        &self,
        factor_source: &FactorSource,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> SignWithFactorSourceOrSourcesOutcome {
        todo!()
    }

    async fn prompt_user_if_retry_with(
        &self,
        factor_source: &FactorSource,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> bool {
        todo!()
    }
}

pub enum SigningDriver {
    Parallel(SigningDriverDeviceFactorSource),
    Serial(SigningDriverSerial),
}

#[async_trait::async_trait]
impl IsSigningDriver for SigningDriver {
    fn factor_source_kind(&self) -> FactorSourceKind {
        match self {
            Self::Parallel(driver) => driver.factor_source_kind(),
            Self::Serial(driver) => driver.factor_source_kind(),
        }
    }

    async fn sign<S: SignaturesBuilder>(
        &self,
        kind: FactorSourceKind,
        factor_sources: IndexSet<FactorSource>,
        signatures_builder: &S,
    ) -> Result<IndexSet<SignatureByOwnedFactorForPayload>> {
        match self {
            Self::Parallel(driver) => {
                todo!()
                // let super_batched_intent_hashes = factor_sources
                //     .iter()
                //     .flat_map(|f| self.transactions_to_sign_with_factor_source(f))
                //     .collect::<IndexSet<_>>();

                // let super_batched_factor_instances = factor_sources
                //     .iter()
                //     .flat_map(|f| self.factor_instances_to_sign_with_using_factor_source(f))
                //     .collect::<IndexSet<_>>();

                // let outcome = signing_driver
                //     .sign_parallel(
                //         factor_sources,
                //         super_batched_intent_hashes,
                //         super_batched_factor_instances,
                //     )
                //     .await;

                // super_batched_signatures
                // self.add_signatures(super_batched_signatures)
            }
            Self::Serial(driver) => {
                let mut signatures_from_all_sources = IndexSet::new();
                for factor_source in factor_sources.iter() {
                    assert_eq!(factor_source.kind(), kind);

                    let do_sign = async || {
                        let batched_intent_hashes = signatures_builder
                            .transactions_to_sign_with_factor_source(factor_source);

                        let batched_factor_instances = signatures_builder
                            .factor_instances_to_sign_with_using_factor_source(factor_source);

                        driver
                            .sign_serial(
                                factor_source,
                                batched_intent_hashes,
                                batched_factor_instances,
                            )
                            .await
                    };

                    let sign_with_factor_source_outcome = do_sign().await;

                    match sign_with_factor_source_outcome {
                        SignWithFactorSourceOrSourcesOutcome::Signed(signatures) => {
                            signatures_from_all_sources.extend(signatures)
                        }
                        SignWithFactorSourceOrSourcesOutcome::Skipped => {
                            signatures_builder.skipped(IndexSet::from_iter([factor_source]))
                        }
                        SignWithFactorSourceOrSourcesOutcome::Interrupted(interruption) => {
                            match interruption {
                                SignWithFactorSourceOrSourcesInterruption::Failed => driver
                                    .prompt_user_if_retry_with(
                                        factor_source,
                                        intent_hashes,
                                        factor_instances,
                                    ),
                            }
                        }
                    }

                    // batched_signatures
                    // self.add_signatures(batched_signatures)
                }
            }
        }
        todo!()
    }
}

pub struct SigningDriversContext;
impl SigningDriversContext {
    pub fn driver_for_factor_source_kind(&self, kind: FactorSourceKind) -> SigningDriver {
        match kind {
            FactorSourceKind::Device => SigningDriver::Parallel(SigningDriverDeviceFactorSource),
            _ => SigningDriver::Serial(SigningDriverSerial::new(kind)),
        }
    }
}
