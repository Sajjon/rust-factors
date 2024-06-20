use std::cell::RefCell;

use crate::prelude::*;
use itertools::Itertools;

/// Root Signing Context: Aggregates over multiple Transactions.
pub struct SignaturesBuilderLevel0 {
    /// Abstraction of a user signing, decides for every factor source if
    /// she wants to skip signing with the factor source if she can,
    /// or cancel the whole signing process (context) or otherwise
    /// asynchronously sign.
    user: SigningUser,

    /// Factor sources grouped by kind, sorted according to "signing order",
    /// that is, we want to control which factor source kind users signs with
    /// first, second etc, e.g. typically we prompt user to sign with Ledgers
    /// first, and if a user might lack access to that Ledger device, then it is
    /// best to "fail fast", otherwise we might waste the users time, if she has
    /// e.g. answered security questions and then is asked to sign with a Ledger
    /// she might not have handy at the moment - or might not be in front of a
    /// computer and thus unable to make a connection between the Radix Wallet
    /// and a Ledger device.
    factors_of_kind: IndexMap<FactorSourceKind, IndexSet<FactorSource>>,

    /// Lookup payloads that need to be signed with a factor source by its
    /// FactorSourceID. By "payload" we mean a transaction of a ROLA challenge.
    /// We support signing of multiple transactions in one go, therefore, the
    /// plural form of payload**s**.
    ///
    /// The same FactorSource might be required by many payloads
    /// and per payload might be required by many entities, e.g. transactions
    /// `t0` and `t1`, where
    /// `t0` is signed by accounts: A and B
    /// `t1` is signed by accounts: A, C and D,
    ///
    /// Where A, B, C and D, all use the factor source, e.g. some arculus
    /// card which the user has setup as a factor (source) for all these accounts.
    factor_to_payloads: HashMap<FactorSourceID, IndexSet<IntentHash>>,

    /// Lookup from payload (TXID) to signatures builders.
    builders_level_0: RefCell<HashMap<IntentHash, SignaturesBuilderLevel1>>,
}

impl SignaturesBuilderLevel0 {
    pub fn new(
        user: SigningUser,
        all_factor_sources_in_profile: IndexSet<FactorSource>,
        transactions: IndexSet<TransactionIntent>,
    ) -> Self {
        let mut builders_level_0 = HashMap::<IntentHash, SignaturesBuilderLevel1>::new();

        let all_factor_sources_in_profile = all_factor_sources_in_profile
            .into_iter()
            .map(|f| (f.id, f))
            .collect::<HashMap<FactorSourceID, FactorSource>>();

        let mut factor_to_payloads = HashMap::<FactorSourceID, IndexSet<IntentHash>>::new();

        let mut used_factor_sources = HashSet::<FactorSource>::new();

        let mut use_factor_in_tx = |id: &FactorSourceID, txid: &IntentHash| {
            if let Some(ref mut txids) = factor_to_payloads.get_mut(id) {
                txids.insert(txid.clone());
            } else {
                factor_to_payloads.insert(id.clone(), IndexSet::from_iter([txid.clone()]));
            }

            assert!(!factor_to_payloads.is_empty());

            let factor_source = all_factor_sources_in_profile
                .get(id)
                .expect("Should have all factor sources");
            used_factor_sources.insert(factor_source.clone());

            assert!(!used_factor_sources.is_empty());
        };

        for transaction in transactions {
            let mut builders_level_2 =
                HashMap::<AccountAddressOrIdentityAddress, SignaturesBuilderLevel2>::new();

            for entity in transaction.clone().entities_requiring_auth {
                let address = entity.address;
                match entity.security_state {
                    EntitySecurityState::Securified(sec) => {
                        let primary_role_matrix = sec;

                        let mut add = |factors: Vec<FactorInstance>| {
                            factors.into_iter().for_each(|f| {
                                let factor_source_id = f.factor_source_id;
                                use_factor_in_tx(&factor_source_id, &transaction.intent_hash);
                            })
                        };

                        add(primary_role_matrix.override_factors.clone());
                        add(primary_role_matrix.threshold_factors.clone());

                        let builder = SignaturesBuilderLevel2::new_securified(
                            address.clone(),
                            primary_role_matrix,
                        );
                        builders_level_2.insert(address.clone(), builder);
                    }
                    EntitySecurityState::Unsecured(uec) => {
                        let factor_instance = uec;
                        let factor_source_id = factor_instance.factor_source_id;
                        use_factor_in_tx(&factor_source_id, &transaction.intent_hash);

                        let builder = SignaturesBuilderLevel2::new_unsecurified(
                            address.clone(),
                            factor_instance,
                        );
                        builders_level_2.insert(address.clone(), builder);
                    }
                }
            }
            builders_level_0.insert(
                transaction.intent_hash.clone(),
                SignaturesBuilderLevel1::new(transaction.intent_hash.clone(), builders_level_2),
            );
        }

        let factors_of_kind = used_factor_sources
            .into_iter()
            .into_grouping_map_by(|x| x.kind())
            .collect::<IndexSet<FactorSource>>();

        let mut factors_of_kind = factors_of_kind
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().sorted().collect::<IndexSet<_>>()))
            .collect::<IndexMap<FactorSourceKind, IndexSet<FactorSource>>>();

        factors_of_kind.sort_keys();

        let self_ = Self {
            user,
            builders_level_0: builders_level_0.into(),
            factors_of_kind,
            factor_to_payloads,
        };

        // println!("\n\nuser: {:?}", &self_.user);
        // {
        //     println!(
        //         "\n\nbuilders_level_0: {:?}",
        //         &self_
        //             .builders_level_0
        //             .borrow()
        //             .iter()
        //             .map(|(k, v)| format!("k: {:?} => v: {:?}", &k, &v))
        //             .join("\n")
        //     );
        // }
        // println!(
        //     "\n\nfactors_of_kind: {:?}",
        //     &self_
        //         .factors_of_kind
        //         .iter()
        //         .map(|(k, v)| format!("k: {:?} => v: {:?}", &k, &v))
        //         .join("\n")
        // );
        // println!(
        //     "\n\nfactor_to_payloads: {:?}",
        //     &self_
        //         .factor_to_payloads
        //         .iter()
        //         .map(|(k, v)| format!("k: {:?} => v: {:?}", &k, &v))
        //         .join("\n")
        // );

        self_
    }
}

