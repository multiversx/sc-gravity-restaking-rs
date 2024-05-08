use crate::{
    delegation::ClaimStatusType, user_actions::common::MIN_EGLD_TO_DELEGATE, StorageCache,
    ERROR_NOT_ACTIVE, ERROR_RECOMPUTE_RESERVES, ERROR_RECOMPUTE_TOO_SOON,
};

multiversx_sc::imports!();

pub const RECOMPUTE_BLOCK_OFFSET: u64 = 10;

#[multiversx_sc::module]
pub trait RecomputeTokenReserveModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::delegation::DelegationModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + super::common::CommonModule
{
    #[endpoint(recomputeTokenReserve)]
    fn recompute_token_reserve(&self) {
        let mut storage_cache = StorageCache::new(self);
        let claim_status_mapper = self.delegation_claim_status();
        let mut claim_status = claim_status_mapper.get();

        require!(
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );
        require!(
            claim_status.status == ClaimStatusType::Finished,
            ERROR_RECOMPUTE_RESERVES
        );

        let current_block = self.blockchain().get_block_nonce();
        require!(
            current_block >= claim_status.last_claim_block + RECOMPUTE_BLOCK_OFFSET,
            ERROR_RECOMPUTE_TOO_SOON
        );

        let current_egld_balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        if current_egld_balance
            > &storage_cache.total_withdrawn_egld + &claim_status.starting_token_reserve
        {
            let rewards = &current_egld_balance
                - &storage_cache.total_withdrawn_egld
                - &claim_status.starting_token_reserve;
            storage_cache.rewards_reserve += rewards;
        }

        if storage_cache.rewards_reserve >= MIN_EGLD_TO_DELEGATE {
            claim_status.status = ClaimStatusType::Delegable;
        } else {
            claim_status.status = ClaimStatusType::Insufficient;
        }

        claim_status_mapper.set(claim_status);
    }
}
