use crate::{unique_payments::UniquePayments, user_actions::validator::INVALID_MAX_AMOUNT_ERR_MSG};

use super::common_actions::AddDelegationArgs;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type Epoch = u64;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct SovereignInfo<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub opt_max_restaking_cap: Option<BigUint<M>>,
}

impl<M: ManagedTypeApi> SovereignInfo<M> {
    #[inline]
    pub fn new(name: ManagedBuffer<M>, description: ManagedBuffer<M>) -> Self {
        Self {
            name,
            description,
            opt_max_restaking_cap: Option::None,
        }
    }
}

#[multiversx_sc::module]
pub trait SovereignModule:
    crate::token_whitelist::TokenWhitelistModule
    + crate::user_actions::common_actions::CommonActionsModule
    + crate::user_actions::common_storage::CommonStorageModule
    + crate::events::sov_events::SovEventsModule
    + utils::UtilsModule
{
    #[endpoint(registerSov)]
    fn register_sov(&self, name: ManagedBuffer, description: ManagedBuffer) {
        self.require_not_empty_buffer(&name);

        let caller = self.blockchain().get_caller();
        let caller_id = self.sov_id().insert_new(&caller);

        let id_for_name_mapper = self.sov_chain_for_name(&name);
        require!(id_for_name_mapper.is_empty(), "Name already taken");

        let sov_info = SovereignInfo::new(name, description);
        self.sov_info(caller_id).set(&sov_info);
        id_for_name_mapper.set(caller_id);

        self.emit_sov_register_event(caller, sov_info);
    }

    #[endpoint(setUpRewards)]
    fn set_up_rewards(
        &self,
        _start_epoch: Epoch,
        _end_epoch: Epoch,
        _total_value: BigUint,
        /* _computation: ???, */
    ) {
        let caller = self.blockchain().get_caller();
        let _caller_id = self.sov_id().get_id_non_zero(&caller);
        // TODO: Unsure what to do with all this info yet
    }

    #[endpoint(unRegister)]
    fn unregister(&self) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.sov_id().remove_by_address(&caller);
        require!(caller_id != NULL_ID, "Unknown sovereign chain");

        let sov_info = self.sov_info(caller_id).take();
        self.sov_chain_for_name(&sov_info.name).clear();

        self.emit_sov_unregister_event(caller);
    }

    #[payable("*")]
    #[endpoint(addRewards)]
    fn add_rewards(&self) {
        // TODO
    }

    #[payable("*")]
    #[endpoint(addOwnSecurityFunds)]
    fn add_own_security_funds(&self) {
        let sov_chain = self.blockchain().get_caller();
        let sov_id = self.sov_id().get_id_non_zero(&sov_chain);
        let sov_info = self.sov_info(sov_id).get();
        let user_id_of_sov_chain = self.user_ids().get_id_or_insert(&sov_chain);

        let payments = self.get_non_empty_payments();
        let mut total = BigUint::zero();
        for payment in &payments {
            self.require_token_in_whitelist(&payment.token_identifier);

            total += self.get_total_staked_egld(&payment.token_identifier, &payment.amount);
        }

        let args = AddDelegationArgs {
            total_delegated_mapper: self.total_delegated_sov_amount(sov_id),
            total_by_user_mapper: self.total_sov_by_user(user_id_of_sov_chain, sov_id),
            all_delegators_mapper: &mut self.all_sov_delegators(sov_id),
            delegated_by_mapper: self.delegated_sov_by(user_id_of_sov_chain, sov_id),
            opt_max_delegation: sov_info.opt_max_restaking_cap,
            payments_to_add: payments.clone(),
            total_amount: total,
            caller_id: user_id_of_sov_chain,
        };
        self.add_delegation(args);

        self.sov_add_own_security_funds_event(sov_chain, payments);
    }

    #[endpoint(setMaxReStakingCap)]
    fn set_max_restaking_cap(&self, max_cap: BigUint) {
        let caller = self.blockchain().get_caller();
        let sov_id = self.sov_id().get_id_non_zero(&caller);
        self.sov_info(sov_id).update(|sov_info| {
            let current_total = self.total_delegated_sov_amount(sov_id).get();
            require!(max_cap >= current_total, INVALID_MAX_AMOUNT_ERR_MSG);

            sov_info.opt_max_restaking_cap = Some(max_cap.clone());
        });

        self.emit_sov_set_max_restaking_cap_event(caller, max_cap);
    }

    #[view(getSovInfo)]
    fn get_sov_info(&self, sov_address: ManagedAddress) -> SovereignInfo<Self::Api> {
        let sov_id = self.sov_id().get_id_non_zero(&sov_address);

        self.sov_info(sov_id).get()
    }

    fn require_valid_sov_id(&self, sov_id: AddressId) {
        require!(sov_id != NULL_ID, "Invalid chain name");
    }

    #[storage_mapper("sovId")]
    fn sov_id(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("sovInfo")]
    fn sov_info(&self, sov_id: AddressId) -> SingleValueMapper<SovereignInfo<Self::Api>>;

    #[storage_mapper("sovForName")]
    fn sov_chain_for_name(&self, name: &ManagedBuffer) -> SingleValueMapper<AddressId>;

    #[storage_mapper("allSovDelegators")]
    fn all_sov_delegators(&self, sov_id: AddressId) -> UnorderedSetMapper<AddressId>;

    #[storage_mapper("delegatedSovBy")]
    fn delegated_sov_by(
        &self,
        user_id: AddressId,
        sov_id: AddressId,
    ) -> SingleValueMapper<UniquePayments<Self::Api>>;

    #[storage_mapper("totalDelegatedSovAmount")]
    fn total_delegated_sov_amount(&self, sov_id: AddressId) -> SingleValueMapper<BigUint>;

    #[storage_mapper("totalSovByUser")]
    fn total_sov_by_user(
        &self,
        user_id: AddressId,
        sov_id: AddressId,
    ) -> SingleValueMapper<BigUint>;
}
