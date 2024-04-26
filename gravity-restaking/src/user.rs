use multiversx_sc::api::StorageMapperApi;

use crate::{
    call_delegation::EGLD_TOKEN_ID,
    unique_payments::{PaymentsVec, UniquePayments},
    validator::ValidatorConfig,
};

multiversx_sc::imports!();

pub type PaymentsMultiValue<M> =
    MultiValueEncoded<M, MultiValue3<TokenIdentifier<M>, u64, BigUint<M>>>;

pub struct AddDelegationArgs<'a, M: StorageMapperApi> {
    pub total_amount_mapper: SingleValueMapper<M, BigUint<M>>,
    pub total_by_user_mapper: SingleValueMapper<M, BigUint<M>>,
    pub all_delegators_mapper: &'a mut UnorderedSetMapper<M, AddressId>,
    pub delegated_by_mapper: SingleValueMapper<M, UniquePayments<M>>,
    pub opt_validator_config_mapper: Option<SingleValueMapper<M, ValidatorConfig<M>>>,
    pub payments_to_add: PaymentsVec<M>,
    pub total_amount: BigUint<M>,
    pub caller_id: AddressId,
}

#[multiversx_sc::module]
pub trait UserModule:
    crate::token_whitelist::TokenWhitelistModule
    + crate::validator::ValidatorModule
    + utils::UtilsModule
{
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) {
        let payments = self.get_non_empty_payments();
        let caller = self.blockchain().get_caller();
        let ids_mapper = self.user_ids();
        let mut caller_id = ids_mapper.get_id(&caller);
        let mut user_tokens = if caller_id == NULL_ID {
            caller_id = ids_mapper.insert_new(&caller);

            UniquePayments::new()
        } else {
            self.user_tokens(caller_id).get()
        };

        for payment in &payments {
            self.require_token_in_whitelist(&payment.token_identifier);

            user_tokens.add_payment(payment);
        }

        self.user_tokens(caller_id).set(user_tokens);
    }

    /// Pairs of (token_id, nonce, amount)
    #[endpoint]
    fn withdraw(&self, tokens: PaymentsMultiValue<Self::Api>) {
        self.require_non_empty_args(&tokens);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);

        let egld_token_id = TokenIdentifier::from_esdt_bytes(EGLD_TOKEN_ID);
        let mut output_payments = PaymentsVec::new();
        let mut total_egld = BigUint::zero();
        self.user_tokens(caller_id).update(|user_tokens| {
            for token_tuple in tokens {
                let (token_id, nonce, amount) = token_tuple.into_tuple();
                require!(amount > 0, "Can't withdraw 0");

                let payment = EsdtTokenPayment::new(token_id, nonce, amount);
                let deduct_result = user_tokens.deduct_payment(&payment);
                require!(deduct_result.is_ok(), "Trying to withdraw too many tokens");

                if payment.token_identifier != egld_token_id {
                    output_payments.push(payment);
                } else {
                    total_egld += payment.amount;
                }
            }
        });

        self.send().direct_non_zero_egld(&caller, &total_egld);

        if !output_payments.is_empty() {
            self.send().direct_multi(&caller, &output_payments);
        }
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let user_tokens = self.user_tokens(caller_id).update(|user_tokens| {
            let output = (*user_tokens).clone();
            *user_tokens = UniquePayments::new();

            output
        });

        let mut output_payments = user_tokens.into_payments();
        require!(!output_payments.is_empty(), "Nothing to withdraw");

        let egld_token_id = TokenIdentifier::from_esdt_bytes(EGLD_TOKEN_ID);
        let mut opt_index_to_remove = None;
        for (i, payment) in output_payments.iter().enumerate() {
            if payment.token_identifier == egld_token_id {
                opt_index_to_remove = Some(i);

                break;
            }
        }

        if let Some(index_to_remove) = opt_index_to_remove {
            let egld_payment = output_payments.get(index_to_remove);
            output_payments.remove(index_to_remove);

            self.send().direct_egld(&caller, &egld_payment.amount);
        }

        if !output_payments.is_empty() {
            self.send().direct_multi(&caller, &output_payments);
        }
    }

    #[endpoint(delegateToValidator)]
    fn delegate_to_validator(
        &self,
        validator: ManagedAddress,
        tokens: PaymentsMultiValue<Self::Api>,
    ) {
        self.require_non_empty_args(&tokens);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let validator_id = self.validator_id().get_id_non_zero(&validator);

        let mut output_payments = PaymentsVec::new();
        let mut total = BigUint::zero();
        self.user_tokens(caller_id).update(|user_tokens| {
            for token_tuple in tokens {
                let (token_id, nonce, amount) = token_tuple.into_tuple();
                require!(amount > 0, "Can't delegate 0");

                // in case the token was removed from the whitelist in the meantime
                self.require_token_in_whitelist(&token_id);

                let payment = EsdtTokenPayment::new(token_id, nonce, amount);
                let deduct_result = user_tokens.deduct_payment(&payment);
                require!(deduct_result.is_ok(), "Trying to delegate too many tokens");

                total += self.get_total_staked_egld(&payment.token_identifier, &payment.amount);
                output_payments.push(payment);
            }
        });

        let args = AddDelegationArgs {
            total_amount_mapper: self.total_delegated_amount(validator_id),
            total_by_user_mapper: self.total_by_user(caller_id, validator_id),
            all_delegators_mapper: &mut self.all_delegators(validator_id),
            delegated_by_mapper: self.delegated_by(caller_id, validator_id),
            opt_validator_config_mapper: Some(self.validator_config(validator_id)),
            payments_to_add: output_payments,
            total_amount: total,
            caller_id,
        };
        self.add_delegation(args);
    }

    #[endpoint(revokeDelegationFromValidator)]
    fn revoke_delegation_from_validator(
        &self,
        validator: ManagedAddress,
        tokens: PaymentsMultiValue<Self::Api>,
    ) {
        self.require_non_empty_args(&tokens);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let validator_id = self.validator_id().get_id_non_zero(&validator);

        let delegated_by_mapper = self.delegated_by(caller_id, validator_id);
        require!(
            !delegated_by_mapper.is_empty(),
            "Nothing delegated to this validator"
        );

        let mut output_payments = PaymentsVec::new();
        let mut total = BigUint::zero();
        delegated_by_mapper.update(|delegated_tokens| {
            for token_tuple in tokens {
                let (token_id, nonce, amount) = token_tuple.into_tuple();
                require!(amount > 0, "Can't revoke 0");

                let payment = EsdtTokenPayment::new(token_id, nonce, amount);
                let deduct_result = delegated_tokens.deduct_payment(&payment);
                require!(deduct_result.is_ok(), "Trying to revoke too many tokens");

                total += self.get_total_staked_egld(&payment.token_identifier, &payment.amount);
                output_payments.push(payment);
            }
        });

        self.total_delegated_amount(validator_id)
            .update(|total_del| {
                *total_del -= &total;
            });
        self.total_by_user(caller_id, validator_id)
            .update(|total_user| {
                *total_user -= total;

                if *total_user == 0 {
                    let _ = self.all_delegators(validator_id).swap_remove(&caller_id);
                }
            });

        self.user_tokens(caller_id).update(|user_tokens| {
            for payment in &output_payments {
                user_tokens.add_payment(payment);
            }
        });
    }

    /// Used by validators
    #[payable("*")]
    #[endpoint(addOwnDelegation)]
    fn add_own_delegation(&self) {
        let validator = self.blockchain().get_caller();
        let validator_id = self.validator_id().get_id_non_zero(&validator);
        let user_id_of_validator = self.user_ids().get_id_or_insert(&validator);

        let payments = self.get_non_empty_payments();
        let mut total = BigUint::zero();
        for payment in &payments {
            self.require_token_in_whitelist(&payment.token_identifier);

            total += self.get_total_staked_egld(&payment.token_identifier, &payment.amount);
        }

        let args = AddDelegationArgs {
            total_amount_mapper: self.total_delegated_amount(validator_id),
            total_by_user_mapper: self.total_by_user(user_id_of_validator, validator_id),
            all_delegators_mapper: &mut self.all_delegators(validator_id),
            delegated_by_mapper: self.delegated_by(user_id_of_validator, validator_id),
            opt_validator_config_mapper: Some(self.validator_config(validator_id)),
            payments_to_add: payments,
            total_amount: total,
            caller_id: user_id_of_validator,
        };
        self.add_delegation(args);
    }

    fn add_delegation(&self, args: AddDelegationArgs<Self::Api>) {
        args.total_amount_mapper.update(|total_del| {
            *total_del += &args.total_amount;

            if let Some(config_mapper) = args.opt_validator_config_mapper {
                let config = config_mapper.get();
                if let Some(max_amt) = config.opt_max_delegation {
                    require!(*total_del <= max_amt, "Max delegated amount exceeded");
                }
            }
        });

        args.total_by_user_mapper
            .update(|total_user| *total_user += args.total_amount);

        let _ = args.all_delegators_mapper.insert(args.caller_id);

        let mut tokens_delegated_by_user = if !args.delegated_by_mapper.is_empty() {
            args.delegated_by_mapper.get()
        } else {
            UniquePayments::new()
        };
        for payment in &args.payments_to_add {
            tokens_delegated_by_user.add_payment(payment);
        }
        args.delegated_by_mapper.set(tokens_delegated_by_user);
    }

    fn require_non_empty_args(&self, args: &PaymentsMultiValue<Self::Api>) {
        require!(!args.is_empty(), "No arguments");
    }

    #[storage_mapper("userIds")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[view(getUserTokens)]
    #[storage_mapper("userTokens")]
    fn user_tokens(&self, user_id: AddressId) -> SingleValueMapper<UniquePayments<Self::Api>>;
}
