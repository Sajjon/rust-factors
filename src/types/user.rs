use crate::prelude::*;
use rand::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SigningUserInput {
    Sign,
    Skip,
}

#[async_trait::async_trait]
pub trait IsSigningUser {
    async fn sign_or_skip(
        &self,
        factor_source: &FactorSource,
        invalid_tx_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
    ) -> SigningUserInput;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestSigningUser {
    /// Emulation of a "prudent" user, that signs with all factors sources, i.e.
    /// she never ever "skips" a factor source
    Prudent,

    /// Emulation of a "lazy" user, that skips signing with as many factor
    /// sources as possible.
    Lazy(Laziness),

    /// Emulation of a "random" user, that skips signing some factor sources
    ///  at random.
    Random,
}
impl TestSigningUser {
    pub fn lazy_always_skip() -> Self {
        Self::Lazy(Laziness::always_skip())
    }
    /// Skips only if `invalid_tx_if_skipped` is empty
    pub fn lazy_sign_minimum() -> Self {
        Self::Lazy(Laziness::sign_minimum())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Laziness {
    act: fn(&FactorSource, IndexSet<InvalidTransactionIfSkipped>) -> SigningUserInput,
}
impl Laziness {
    pub fn new(
        act: fn(&FactorSource, IndexSet<InvalidTransactionIfSkipped>) -> SigningUserInput,
    ) -> Self {
        Self { act }
    }
    pub fn always_skip() -> Self {
        Self::new(|_, _| SigningUserInput::Skip)
    }
    /// Skips only if `invalid_tx_if_skipped` is empty
    pub fn sign_minimum() -> Self {
        Self::new(|_, invalid_tx_if_skipped| {
            if invalid_tx_if_skipped.is_empty() {
                println!("🙅🏻‍♀️ SHOULD SKIP!");
                SigningUserInput::Skip
            } else {
                println!("✍🏻 SIGNING!");
                SigningUserInput::Sign
            }
        })
    }
}

#[async_trait::async_trait]
impl IsSigningUser for TestSigningUser {
    async fn sign_or_skip(
        &self,
        factor_source: &FactorSource,
        invalid_tx_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
    ) -> SigningUserInput {
        match self {
            TestSigningUser::Prudent => SigningUserInput::Sign,
            TestSigningUser::Lazy(laziness) => (laziness.act)(factor_source, invalid_tx_if_skipped),
            TestSigningUser::Random => {
                let mut rng = rand::thread_rng();
                let num: f64 = rng.gen(); // generates a float between 0 and 1
                if num > 0.5 {
                    SigningUserInput::Skip
                } else {
                    SigningUserInput::Sign
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SigningUser {
    Test(TestSigningUser),
}

#[async_trait::async_trait]
impl IsSigningUser for SigningUser {
    async fn sign_or_skip(
        &self,
        factor_source: &FactorSource,
        invalid_tx_if_skipped: IndexSet<InvalidTransactionIfSkipped>,
    ) -> SigningUserInput {
        match self {
            SigningUser::Test(test_user) => {
                test_user
                    .sign_or_skip(&factor_source, invalid_tx_if_skipped)
                    .await
            }
        }
    }
}
