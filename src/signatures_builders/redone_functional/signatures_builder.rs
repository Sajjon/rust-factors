use std::{cell::RefCell, collections::BTreeSet};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningInputForFactorSource {
    pub factor_source: FactorSource,
    pub intent_hash: IntentHash,
    pub factor_instances: IndexSet<OwnedFactorInstance>,
    pub entities_which_would_fail_auth:
        Vec<AccountAddressOrIdentityAddress>,
}
impl SigningInputForFactorSource {
    pub fn new(
        factor_source: FactorSource,
        intent_hash: IntentHash,
        factor_instances: IndexSet<OwnedFactorInstance>,
        entities_which_would_fail_auth: Vec<
            AccountAddressOrIdentityAddress,
        >,
    ) -> Self {
        Self {
            factor_source,
            intent_hash,
            factor_instances,
            entities_which_would_fail_auth,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchTransactionSigningInputForFactorSource {
    pub factor_source: FactorSource,
    pub input_for_each_transaction:
        IndexMap<IntentHash, SigningInputForFactorSource>,
}

impl BatchTransactionSigningInputForFactorSource {
    pub fn new(
        factor_source: FactorSource,
        input_for_each_transaction: IndexMap<
            IntentHash,
            SigningInputForFactorSource,
        >,
    ) -> Self {
        Self {
            factor_source,
            input_for_each_transaction,
        }
    }

    pub fn invalid_transactions_if_skipped(
        &self,
    ) -> IndexSet<InvalidTransactionIfSkipped> {
        todo!()
    }
}

pub struct SignaturesBuilder {
    signing_drivers_context: SigningDriversContext,
    /// Factor sources grouped by kind, sorted according to "signing order",
    /// that is, we want to control which factor source kind users signs with
    /// first, second etc, e.g. typically we prompt user to sign with Ledgers
    /// first, and if a user might lack access to that Ledger device, then it is
    /// best to "fail fast", otherwise we might waste the users time, if she has
    /// e.g. answered security questions and then is asked to sign with a Ledger
    /// she might not have handy at the moment - or might not be in front of a
    /// computer and thus unable to make a connection between the Radix Wallet
    /// and a Ledger device.
    factors_of_kind:
        IndexMap<FactorSourceKind, IndexSet<FactorSource>>,

    /// Signature builders for for each factor source, for each transaction,
    /// for each entity to sign.
    petitions: RefCell<Petitions>,
}

impl SignaturesBuilder {
    pub fn new(
        all_factor_sources_in_profile: IndexSet<FactorSource>,
        transactions: IndexSet<TransactionIntent>,
        signing_drivers_context: SigningDriversContext,
    ) -> Self {
        let mut petitions_for_all_transactions =
            IndexMap::<IntentHash, PetitionOfTransaction>::new();

        let all_factor_sources_in_profile =
            all_factor_sources_in_profile
                .into_iter()
                .map(|f| (f.id, f))
                .collect::<HashMap<FactorSourceID, FactorSource>>();

        let mut factor_to_payloads =
            HashMap::<FactorSourceID, IndexSet<IntentHash>>::new();

        let mut used_factor_sources = HashSet::<FactorSource>::new();

        let mut use_factor_in_tx =
            |id: &FactorSourceID, txid: &IntentHash| {
                if let Some(ref mut txids) =
                    factor_to_payloads.get_mut(id)
                {
                    txids.insert(txid.clone());
                } else {
                    factor_to_payloads.insert(
                        id.clone(),
                        IndexSet::from_iter([txid.clone()]),
                    );
                }

                assert!(!factor_to_payloads.is_empty());

                let factor_source = all_factor_sources_in_profile
                    .get(id)
                    .expect("Should have all factor sources");
                used_factor_sources.insert(factor_source.clone());

                assert!(!used_factor_sources.is_empty());
            };

        for (index, transaction) in
            transactions.into_iter().enumerate()
        {
            let transaction_index = TransactionIndex::new(
                index,
                transaction.intent_hash.clone(),
            );

            let mut petitions_for_entities =
                BTreeSet::<PetitionOfTransactionByEntity>::new();

            for entity in transaction.clone().entities_requiring_auth
            {
                let address = entity.address;
                match entity.security_state {
                    EntitySecurityState::Securified(sec) => {
                        let primary_role_matrix = sec;

                        let mut add =
                            |factors: Vec<FactorInstance>| {
                                factors.into_iter().for_each(|f| {
                                    let factor_source_id =
                                        f.factor_source_id;
                                    use_factor_in_tx(
                                        &factor_source_id,
                                        &transaction.intent_hash,
                                    );
                                })
                            };

                        add(primary_role_matrix
                            .override_factors
                            .clone());
                        add(primary_role_matrix
                            .threshold_factors
                            .clone());

                        // let builder = SignaturesBuilderLevel2::new_securified(
                        //     address.clone(),
                        //     primary_role_matrix,
                        // );
                        // builders_level_2.insert(address.clone(), builder);
                        let petition = PetitionOfTransactionByEntity::new_securified(
                            transaction_index.clone(),
                            address.clone(),
                            primary_role_matrix,
                        );
                        petitions_for_entities.insert(petition);
                    }
                    EntitySecurityState::Unsecured(uec) => {
                        let factor_instance = uec;
                        let factor_source_id =
                            factor_instance.factor_source_id;
                        use_factor_in_tx(
                            &factor_source_id,
                            &transaction.intent_hash,
                        );

                        // let builder =
                        //     SignaturesBuilderLevel2::new_unsecurified(
                        //         address.clone(),
                        //         factor_instance,
                        //     );
                        // builders_level_2
                        //     .insert(address.clone(), builder);

                        let petition = PetitionOfTransactionByEntity::new_unsecurified(
                            transaction_index.clone(),
                            address.clone(),
                            factor_instance
                        );
                        petitions_for_entities.insert(petition);
                    }
                }
            }
            // builders_level_0.insert(
            //     transaction.intent_hash.clone(),
            //     SignaturesBuilderLevel1::new(
            //         transaction.intent_hash.clone(),
            //         builders_level_2,
            //     ),
            // );

            let petition_of_tx = PetitionOfTransaction::new(
                transaction.intent_hash.clone(),
                petitions_for_entities,
            );

            petitions_for_all_transactions
                .insert(transaction.intent_hash, petition_of_tx);
        }

        let factors_of_kind = used_factor_sources
            .into_iter()
            .into_grouping_map_by(|x| x.kind())
            .collect::<IndexSet<FactorSource>>();

        let mut factors_of_kind = factors_of_kind
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .sorted()
                        .collect::<IndexSet<_>>(),
                )
            })
            .collect::<IndexMap<
                FactorSourceKind,
                IndexSet<FactorSource>,
            >>();

        factors_of_kind.sort_keys();

        let petitions = Petitions::new(
            factor_to_payloads,
            petitions_for_all_transactions,
        );

        Self {
            signing_drivers_context,
            factors_of_kind,
            petitions: petitions.into(),
        }
    }
}

