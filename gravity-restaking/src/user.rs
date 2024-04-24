use crate::unique_payments::{PaymentsVec, UniquePayments};

multiversx_sc::imports!();

pub type PaymentsMultiValue<M> =
    MultiValueEncoded<M, MultiValue3<TokenIdentifier<M>, u64, BigUint<M>>>;

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

        let whitelist_mapper = self.token_whitelist();
        for payment in &payments {
            require!(
                whitelist_mapper.contains(&payment.token_identifier),
                "Invalid token"
            );

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

        let mut output_payments = PaymentsVec::new();
        self.user_tokens(caller_id).update(|user_tokens| {
            for token_tuple in tokens {
                let (token_id, nonce, amount) = token_tuple.into_tuple();
                require!(amount > 0, "Can't withdraw 0");

                let payment = EsdtTokenPayment::new(token_id, nonce, amount);
                let deduct_result = user_tokens.deduct_payment(&payment);
                require!(deduct_result.is_ok(), "Trying to withdraw too many tokens");

                output_payments.push(payment);
            }
        });

        self.send().direct_multi(&caller, &output_payments);
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

        let output_payments = user_tokens.into_payments();
        require!(!output_payments.is_empty(), "Nothing to withdraw");

        self.send().direct_multi(&caller, &output_payments);
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

                let payment = EsdtTokenPayment::new(token_id, nonce, amount);
                let deduct_result = user_tokens.deduct_payment(&payment);
                require!(deduct_result.is_ok(), "Trying to delegate too many tokens");

                total += &payment.amount;
                output_payments.push(payment);
            }
        });

        self.total_delegated_amount(validator_id)
            .update(|total_del| {
                *total_del += &total;

                let config = self.validator_config(validator_id).get();
                if let Some(max_amt) = config.opt_max_delegation {
                    require!(*total_del <= max_amt, "Max delegated amount exceeded");
                }
            });
        self.total_by_user(caller_id, validator_id)
            .update(|total_user| *total_user += total);

        let _ = self.all_delegators(validator_id).insert(caller_id);

        let delegated_by_mapper = self.delegated_by(caller_id, validator_id);
        let mut tokens_delegated_by_user = if !delegated_by_mapper.is_empty() {
            delegated_by_mapper.get()
        } else {
            UniquePayments::new()
        };
        for payment in &output_payments {
            tokens_delegated_by_user.add_payment(payment);
        }
        delegated_by_mapper.set(tokens_delegated_by_user);
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

                total += &payment.amount;
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

    fn require_non_empty_args(&self, args: &PaymentsMultiValue<Self::Api>) {
        require!(!args.is_empty(), "No arguments");
    }

    #[storage_mapper("userIds")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[view(getUserTokens)]
    #[storage_mapper("userTokens")]
    fn user_tokens(&self, user_id: AddressId) -> SingleValueMapper<UniquePayments<Self::Api>>;
}
