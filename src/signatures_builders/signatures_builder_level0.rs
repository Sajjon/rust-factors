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

    /// IDs of factor sources that the user has been prompted to sign with, which
    /// user either did sign with or skipped. This might be a subset of all
    /// factors in `factors_of_kind`, e.g. if user is signing a single transaction
    /// with threshold factors only, and the user skips so many factor sources
    /// that the transaction is invalid, we bail out early and the user is not
    /// prompted to sign with any more factors, resulting in `prompted_factor_sources`
    /// being a subset of `factors_of_kind`.
    ///
    /// Note that this set contains **both signed and skipped** factor sources,
    /// so if a user has signed with e.g. one override factor and skipped each
    /// individual remaining factor source, then this set will contain all those
    /// factor sources. However, if the user says "skip all remaining factors",
    /// those will not be included in this set (save the one prompted for when user
    /// pressed "skip all remaining factors" button).
    prompted_factor_sources: RefCell<IndexSet<FactorSourceID>>,

    /// If an intent hash is in this map, that transaction has failed, which can
    /// be determined just by the keys of this map, the values contains all the
    /// entities which fails auth.
    failed_transactions: RefCell<IndexMap<IntentHash, IndexSet<AccountAddressOrIdentityAddress>>>,
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
            prompted_factor_sources: RefCell::new(IndexSet::new()),
            failed_transactions: RefCell::new(IndexMap::new()),
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

    fn can_skip_all_remaining_factors_without_any_failing_tx(&self) -> bool {
        false
    }

    async fn do_sign(&self) -> () {
        let factors_of_kind = self.factors_of_kind.clone();
        for (kind, factor_sources) in factors_of_kind.into_iter() {
            for factor_source in factor_sources.iter() {
                assert_eq!(factor_source.kind(), kind);

                self.prompted_factor_sources
                    .borrow_mut()
                    .insert(factor_source.id.clone());

                if self.can_skip_all_remaining_factors_without_any_failing_tx() {
                    let is_skipping = self
                        .user
                        .skip_next_and_all_remaining_factors_since_all_tx_are_already_valid(
                            factor_source,
                        )
                        .await;
                    if is_skipping {
                        return;
                    }
                }

                let invalid_txs_if_skipped = self.invalid_if_skip_factor_source(factor_source);
                let is_skipping = match self
                    .user
                    .sign_or_skip(factor_source, invalid_txs_if_skipped.clone())
                    .await
                {
                    SigningUserInput::Skip => true,
                    SigningUserInput::Sign => false,
                };

                if is_skipping {
                    self.skip_factor_sources(factor_source)
                } else {
                    self.sign_with(factor_source).await
                }

                if is_skipping {
                    for invalid_tx_if_skipped in invalid_txs_if_skipped {
                        let mut invalid_transactions = self.failed_transactions.borrow_mut();
                        let key = invalid_tx_if_skipped.intent_hash;
                        if let Some(ref mut entities) = invalid_transactions.get_mut(&key) {
                            entities.extend(invalid_tx_if_skipped.entities_which_would_fail_auth);
                        } else {
                            invalid_transactions.insert(
                                key,
                                IndexSet::from_iter(
                                    invalid_tx_if_skipped.entities_which_would_fail_auth,
                                ),
                            );
                        }

                        assert!(!invalid_transactions.is_empty());
                    }
                }
            }
        }
    }

    pub async fn sign(&self) -> SigningOutcome {
        self.do_sign().await;
        SigningOutcome {
            all_signatures: self.signatures().clone(),
            prompted_factor_sources: self.prompted_factor_sources.borrow().clone(),
        }
    }
}