impl SignaturesBuilder {
    /// If all transactions already would fail, or if all transactions already are done, then
    /// no point in continuing.
    fn continue_if_necessary(&self) -> Result<()> {
        todo!()
    }

    async fn do_sign(&self) -> Result<()> {
        let factors_of_kind = self.factors_of_kind.clone();
        for (kind, factor_sources) in factors_of_kind.into_iter() {
            let signing_driver = self
                .signing_drivers_context
                .driver_for_factor_source_kind(kind);

            signing_driver.sign(kind, factor_sources, self).await;

            self.continue_if_necessary()?;
        }
        Ok(())
    }
}

impl SignaturesBuilder {
    /// Used by "Parallel" SigningDrivers
    pub(super) fn input_per_factors_source(
        &self,
        factor_sources: IndexSet<FactorSource>,
    ) -> IndexMap<
        FactorSource,
        BatchTransactionSigningInputForFactorSource,
    > {
        self.petitions
            .borrow()
            .input_per_factors_source(factor_sources)
    }

    /// Used by "Serial" SigningDrivers
    pub(super) fn input_for_factor_source(
        &self,
        factor_source: &FactorSource,
    ) -> BatchTransactionSigningInputForFactorSource {
        self.petitions
            .borrow()
            .input_for_factor_source(factor_source)
    }

    pub(super) fn process(
        &self,
        outcome: SignWithFactorSourceOrSourcesOutcome,
    ) {
        todo!()
    }
}

impl SignaturesBuilder {
    pub async fn sign(&self) -> Result<SignaturesOutcome> {
        self.do_sign().await?;
        let outcome = SignaturesOutcome::new(
            MaybeSignedTransactions::new(IndexMap::new()),
            MaybeSignedTransactions::new(IndexMap::new()),
        );
        Ok(outcome)
    }
}
