use itertools::Itertools;

use crate::{prelude::*, signatures_builders::redone_functional::signatures_builder};

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
    /// Timeout
    Timeout,
    /// User aborted
    UserAborted,
    /// Something went wrong.
    Failed,
}

pub struct SigningDriverParallell {
    kind: FactorSourceKind,
}

impl SigningDriverParallell {
    fn new(kind: FactorSourceKind) -> Self {
        Self { kind }
    }
    fn factor_source_kind(&self) -> FactorSourceKind {
        self.factor_source_kind()
    }

    async fn sign_parallel(
        &self,
        inputs: Vec<&SigningInputForFactorSource>,
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
        input: &SigningInputForFactorSource,
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
    Parallel(SigningDriverParallell),
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
    ) -> Result<IndexSet<SignatureByOwnedFactorForPayload>> {
        let inputs = signatures_builder.input_per_factors_source(factor_sources.clone());

        let mut outputs = IndexSet::<SignWithFactorSourceOrSourcesOutcome>::new();

        let mut reduce = |inputs: Vec<&SigningInputForFactorSource>,
                          output: SignWithFactorSourceOrSourcesOutcome|
         -> Result<()> {
            match output {
                SignWithFactorSourceOrSourcesOutcome::Interrupted(_) => todo!(),
                SignWithFactorSourceOrSourcesOutcome::Skipped => signatures_builder.skipped(
                    inputs
                        .into_iter()
                        .map(|i| i.factor_source.clone())
                        .collect::<IndexSet<_>>(),
                ),
                SignWithFactorSourceOrSourcesOutcome::Signed(signatures) => todo!(),
            };
            outputs.insert(output);
            todo!()
        };

        match self {
            Self::Parallel(driver) => {
                let inputs = inputs.values().into_iter().collect_vec();
                let output = driver.sign_parallel(inputs.clone()).await;
                reduce(inputs, output)?;
            }
            Self::Serial(driver) => {
                for factor_source in factor_sources.iter() {
                    assert_eq!(factor_source.kind(), kind);
                    let input = inputs.get(factor_source).unwrap();
                    let output = driver.sign_serial(input).await;
                    reduce(vec![input], output)?;
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
            FactorSourceKind::Device => SigningDriver::Parallel(SigningDriverParallell::new(kind)),
            _ => SigningDriver::Serial(SigningDriverSerial::new(kind)),
        }
    }
}
