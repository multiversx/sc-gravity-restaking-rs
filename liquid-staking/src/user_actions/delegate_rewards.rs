use crate::{
    delegation::ClaimStatusType, delegation_proxy::ProxyTrait as _, StorageCache,
    ERROR_CLAIM_REDELEGATE, ERROR_NOT_ACTIVE,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DelegateRewardsModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::delegation::DelegationModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + super::common::CommonModule
{
    #[endpoint(delegateRewards)]
    fn delegate_rewards(&self) {
        let mut storage_cache = StorageCache::new(self);
        let claim_status = self.delegation_claim_status().get();
        require!(
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );
        require!(
            claim_status.status == ClaimStatusType::Delegable,
            ERROR_CLAIM_REDELEGATE
        );

        let rewards_reserve = storage_cache.rewards_reserve.clone();
        storage_cache.rewards_reserve = BigUint::zero();
        let delegation_contract = self.get_delegation_contract_for_delegate(&rewards_reserve);
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract.clone())
            .delegate()
            .with_gas_limit(gas_for_async_call)
            .with_egld_transfer(rewards_reserve.clone())
            .async_call()
            .with_callback(
                DelegateRewardsModule::callbacks(self)
                    .delegate_rewards_callback(delegation_contract, rewards_reserve),
            )
            .call_and_exit()
    }

    #[callback]
    fn delegate_rewards_callback(
        &self,
        delegation_contract: ManagedAddress,
        staked_tokens: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut storage_cache = StorageCache::new(self);
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.total_staked_from_ls_contract += &staked_tokens;
                    });

                self.delegation_claim_status()
                    .update(|claim_status| claim_status.status = ClaimStatusType::Redelegated);

                storage_cache.virtual_egld_reserve += &staked_tokens;
                let sc_address = self.blockchain().get_sc_address();
                self.emit_add_liquidity_event(&storage_cache, &sc_address, BigUint::zero());
            }
            ManagedAsyncCallResult::Err(_) => {
                storage_cache.rewards_reserve = staked_tokens;
                self.move_delegation_contract_to_back(delegation_contract);
            }
        }
    }
}
