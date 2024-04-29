#![no_std]

multiversx_sc::imports!();

pub mod token_whitelist;
pub mod unique_payments;
pub mod user_actions;

#[multiversx_sc::contract]
pub trait GravityRestaking:
    user_actions::call_delegation::CallDelegationModule
    + token_whitelist::TokenWhitelistModule
    + user_actions::user::UserModule
    + user_actions::validator::ValidatorModule
    + user_actions::sovereign::SovereignModule
    + user_actions::common_actions::CommonActionsModule
    + utils::UtilsModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
