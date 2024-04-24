// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           19
// Async Callback:                       1
// Total number of exported functions:  21

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    gravity_restaking
    (
        init => init
        upgrade => upgrade
        moveStakeToReStaking => move_stake_to_re_staking
        addTokenToWhitelist => add_token_to_whitelist
        removeTokenFromWhitelist => remove_token_from_whitelist
        getTokenWhitelist => token_whitelist
        getStakedEgldForOneToken => staked_egld_for_one_token
        deposit => deposit
        withdraw => withdraw
        withdrawAll => withdraw_all
        delegateToValidator => delegate_to_validator
        revokeDelegationFromValidator => revoke_delegation_from_validator
        getUserTokens => user_tokens
        register => register
        addKeys => add_keys
        removeKeys => remove_keys
        setUpFee => set_up_fee
        setMaxDelegation => set_max_delegation
        getValidatorConfig => get_validator_config
        getTotalDelegatedAmount => get_total_delegated_amount
    )
}

multiversx_sc_wasm_adapter::async_callback! { gravity_restaking }
