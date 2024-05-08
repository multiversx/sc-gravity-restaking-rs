use crate::{
    delegation_proxy::ProxyTrait as _, user_actions::common::MIN_EGLD_TO_DELEGATE, StorageCache,
    ERROR_BAD_PAYMENT_AMOUNT, ERROR_NOT_ACTIVE,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AddLiquidityModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::delegation::DelegationModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + super::common::CommonModule
{
    #[payable("EGLD")]
    #[endpoint(addLiquidity)]
    fn add_liquidity(&self) {
        self.blockchain().check_caller_is_user_account();

        let storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();

        let payment = self.call_value().egld_value().clone_value();
        require!(
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );
        require!(payment >= MIN_EGLD_TO_DELEGATE, ERROR_BAD_PAYMENT_AMOUNT);

        let delegation_contract = self.get_delegation_contract_for_delegate(&payment);
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract.clone())
            .delegate()
            .with_gas_limit(gas_for_async_call)
            .with_egld_transfer(payment.clone())
            .async_call()
            .with_callback(AddLiquidityModule::callbacks(self).add_liquidity_callback(
                caller,
                delegation_contract,
                payment,
            ))
            .call_and_exit()
    }

    #[callback]
    fn add_liquidity_callback(
        &self,
        caller: ManagedAddress,
        delegation_contract: ManagedAddress,
        staked_tokens: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let mut storage_cache = StorageCache::new(self);
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.total_staked_from_ls_contract += &staked_tokens;
                    });

                let ls_token_amount = self.pool_add_liquidity(&staked_tokens, &mut storage_cache);
                let user_payment = self.mint_ls_token(ls_token_amount);
                self.send().direct_esdt(
                    &caller,
                    &user_payment.token_identifier,
                    user_payment.token_nonce,
                    &user_payment.amount,
                );

                self.emit_add_liquidity_event(&storage_cache, &caller, user_payment.amount);
            }
            ManagedAsyncCallResult::Err(_) => {
                self.send().direct_egld(&caller, &staked_tokens);
                self.move_delegation_contract_to_back(delegation_contract);
            }
        }
    }
}
