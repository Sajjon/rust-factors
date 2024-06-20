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

    /// If a factor source kind can be used in parallel or serial manner.
    fn concurrency(&self) -> SigningFactorConcurrency;

    async fn prompt_user_if_retry_with(
        &self,
        factor_source: &FactorSource,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> bool {
        true
    }

    /// Sign a set of intents with a set of factor instances with a single
    /// factor source.
    async fn sign_serial(
        &self,
        factor_source: &FactorSource,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> SignWithFactorSourceOrSourcesOutcome {
        panic!("Should not have called sign_serial on a parallel driver")
    }

    /// Sign a set of intents with a set of factor instances for each
    /// factor source in factor sources.
    async fn sign_parallel(
        &self,
        factor_sources: IndexSet<FactorSource>,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> SignWithFactorSourceOrSourcesOutcome {
        panic!("Should not have called sign_parallel on a serial driver")
    }
}

pub struct SigningDriverDeviceFactorSource;

#[async_trait::async_trait]
impl IsSigningDriver for SigningDriverDeviceFactorSource {
    fn factor_source_kind(&self) -> FactorSourceKind {
        FactorSourceKind::Device
    }

    fn concurrency(&self) -> SigningFactorConcurrency {
        SigningFactorConcurrency::Parallel
    }

    async fn sign_parallel(
        &self,
        factor_sources: IndexSet<FactorSource>,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> SignWithFactorSourceOrSourcesOutcome {
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

#[async_trait::async_trait]
impl IsSigningDriver for SigningDriverSerial {
    fn factor_source_kind(&self) -> FactorSourceKind {
        self.kind
    }

    fn concurrency(&self) -> SigningFactorConcurrency {
        SigningFactorConcurrency::Serial
    }

    async fn sign_serial(
        &self,
        factor_source: &FactorSource,
        intent_hashes: IndexSet<&IntentHash>,
        factor_instances: IndexSet<&OwnedFactorInstance>,
    ) -> SignWithFactorSourceOrSourcesOutcome {
        todo!()
    }
}

pub enum SigningDriver {
    Parallel(SigningDriverDeviceFactorSource),
    Serial(SigningDriverSerial),
}

impl IsSigningDriver for SigningDriver {
    fn factor_source_kind(&self) -> FactorSourceKind {
        match self {
            Self::Parallel(driver) => driver.factor_source_kind(),
            Self::Serial(driver) => driver.factor_source_kind(),
        }
    }

    fn concurrency(&self) -> SigningFactorConcurrency {
        match self {
            Self::Parallel(driver) => driver.concurrency(),
            Self::Serial(driver) => driver.concurrency(),
        }
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
