use crate::unique_payments::{PaymentsVec, UniquePayments};

use super::{
    call_delegation::EGLD_TOKEN_ID,
    common_actions::{AddDelegationArgs, RemoveDelegationArgs},
};

multiversx_sc::imports!();

pub type PaymentsMultiValue<M> =
    MultiValueEncoded<M, MultiValue3<TokenIdentifier<M>, u64, BigUint<M>>>;

#[multiversx_sc::module]
pub trait UserModule:
    crate::token_whitelist::TokenWhitelistModule
    + super::validator::ValidatorModule
    + super::sovereign::SovereignModule
    + super::unbond::UnbondModule
    + super::common_actions::CommonActionsModule
    + super::common_storage::CommonStorageModule
    + crate::events::user_events::UserEventsModule
    + crate::events::validator_events::ValidatorEventsModule
    + crate::events::sov_events::SovEventsModule
    + utils::UtilsModule
{
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) {
        let payments = self.get_non_empty_payments();
        let caller = self.blockchain().get_caller();
        self.deposit_common(&caller, &payments);

        self.emit_user_deposit_event(caller, payments);
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
        let validator_config = self.validator_config(validator_id).get();
        let (output_payments, total) =
            self.before_add_delegation(self.user_tokens(caller_id), tokens);

        let args = AddDelegationArgs {
            total_delegated_mapper: self.total_delegated_amount(validator_id),
            total_by_user_mapper: self.total_by_user(caller_id, validator_id),
            all_delegators_mapper: &mut self.all_delegators(validator_id),
            delegated_by_mapper: self.delegated_by(caller_id, validator_id),
            opt_max_delegation: validator_config.opt_max_delegation,
            payments_to_add: output_payments.clone(),
            total_amount: total,
            caller_id,
        };
        self.add_delegation(args);

        self.emit_delegate_validator_event(caller, validator, output_payments);
    }

    #[endpoint(delegateForSovereignChain)]
    fn delegate_for_sovereign_chain(
        &self,
        sov_name: ManagedBuffer,
        tokens: PaymentsMultiValue<Self::Api>,
    ) {
        self.require_non_empty_args(&tokens);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let sov_id = self.sov_chain_for_name(&sov_name).get();
        self.require_valid_sov_id(sov_id);

        let sov_info = self.sov_info(sov_id).get();
        let (output_payments, total) =
            self.before_add_delegation(self.user_tokens(caller_id), tokens);

        let args = AddDelegationArgs {
            total_delegated_mapper: self.total_delegated_sov_amount(sov_id),
            total_by_user_mapper: self.total_sov_by_user(caller_id, sov_id),
            all_delegators_mapper: &mut self.all_sov_delegators(sov_id),
            delegated_by_mapper: self.delegated_sov_by(caller_id, sov_id),
            opt_max_delegation: sov_info.opt_max_restaking_cap,
            payments_to_add: output_payments.clone(),
            total_amount: total,
            caller_id,
        };
        self.add_delegation(args);

        let sov_address = unsafe { self.sov_id().get_address(sov_id).unwrap_unchecked() };
        self.emit_delgate_sov_event(caller, sov_address, output_payments);
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

        let args = RemoveDelegationArgs {
            total_delegated_mapper: self.total_delegated_amount(validator_id),
            total_by_user_mapper: self.total_by_user(caller_id, validator_id),
            all_delegators_mapper: &mut self.all_delegators(validator_id),
            delegated_by_mapper: self.delegated_by(caller_id, validator_id),
            tokens,
            caller_id,
        };
        let output_unique_payments = self.remove_delegation(args);
        self.add_unbond_tokens(caller_id, output_unique_payments.clone());

        self.emit_revoke_validator_event(caller, validator, output_unique_payments);
    }

    #[endpoint(revokeDelegationFromSovereignChain)]
    fn revoke_delegation_from_sovereign_chain(
        &self,
        sov_name: ManagedBuffer,
        tokens: PaymentsMultiValue<Self::Api>,
    ) {
        self.require_non_empty_args(&tokens);

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let sov_id = self.sov_chain_for_name(&sov_name).get();
        self.require_valid_sov_id(sov_id);

        let args = RemoveDelegationArgs {
            total_delegated_mapper: self.total_delegated_sov_amount(sov_id),
            total_by_user_mapper: self.total_sov_by_user(caller_id, sov_id),
            all_delegators_mapper: &mut self.all_sov_delegators(sov_id),
            delegated_by_mapper: self.delegated_sov_by(caller_id, sov_id),
            tokens,
            caller_id,
        };
        let output_unique_payments = self.remove_delegation(args);
        self.add_unbond_tokens(caller_id, output_unique_payments.clone());

        let sov_address = unsafe { self.sov_id().get_address(sov_id).unwrap_unchecked() };
        self.emit_revoke_sov_event(caller, sov_address, output_unique_payments);
    }

    #[endpoint(unbondTokensCaller)]
    fn unbond_tokens_caller(&self) {
        let caller = self.blockchain().get_caller();
        let output_payments = self.unbond_common(&caller);
        if !output_payments.is_empty() {
            self.send().direct_multi(&caller, &output_payments);

            self.emit_unbond_tokens_caller_event(caller, output_payments);
        }
    }

    #[endpoint(unbondTokensGravityRestaking)]
    fn unbond_tokens_gravity_restaking(&self) {
        let caller = self.blockchain().get_caller();
        let output_payments = self.unbond_common(&caller);
        if !output_payments.is_empty() {
            self.deposit_common(&caller, &output_payments);

            self.emit_unbond_tokens_gravity_restaking_event(caller, output_payments);
        }
    }

    fn unbond_common(&self, caller: &ManagedAddress) -> PaymentsVec<Self::Api> {
        let caller_id = self.user_ids().get_id_non_zero(caller);
        let output_unique_payments = self.unbond_tokens_common(caller_id);

        output_unique_payments.into_payments()
    }

    fn deposit_common(&self, caller: &ManagedAddress, payments: &PaymentsVec<Self::Api>) {
        let ids_mapper = self.user_ids();
        let mut caller_id = ids_mapper.get_id(caller);
        let mut user_tokens = if caller_id == NULL_ID {
            caller_id = ids_mapper.insert_new(caller);

            UniquePayments::new()
        } else {
            self.user_tokens(caller_id).get()
        };

        for payment in payments {
            self.require_token_in_whitelist(&payment.token_identifier);

            user_tokens.add_payment(payment);
        }

        self.user_tokens(caller_id).set(user_tokens);
    }

    fn require_non_empty_args(&self, args: &PaymentsMultiValue<Self::Api>) {
        require!(!args.is_empty(), "No arguments");
    }
}
