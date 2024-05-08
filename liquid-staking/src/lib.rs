#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const MAX_DELEGATION_ADDRESSES: usize = 50;

pub type Epoch = u64;
pub type Percent = u64;

pub mod config;
mod contexts;
pub mod delegation;
pub mod delegation_proxy;
pub mod errors;
mod events;
mod liquidity_pool;
pub mod user_actions;

use crate::{
    delegation::{ClaimStatus, ClaimStatusType},
    errors::*,
};

use contexts::base::*;
use liquidity_pool::State;

#[multiversx_sc::contract]
pub trait LiquidStaking<ContractReader>:
    liquidity_pool::LiquidityPoolModule
    + config::ConfigModule
    + events::EventsModule
    + delegation::DelegationModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + user_actions::common::CommonModule
    + user_actions::add_liquidity::AddLiquidityModule
    + user_actions::remove_liquidity::RemoveLiquidityModule
    + user_actions::unbond::UnbondModule
    + user_actions::claim_rewards::ClaimRewardsModule
    + user_actions::delegate_rewards::DelegateRewardsModule
    + user_actions::recompute_token_reserve::RecomputeTokenReserveModule
{
    #[init]
    fn init(&self) {
        self.state().set(State::Inactive);
        self.max_delegation_addresses()
            .set(MAX_DELEGATION_ADDRESSES);

        let current_epoch = self.blockchain().get_block_epoch();
        let claim_status = ClaimStatus {
            status: ClaimStatusType::Insufficient,
            last_claim_epoch: current_epoch,
            last_claim_block: 0u64,
            current_node: 0,
            starting_token_reserve: BigUint::zero(),
        };

        self.delegation_claim_status().set(claim_status);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