impl IsSignaturesBuilder for SignaturesBuilderLevel0 {
    type InvalidIfSkipped = InvalidTransactionIfSkipped;

    fn invalid_if_skip_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> IndexSet<Self::InvalidIfSkipped> {
        let tx_ids = self
            .factor_to_payloads
            .get(&factor_source.id)
            .expect(&format!(
                "Nil found when unwrapping factor_to_payloads by factor_source: '{:?}'",
                &factor_source.id
            ));

        tx_ids
            .into_iter()
            .flat_map(|txid| {
                self.builders_level_0
                    .borrow()
                    .get(txid)
                    .unwrap()
                    .invalid_if_skip_factor_source(factor_source)
            })
            .collect::<IndexSet<_>>()
    }

    fn skip_factor_sources(&self, factor_source: &FactorSource) {
        let tx_ids = self.factor_to_payloads.get(&factor_source.id).unwrap();

        let mut builders_level_0 = self.builders_level_0.borrow_mut();

        tx_ids.into_iter().for_each(|txid| {
            builders_level_0
                .get_mut(txid)
                .unwrap()
                .skip_factor_sources(factor_source)
        });

        drop(builders_level_0);
    }

    fn append_signature(&self, signature: SignatureByOwnedFactorForPayload) {
        let mut builders_level_0 = self.builders_level_0.borrow_mut();

        builders_level_0
            .get_mut(&signature.intent_hash)
            .unwrap()
            .append_signature(signature.clone());

        drop(builders_level_0);
    }

    fn signatures(&self) -> IndexSet<SignatureByOwnedFactorForPayload> {
        self.builders_level_0
            .borrow()
            .values()
            .into_iter()
            .flat_map(|builders_level_1| builders_level_1.signatures())
            .collect()
    }

    fn has_fulfilled_signatures_requirement(&self) -> bool {
        self.builders_level_0
            .borrow()
            .values()
            .all(|builders_level_1| builders_level_1.has_fulfilled_signatures_requirement())
    }
}

impl SignaturesBuilderLevel0 {
    async fn sign_with(&self, factor_source: &FactorSource) {
        let mut signatures = IndexSet::<SignatureByOwnedFactorForPayload>::new();
        {
            let factor_source_id = &factor_source.id;

            let builders_level_0 = self.builders_level_0.borrow();
            for intent_hash in self
                .factor_to_payloads
                .get(factor_source_id)
                .unwrap()
                .iter()
            {
                let signatures_builder = builders_level_0.get(intent_hash).unwrap();
                let owned_instances =
                    signatures_builder.owned_instances_of_factor_source(factor_source_id);
                let sigs = factor_source.batch_sign(intent_hash, owned_instances).await;
                signatures.extend(sigs);
            }
        }
        signatures
            .into_iter()
            .for_each(|s| self.append_signature(s));
    }

    pub async fn sign(&self) -> SignaturesOutcome {
        let factors_of_kind = self.factors_of_kind.clone();
        for (kind, factor_sources) in factors_of_kind.into_iter() {
            for factor_source in factor_sources.iter() {
                assert_eq!(factor_source.kind(), kind);
            }
        }
        SignaturesOutcome::new(
            MaybeSignedTransactions::new(IndexMap::new()),
            MaybeSignedTransactions::new(IndexMap::new()),
        )
    }
}
