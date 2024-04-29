use crate::unique_payments::UniquePayments;

use super::common_actions::AddDelegationArgs;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const BLS_KEY_LEN: usize = 96;
const BLS_SIG_LEN: usize = 48;
const MAX_PERCENT: Percent = 10_000;

pub static INVALID_MAX_AMOUNT_ERR_MSG: &[u8] = b"Cannot set max below the current delegated amount";

pub type BlsKey<M> = ManagedByteArray<M, BLS_KEY_LEN>;
pub type BlsSignature<M> = ManagedByteArray<M, BLS_SIG_LEN>;
pub type Percent = u32;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct ValidatorConfig<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub bls_keys: ManagedVec<M, BlsKey<M>>,
    pub fee: Percent,
    pub opt_max_delegation: Option<BigUint<M>>,
}

impl<M: ManagedTypeApi> ValidatorConfig<M> {
    pub fn new(name: ManagedBuffer<M>) -> Self {
        Self {
            name,
            bls_keys: ManagedVec::new(),
            fee: 0,
            opt_max_delegation: None,
        }
    }
}

#[multiversx_sc::module]
pub trait ValidatorModule:
    crate::token_whitelist::TokenWhitelistModule
    + crate::user_actions::common_actions::CommonActionsModule
    + crate::user_actions::common_storage::CommonStorageModule
    + crate::events::validator_events::ValidatorEventsModule
    + utils::UtilsModule
{
    #[endpoint]
    fn register(&self, name: ManagedBuffer) {
        self.require_not_empty_buffer(&name);

        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().insert_new(&caller);

        let id_for_name_mapper = self.id_for_name(&name);
        require!(id_for_name_mapper.is_empty(), "Name already taken");

        self.validator_config(caller_id)
            .set(ValidatorConfig::new(name.clone()));
        id_for_name_mapper.set(caller_id);

        self.emit_validator_register_event(caller, name);
    }

    /// pairs of bls_key and signed message of own address
    #[endpoint(addKeys)]
    fn add_keys(
        &self,
        pairs: MultiValueEncoded<MultiValue2<BlsKey<Self::Api>, BlsSignature<Self::Api>>>,
    ) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().get_id_non_zero(&caller);

        let mut new_bls_keys = ManagedVec::<Self::Api, _>::new();
        for pair in pairs {
            let (bls_key, bls_sig) = pair.into_tuple();
            let valid_sig = self.crypto().verify_bls(
                bls_key.as_managed_buffer(),
                caller.as_managed_buffer(),
                bls_sig.as_managed_buffer(),
            );
            require!(valid_sig, "Invalid BLS signature");

            new_bls_keys.push(bls_key);
        }

        let keys_added = new_bls_keys.clone();
        self.validator_config(caller_id).update(|config| {
            while !new_bls_keys.is_empty() {
                let current_key = new_bls_keys.get(0);
                for existing_key in &config.bls_keys {
                    require!(existing_key != *current_key, "Key already known");
                }

                config.bls_keys.push((*current_key).clone());
                new_bls_keys.remove(0);
            }
        });

        self.emit_validator_add_bls_keys_event(caller, keys_added);
    }

    #[endpoint(removeKeys)]
    fn remove_keys(&self, keys: MultiValueEncoded<BlsKey<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().get_id_non_zero(&caller);

        let mapper = self.validator_config(caller_id);
        let mut config = mapper.get();
        let mut removed_keys = ManagedVec::new();
        for key in keys {
            let opt_index = config.bls_keys.find(&key);
            require!(opt_index.is_some(), "Key not found");

            let index = unsafe { opt_index.unwrap_unchecked() };
            config.bls_keys.remove(index);
            removed_keys.push(key);
        }

        mapper.set(config);

        self.emit_validator_remove_bls_keys_event(caller, removed_keys);
    }

    // TODO: validateFor@projectID@LIST<BLSKEYS>@LISTOFStakeEGLDAssets

    #[endpoint(setUpFee)]
    fn set_up_fee(&self, fee: Percent) {
        require!(fee <= MAX_PERCENT, "Invalid fee percent");

        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().get_id_non_zero(&caller);
        self.validator_config(caller_id)
            .update(|config| config.fee = fee);

        self.emit_validator_set_fee_event(caller, fee);
    }

    #[endpoint(setMaxDelegation)]
    fn set_max_delegation(&self, max_delegation: BigUint) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().get_id_non_zero(&caller);
        self.validator_config(caller_id).update(|config| {
            let current_total = self.total_delegated_amount(caller_id).get();
            require!(max_delegation >= current_total, INVALID_MAX_AMOUNT_ERR_MSG);

            config.opt_max_delegation = Some(max_delegation.clone());
        });

        self.emit_validator_set_max_delegation_event(caller, max_delegation);
    }

    #[payable("*")]
    #[endpoint(addOwnDelegation)]
    fn add_own_delegation(&self) {
        let validator = self.blockchain().get_caller();
        let validator_id = self.validator_id().get_id_non_zero(&validator);
        let user_id_of_validator = self.user_ids().get_id_or_insert(&validator);
        let validator_config = self.validator_config(validator_id).get();

        let payments = self.get_non_empty_payments();
        let mut total = BigUint::zero();
        for payment in &payments {
            self.require_token_in_whitelist(&payment.token_identifier);

            total += self.get_total_staked_egld(&payment.token_identifier, &payment.amount);
        }

        let args = AddDelegationArgs {
            total_delegated_mapper: self.total_delegated_amount(validator_id),
            total_by_user_mapper: self.total_by_user(user_id_of_validator, validator_id),
            all_delegators_mapper: &mut self.all_delegators(validator_id),
            delegated_by_mapper: self.delegated_by(user_id_of_validator, validator_id),
            opt_max_delegation: validator_config.opt_max_delegation,
            payments_to_add: payments.clone(),
            total_amount: total,
            caller_id: user_id_of_validator,
        };
        self.add_delegation(args);

        self.emit_validator_add_own_delegation_event(validator, payments);
    }

    #[view(getValidatorConfig)]
    fn get_validator_config(&self, address: ManagedAddress) -> ValidatorConfig<Self::Api> {
        let validator_id = self.validator_id().get_id_non_zero(&address);

        self.validator_config(validator_id).get()
    }

    #[view(getTotalDelegatedAmount)]
    fn get_total_delegated_amount(&self, address: ManagedAddress) -> BigUint {
        let validator_id = self.validator_id().get_id_non_zero(&address);

        self.total_delegated_amount(validator_id).get()
    }

    #[storage_mapper("validatorId")]
    fn validator_id(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("validatorConfig")]
    fn validator_config(
        &self,
        validator_id: AddressId,
    ) -> SingleValueMapper<ValidatorConfig<Self::Api>>;

    #[storage_mapper("idForName")]
    fn id_for_name(&self, name: &ManagedBuffer) -> SingleValueMapper<AddressId>;

    #[storage_mapper("allDelegators")]
    fn all_delegators(&self, validator_id: AddressId) -> UnorderedSetMapper<AddressId>;

    #[storage_mapper("delegatedBy")]
    fn delegated_by(
        &self,
        user_id: AddressId,
        validator_id: AddressId,
    ) -> SingleValueMapper<UniquePayments<Self::Api>>;

    #[storage_mapper("totalDelegatedAmount")]
    fn total_delegated_amount(&self, validator_id: AddressId) -> SingleValueMapper<BigUint>;

    #[storage_mapper("totalByUser")]
    fn total_by_user(
        &self,
        user_id: AddressId,
        validator_id: AddressId,
    ) -> SingleValueMapper<BigUint>;
}
