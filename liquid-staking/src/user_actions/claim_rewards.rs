use multiversx_sc_modules::ongoing_operation::{
    CONTINUE_OP, DEFAULT_MIN_GAS_TO_SAVE_PROGRESS, STOP_OP,
};

use crate::{
    delegation::{ClaimStatus, ClaimStatusType},
    delegation_proxy::ProxyTrait as _,
    StorageCache, ERROR_NOT_ACTIVE, ERROR_NO_DELEGATION_CONTRACTS,
};

multiversx_sc::imports!();

pub const DEFAULT_GAS_TO_CLAIM_REWARDS: u64 = 6_000_000;

#[multiversx_sc::module]
pub trait ClaimRewardsModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::delegation::DelegationModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + super::common::CommonModule
{
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let storage_cache = StorageCache::new(self);

        require!(
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );

        let delegation_addresses_mapper = self.delegation_addresses_list();
        require!(
            !delegation_addresses_mapper.is_empty(),
            ERROR_NO_DELEGATION_CONTRACTS
        );
        let claim_status_mapper = self.delegation_claim_status();
        let old_claim_status = claim_status_mapper.get();
        let current_epoch = self.blockchain().get_block_epoch();
        let mut current_claim_status = self.load_operation::<ClaimStatus<Self::Api>>();

        self.check_claim_operation(&current_claim_status, old_claim_status, current_epoch);
        self.prepare_claim_operation(&mut current_claim_status, current_epoch);

        let run_result = self.run_while_it_has_gas(DEFAULT_MIN_GAS_TO_SAVE_PROGRESS, || {
            let delegation_address_node = delegation_addresses_mapper
                .get_node_by_id(current_claim_status.current_node)
                .unwrap();
            let next_node = delegation_address_node.get_next_node_id();
            let delegation_address = delegation_address_node.into_value();

            self.delegation_proxy_obj()
                .contract(delegation_address)
                .claim_rewards()
                .with_gas_limit(DEFAULT_GAS_TO_CLAIM_REWARDS)
                .transfer_execute();

            if next_node == 0 {
                claim_status_mapper.set(current_claim_status.clone());
                return STOP_OP;
            } else {
                current_claim_status.current_node = next_node;
            }

            CONTINUE_OP
        });

        match run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_progress(&current_claim_status);
            }
            OperationCompletionStatus::Completed => {
                claim_status_mapper.update(|claim_status| {
                    claim_status.status = ClaimStatusType::Finished;
                    claim_status.last_claim_block = self.blockchain().get_block_nonce();
                });
            }
        };
    }
}
