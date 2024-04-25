use crate::unique_payments::UniquePayments;

multiversx_sc::imports!();

mod delegation_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait DelegationProxy {
        #[endpoint(moveStakeToReStaking)]
        fn move_stake_to_re_staking(&self, user: ManagedAddress, value: BigUint);
    }
}

// Max i32 seems to be the same value in both Rust and Go
// https://pkg.go.dev/math#pkg-constants
// https://doc.rust-lang.org/std/i32/constant.MAX.html
const MAX_SHARD_ID: u32 = i32::MAX as u32;

pub static EGLD_TOKEN_ID: &[u8] = b"EGLD";

#[multiversx_sc::module]
pub trait CallDelegationModule:
    crate::user::UserModule
    + crate::token_whitelist::TokenWhitelistModule
    + crate::validator::ValidatorModule
    + utils::UtilsModule
{
    #[endpoint(moveStakeToReStaking)]
    fn move_stake_to_re_staking(&self, delegation: ManagedAddress, value: BigUint) {
        let caller = self.blockchain().get_caller();
        let delegation_shard = self.blockchain().get_shard_of_address(&delegation);
        require!(
            delegation_shard == MAX_SHARD_ID,
            "Invalid delegation address"
        );

        self.call_restake_async(delegation, caller, value);
    }

    fn call_restake_async(&self, delegation: ManagedAddress, user: ManagedAddress, value: BigUint) {
        self.delegation_proxy_obj(delegation)
            .move_stake_to_re_staking(user.clone(), value.clone())
            .async_call_promise()
            .with_callback(
                <Self as CallDelegationModule>::callbacks(self).move_stake_callback(user, value),
            )
            .register_promise();
    }

    #[callback]
    fn move_stake_callback(
        &self,
        original_caller: ManagedAddress,
        original_value: BigUint,
        #[call_result] call_result: ManagedAsyncCallResult<()>,
    ) {
        if let ManagedAsyncCallResult::Ok(()) = call_result {
            let ids_mapper = self.user_ids();
            let mut caller_id = ids_mapper.get_id(&original_caller);
            let mut user_tokens = if caller_id == NULL_ID {
                caller_id = ids_mapper.insert_new(&original_caller);

                UniquePayments::new()
            } else {
                self.user_tokens(caller_id).get()
            };

            let egld_payment = EsdtTokenPayment::new(
                TokenIdentifier::from_esdt_bytes(EGLD_TOKEN_ID),
                0,
                original_value,
            );
            user_tokens.add_payment(egld_payment);

            self.user_tokens(caller_id).set(user_tokens);
        }
    }

    #[proxy]
    fn delegation_proxy_obj(
        &self,
        sc_address: ManagedAddress,
    ) -> delegation_proxy::Proxy<Self::Api>;
}
