use crate::{delegation_proxy, StorageCache, ERROR_INSUFFICIENT_GAS};

multiversx_sc::imports!();

pub const MIN_GAS_FOR_CALLBACK: u64 = 12_000_000;
pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;
pub const MIN_EGLD_TO_DELEGATE: u64 = 1_000_000_000_000_000_000;

#[multiversx_sc::module]
pub trait CommonModule:
    crate::config::ConfigModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    // views
    #[view(getLsValueForPosition)]
    fn get_ls_value_for_position(&self, ls_token_amount: BigUint) -> BigUint {
        let storage_cache = StorageCache::new(self);
        self.get_egld_amount(&ls_token_amount, &storage_cache)
    }

    fn get_gas_for_async_call(&self) -> u64 {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK,
            ERROR_INSUFFICIENT_GAS
        );

        gas_left - MIN_GAS_FOR_CALLBACK
    }

    fn send_back_unbond_nft(&self, caller: &ManagedAddress, unstake_token_nonce: u64) {
        let unstake_token_id = self.unstake_token().get_token_id();
        self.send().direct_esdt(
            caller,
            &unstake_token_id,
            unstake_token_nonce,
            &BigUint::from(1u64),
        )
    }

    #[proxy]
    fn delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
