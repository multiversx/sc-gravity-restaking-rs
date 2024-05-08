use crate::{
    config::UnstakeTokenAttributes, delegation_proxy::ProxyTrait as _, StorageCache,
    ERROR_BAD_PAYMENT_AMOUNT, ERROR_BAD_PAYMENT_TOKEN, ERROR_NOT_ACTIVE,
    ERROR_UNSTAKE_PERIOD_NOT_PASSED,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UnbondModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::delegation::DelegationModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + super::common::CommonModule
{
    #[payable("*")]
    #[endpoint(unbondTokens)]
    fn unbond_tokens(&self) {
        self.blockchain().check_caller_is_user_account();
        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        require!(
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );
        require!(
            payment.token_identifier == self.unstake_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0, ERROR_BAD_PAYMENT_AMOUNT);

        let unstake_token_attributes: UnstakeTokenAttributes<Self::Api> = self
            .unstake_token()
            .get_token_attributes(payment.token_nonce);

        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            current_epoch >= unstake_token_attributes.unbond_epoch,
            ERROR_UNSTAKE_PERIOD_NOT_PASSED
        );

        let delegation_contract = unstake_token_attributes.delegation_contract;
        let unstake_amount = unstake_token_attributes.unstake_amount;
        let delegation_contract_mapper = self.delegation_contract_data(&delegation_contract);
        let delegation_contract_data = delegation_contract_mapper.get();
        if delegation_contract_data.total_unbonded_from_ls_contract >= unstake_amount {
            delegation_contract_mapper.update(|contract_data| {
                contract_data.total_unstaked_from_ls_contract -= &unstake_amount;
                contract_data.total_unbonded_from_ls_contract -= &unstake_amount
            });

            storage_cache.total_withdrawn_egld -= &unstake_amount;
            self.unstake_token_supply()
                .update(|x| *x -= &unstake_amount);
            self.burn_unstake_tokens(payment.token_nonce);
            self.send().direct_egld(&caller, &unstake_amount);
        } else {
            let gas_for_async_call = self.get_gas_for_async_call();
            self.delegation_proxy_obj()
                .contract(delegation_contract.clone())
                .withdraw()
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(UnbondModule::callbacks(self).withdraw_tokens_callback(
                    caller,
                    delegation_contract,
                    payment.token_nonce,
                    unstake_amount,
                ))
                .call_and_exit();
        }
    }

    #[callback]
    fn withdraw_tokens_callback(
        &self,
        caller: ManagedAddress,
        delegation_contract: ManagedAddress,
        unstake_token_nonce: u64,
        unstake_token_amount: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let withdraw_amount = self.call_value().egld_value().clone_value();
                let mut storage_cache = StorageCache::new(self);
                let delegation_contract_mapper =
                    self.delegation_contract_data(&delegation_contract);
                if withdraw_amount > 0u64 {
                    delegation_contract_mapper.update(|contract_data| {
                        contract_data.total_unbonded_from_ls_contract += &withdraw_amount
                    });
                    storage_cache.total_withdrawn_egld += &withdraw_amount;
                }
                let delegation_contract_data = delegation_contract_mapper.get();
                if delegation_contract_data.total_unbonded_from_ls_contract >= unstake_token_amount
                {
                    delegation_contract_mapper.update(|contract_data| {
                        contract_data.total_unstaked_from_ls_contract -= &unstake_token_amount;
                        contract_data.total_unbonded_from_ls_contract -= &unstake_token_amount;
                    });
                    storage_cache.total_withdrawn_egld -= &unstake_token_amount;
                    self.unstake_token_supply()
                        .update(|x| *x -= &unstake_token_amount);
                    self.burn_unstake_tokens(unstake_token_nonce);
                    self.send().direct_egld(&caller, &unstake_token_amount);
                } else {
                    self.send_back_unbond_nft(&caller, unstake_token_nonce);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                self.send_back_unbond_nft(&caller, unstake_token_nonce);
            }
        }
    }
}
