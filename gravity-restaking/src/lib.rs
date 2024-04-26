#![no_std]

multiversx_sc::imports!();

pub mod call_delegation;
pub mod common_actions;
pub mod sovereign;
pub mod token_whitelist;
pub mod unique_payments;
pub mod user;
pub mod validator;

#[multiversx_sc::contract]
pub trait GravityRestaking:
    call_delegation::CallDelegationModule
    + token_whitelist::TokenWhitelistModule
    + user::UserModule
    + validator::ValidatorModule
    + sovereign::SovereignModule
    + common_actions::CommonActionsModule
    + utils::UtilsModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
