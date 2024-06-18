#![feature(async_closure)]

mod signatures_builders;
mod types;

pub mod prelude {
    pub use crate::signatures_builders::*;
    pub use crate::types::*;

    pub use std::collections::{HashMap, HashSet};

    pub use indexmap::{IndexMap, IndexSet};
}

pub use prelude::*;

#[cfg(test)]
impl SignaturesBuilderLevel0 {
    pub fn new_test(
        user: TestSigningUser,
        all_factor_sources_in_profile: impl IntoIterator<Item = FactorSource>,
        transactions: impl IntoIterator<Item = TransactionIntent>,
    ) -> Self {
        Self::new(
            SigningUser::Test(user),
            all_factor_sources_in_profile.into_iter().collect(),
            transactions.into_iter().collect(),
        )
    }
    pub fn test_prudent(
        all_factor_sources_in_profile: impl IntoIterator<Item = FactorSource>,
        transactions: impl IntoIterator<Item = TransactionIntent>,
    ) -> Self {
        Self::new_test(
            TestSigningUser::Prudent,
            all_factor_sources_in_profile,
            transactions,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_() {
        let mut context = SignaturesBuilderLevel0::test_prudent([], []);
        context.sign().await;
    }
}
