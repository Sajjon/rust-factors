use crate::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SignaturesOutcome {
    successful_transactions: MaybeSignedTransactions,
    failed_transactions: MaybeSignedTransactions,
}
impl SignaturesOutcome {
    pub fn new(
        successful_transactions: MaybeSignedTransactions,
        failed_transactions: MaybeSignedTransactions,
    ) -> Self {
        Self {
            successful_transactions,
            failed_transactions,
        }
    }

    /// All signatures from both successful transactions and failed transactions.
    pub fn all_signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.successful_transactions
            .all_signatures()
            .union(&self.failed_transactions.all_signatures())
            .cloned()
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MaybeSignedTransactions {
    transactions: IndexMap<IntentHash, IndexSet<SignatureByOwnedFactorForPayload>>,
}

impl MaybeSignedTransactions {
    /// # Panics
    /// Panics if any of the signatures in the transactions list have an intent
    /// hash which does not match its key in the transactions map.
    pub fn new(
        transactions: IndexMap<IntentHash, IndexSet<SignatureByOwnedFactorForPayload>>,
    ) -> Self {
        transactions
            .iter()
            .for_each(|(k, v)| assert!(v.iter().all(|s| s.intent_hash == *k)));

        Self { transactions }
    }
    pub fn all_signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.transactions
            .values()
            .flat_map(|v| v.iter())
            .cloned()
            .collect()
    }
}
