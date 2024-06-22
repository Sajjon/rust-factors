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

    /// Used by "Parallel" SigningDrivers
    pub fn input_per_factors_source(
        &self,
        factor_sources: IndexSet<FactorSource>,
    ) -> IndexMap<
        FactorSource,
        BatchTransactionSigningInputForFactorSource,
    > {
        todo!()
    }

    /// Used by "Serial" SigningDrivers
    pub fn input_for_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> BatchTransactionSigningInputForFactorSource {
        let intent_hashes =
            self.factor_to_txid.get(&factor_source.id).unwrap();

        let input_for_each_transaction: IndexMap<IntentHash, SigningInputForFactorSource> = intent_hashes.into_iter().map(|txid| {
            let petition =
                self.txid_to_petition.borrow().get(txid).unwrap();
            let instances = petition.all_factor_instances();
            let entities_which_would_fail_auth = petition
                .for_entities
                .borrow()
                .iter()
                .filter_map(|petition| {
                    if petition.would_fail_auth() {
                        Some(petition.entity.clone())
                    } else {
                        None
                    }
                })
                .collect();
            let v = SigningInputForFactorSource::new(
                factor_source,
                txid,
                instances,
                entities_which_would_fail_auth,
            );
            (txid, v)
        }).collect::<IndexMap<IntentHash, SigningInputForFactorSource>>();

        BatchTransactionSigningInputForFactorSource::new(
            factor_source.clone(),
            input_for_each_transaction,
        );
    }
}

/// Essentially a wrapper around `IndexSet<PetitionOfTransactionByEntity>>`.
pub(crate) struct PetitionOfTransaction {
    /// Hash of transaction to sign
    pub intent_hash: IntentHash,

    pub for_entities:
        RefCell<BTreeSet<PetitionOfTransactionByEntity>>,
}

impl PetitionOfTransaction {
    pub fn all_factor_instances(&self) -> IndexSet<FactorInstance> {
        self.for_entities
            .borrow()
            .iter()
            .flat_map(|petition| petition.all_factor_instances())
            .collect()
    }
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
