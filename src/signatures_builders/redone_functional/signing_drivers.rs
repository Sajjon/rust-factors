use itertools::Itertools;

use crate::{
    prelude::*,
    signatures_builders::redone_functional::signatures_builder,
};

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

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub enum SignWithFactorSourceOrSourcesOutcome {
    Signed(Vec<SignatureByOwnedFactorForPayload>), // want IndexSet
    Skipped,
    Interrupted(SignWithFactorSourceOrSourcesInterruption),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignWithFactorSourceOrSourcesInterruption {
    /// User aborted
    UserAborted,

    /// Something went wrong or timed out.
    Failed,
}

#[async_trait::async_trait]
pub trait IsParallelSigningDriver {
    fn factor_source_kind(&self) -> FactorSourceKind;
    async fn sign_parallel(
        &self,
        inputs: Vec<&BatchTransactionSigningInputForFactorSource>,
    ) -> SignWithFactorSourceOrSourcesOutcome;
    async fn prompt_user_if_retry_with(
        &self,
        factor_sources: IndexSet<FactorSource>,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> bool;
}

pub struct SigningDriverParallel {
    kind: FactorSourceKind,
}

impl SigningDriverParallel {
    fn new(kind: FactorSourceKind) -> Self {
        Self { kind }
    }
}

#[async_trait::async_trait]
impl IsParallelSigningDriver for SigningDriverParallel {
    fn factor_source_kind(&self) -> FactorSourceKind {
        self.factor_source_kind()
    }

    async fn sign_parallel(
        &self,
        inputs: Vec<&BatchTransactionSigningInputForFactorSource>,
    ) -> SignWithFactorSourceOrSourcesOutcome {
        let mut signatures =
            Vec::<SignatureByOwnedFactorForPayload>::new();

        for x in inputs {
            let factor_source = x.factor_source.clone();
            let s = factor_source
                .batch_sign(x.input_for_each_transaction.clone())
                .await;
            signatures.extend(s);
        }

        SignWithFactorSourceOrSourcesOutcome::Signed(signatures)
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
        input: &BatchTransactionSigningInputForFactorSource,
    ) -> SignWithFactorSourceOrSourcesOutcome {
        let signatures = input
            .factor_source
            .batch_sign(input.input_for_each_transaction.clone())
            .await;

        SignWithFactorSourceOrSourcesOutcome::Signed(
            signatures.into_iter().collect_vec(),
        )
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
    Parallel(Box<dyn IsParallelSigningDriver>),
    Serial(SigningDriverSerial),
}

impl SigningDriver {
    pub fn factor_source_kind(&self) -> FactorSourceKind {
        match self {
            Self::Parallel(driver) => driver.factor_source_kind(),
            Self::Serial(driver) => driver.factor_source_kind(),
        }
    }

    pub async fn sign(
        &self,
        kind: FactorSourceKind,
        factor_sources: IndexSet<FactorSource>,
        signatures_builder: &SignaturesBuilder,
    ) {
        assert!(factor_sources.iter().all(|f| f.kind() == kind));

        match self {
            Self::Parallel(driver) => {
                let inputs = signatures_builder
                    .input_per_factors_source(factor_sources.clone());
                let output = driver
                    .sign_parallel(
                        inputs.values().into_iter().collect_vec(),
                    )
                    .await;
                signatures_builder
                    .process_outcome(output, factor_sources)
            }
            Self::Serial(driver) => {
                for factor_source in factor_sources.iter() {
                    let inputs = signatures_builder
                        .input_per_factors_source(
                            IndexSet::from_iter([
                                factor_source.clone()
                            ]),
                        );
                    let input = inputs.get(factor_source).unwrap();
                    let output = driver.sign_serial(input).await;
                    signatures_builder.process_outcome(
                        output,
                        IndexSet::from_iter([factor_source.clone()]),
                    )
                }
            }
        }
    }
}

pub trait IsSigningDriversContext {
    fn driver_for_factor_source_kind(
        &self,
        kind: FactorSourceKind,
    ) -> SigningDriver;
}

pub struct SigningDriversContext;
impl IsSigningDriversContext for SigningDriversContext {
    fn driver_for_factor_source_kind(
        &self,
        kind: FactorSourceKind,
    ) -> SigningDriver {
        match kind {
            FactorSourceKind::Device => SigningDriver::Parallel(
                Box::new(SigningDriverParallel::new(kind)),
            ),
            _ => {
                SigningDriver::Serial(SigningDriverSerial::new(kind))
            }
        }
    }
}
