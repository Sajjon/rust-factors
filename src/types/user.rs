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

    async fn skip_next_and_all_remaining_factors_since_all_tx_are_already_valid(
        &self,
        next: &FactorSource,
    ) -> bool;
}

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

pub struct Laziness {
    act_sign_or_skip:
        Box<dyn Fn(&FactorSource, IndexSet<InvalidTransactionIfSkipped>) -> SigningUserInput>,

    act_done_skip_next_remaining: Box<dyn Fn(&FactorSource) -> bool>,
}

impl Laziness {
    pub fn new(
        act_sign_or_skip: impl Fn(&FactorSource, IndexSet<InvalidTransactionIfSkipped>) -> SigningUserInput
            + 'static,
        act_done_skip_next_remaining: impl Fn(&FactorSource) -> bool + 'static,
    ) -> Self {
        Self {
            act_sign_or_skip: Box::new(act_sign_or_skip),
            act_done_skip_next_remaining: Box::new(act_done_skip_next_remaining),
        }
    }
    pub fn always_skip() -> Self {
        Self::new(|_, _| SigningUserInput::Skip, |_| true)
    }
    /// Skips only if `invalid_tx_if_skipped` is empty
    pub fn sign_minimum() -> Self {
        Self::new(
            |_, invalid_tx_if_skipped| {
                if invalid_tx_if_skipped.is_empty() {
                    SigningUserInput::Skip
                } else {
                    SigningUserInput::Sign
                }
            },
            |_| true,
        )
    }
}

impl TestSigningUser {
    fn random_bool() -> bool {
        let mut rng = rand::thread_rng();
        let num: f64 = rng.gen(); // generates a float between 0 and 1
        num > 0.5
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
            TestSigningUser::Lazy(laziness) => {
                (laziness.act_sign_or_skip)(factor_source, invalid_tx_if_skipped)
            }
            TestSigningUser::Random => {
                if Self::random_bool() {
                    SigningUserInput::Skip
                } else {
                    SigningUserInput::Sign
                }
            }
        }
    }

    async fn skip_next_and_all_remaining_factors_since_all_tx_are_already_valid(
        &self,
        next: &FactorSource,
    ) -> bool {
        match self {
            TestSigningUser::Prudent => false,
            TestSigningUser::Lazy(laziness) => (laziness.act_done_skip_next_remaining)(&next),
            TestSigningUser::Random => Self::random_bool(),
        }
    }
}

pub enum SigningUser {
    Test(TestSigningUser),
}

unsafe impl Sync for TestSigningUser {}

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

    async fn skip_next_and_all_remaining_factors_since_all_tx_are_already_valid(
        &self,
        next: &FactorSource,
    ) -> bool {
        match self {
            SigningUser::Test(test_user) => {
                test_user
                    .skip_next_and_all_remaining_factors_since_all_tx_are_already_valid(&next)
                    .await
            }
        }
    }
}
