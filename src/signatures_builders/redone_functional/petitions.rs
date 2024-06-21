use std::{cell::RefCell, collections::BTreeSet};

use crate::prelude::*;

pub(crate) struct Petitions {
    /// Lookup from factor to TXID.
    ///
    ///
    /// The same FactorSource might be required by many payloads
    /// and per payload might be required by many entities, e.g. transactions
    /// `t0` and `t1`, where
    /// `t0` is signed by accounts: A and B
    /// `t1` is signed by accounts: A, C and D,
    ///
    /// Where A, B, C and D, all use the factor source, e.g. some arculus
    /// card which the user has setup as a factor (source) for all these accounts.
    factor_to_txid: HashMap<FactorSourceID, IndexSet<IntentHash>>,

    /// Lookup from TXID to signatures builders, sorted according to the order of
    /// transactions passed to the SignaturesBuilder.
    txid_to_petition:
        RefCell<IndexMap<IntentHash, PetitionOfTransaction>>,
}
impl Petitions {
    pub(crate) fn new(
        factor_to_txid: HashMap<FactorSourceID, IndexSet<IntentHash>>,
        txid_to_petition: IndexMap<IntentHash, PetitionOfTransaction>,
    ) -> Self {
        Self {
            factor_to_txid,
            txid_to_petition: RefCell::new(txid_to_petition),
        }
    }
}

/// Essentially a wrapper around `IndexSet<PetitionOfTransactionByEntity>>`.
pub(crate) struct PetitionOfTransaction {
    /// Hash of transaction to sign
    intent_hash: IntentHash,

    for_entities: RefCell<BTreeSet<PetitionOfTransactionByEntity>>,
}
impl PetitionOfTransaction {
    pub(crate) fn new(
        intent_hash: IntentHash,
        for_entities: BTreeSet<PetitionOfTransactionByEntity>,
    ) -> Self {
        Self {
            intent_hash,
            for_entities: RefCell::new(for_entities),
        }
    }
}
