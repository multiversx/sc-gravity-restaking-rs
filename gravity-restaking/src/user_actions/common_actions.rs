use multiversx_sc::api::StorageMapperApi;

use crate::unique_payments::{PaymentsVec, UniquePayments};

use super::{user::PaymentsMultiValue, validator::ValidatorConfig};

multiversx_sc::imports!();

pub struct AddDelegationArgs<'a, S: StorageMapperApi> {
    pub total_delegated_mapper: SingleValueMapper<S, BigUint<S>>,
    pub total_by_user_mapper: SingleValueMapper<S, BigUint<S>>,
    pub all_delegators_mapper: &'a mut UnorderedSetMapper<S, AddressId>,
    pub delegated_by_mapper: SingleValueMapper<S, UniquePayments<S>>,
    pub opt_validator_config_mapper: Option<SingleValueMapper<S, ValidatorConfig<S>>>,
    pub payments_to_add: PaymentsVec<S>,
    pub total_amount: BigUint<S>,
    pub caller_id: AddressId,
}

pub struct RemoveDelegationArgs<'a, S: StorageMapperApi> {
    pub total_delegated_mapper: SingleValueMapper<S, BigUint<S>>,
    pub total_by_user_mapper: SingleValueMapper<S, BigUint<S>>,
    pub all_delegators_mapper: &'a mut UnorderedSetMapper<S, AddressId>,
    pub delegated_by_mapper: SingleValueMapper<S, UniquePayments<S>>,
    pub tokens: PaymentsMultiValue<S>,
    pub caller_id: AddressId,
}

#[multiversx_sc::module]
pub trait CommonActionsModule: crate::token_whitelist::TokenWhitelistModule {
    fn before_add_delegation(
        &self,
        user_tokens_mapper: SingleValueMapper<UniquePayments<Self::Api>>,
        tokens: PaymentsMultiValue<Self::Api>,
    ) -> (PaymentsVec<Self::Api>, BigUint<Self::Api>) {
        let mut output_payments = PaymentsVec::new();
        let mut total = BigUint::zero();
        user_tokens_mapper.update(|user_tokens| {
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

        (output_payments, total)
    }

    fn add_delegation(&self, args: AddDelegationArgs<Self::Api>) {
        args.total_delegated_mapper.update(|total_del| {
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

    fn remove_delegation(
        &self,
        args: RemoveDelegationArgs<Self::Api>,
    ) -> UniquePayments<Self::Api> {
        require!(!args.delegated_by_mapper.is_empty(), "Nothing delegated");

        let mut output_payments = PaymentsVec::new();
        let mut total = BigUint::zero();
        args.delegated_by_mapper.update(|delegated_tokens| {
            for token_tuple in args.tokens {
                let (token_id, nonce, amount) = token_tuple.into_tuple();
                require!(amount > 0, "Can't revoke 0");

                let payment = EsdtTokenPayment::new(token_id, nonce, amount);
                let deduct_result = delegated_tokens.deduct_payment(&payment);
                require!(deduct_result.is_ok(), "Trying to revoke too many tokens");

                total += self.get_total_staked_egld(&payment.token_identifier, &payment.amount);
                output_payments.push(payment);
            }
        });

        args.total_delegated_mapper.update(|total_del| {
            *total_del -= &total;
        });
        args.total_by_user_mapper.update(|total_user| {
            *total_user -= total;

            if *total_user == 0 {
                let _ = args.all_delegators_mapper.swap_remove(&args.caller_id);
            }
        });

        UniquePayments::new_from_payments(output_payments)
    }
}
