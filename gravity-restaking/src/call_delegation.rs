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

#[multiversx_sc::module]
pub trait CallDelegationModule {
    // TODO: Integrate EGLD
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
            .move_stake_to_re_staking(user, value)
            .async_call_promise()
            .with_callback(<Self as CallDelegationModule>::callbacks(self).move_stake_callback())
            .register_promise();
    }

    #[callback]
    fn move_stake_callback(&self, #[call_result] _call_result: ManagedAsyncCallResult<()>) {
        // TODO
    }

    #[proxy]
    fn delegation_proxy_obj(
        &self,
        sc_address: ManagedAddress,
    ) -> delegation_proxy::Proxy<Self::Api>;
}
