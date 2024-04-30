use crate::{unique_payments::PaymentsVec, user_actions::sovereign::SovereignInfo};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SovEventsModule {
    #[inline]
    fn emit_sov_register_event(
        &self,
        sov_address: ManagedAddress,
        sov_info: SovereignInfo<Self::Api>,
    ) {
        self.sov_register_event(sov_address, sov_info);
    }

    #[inline]
    fn emit_sov_unregister_event(&self, sov_address: ManagedAddress) {
        self.sov_unregister_event(sov_address);
    }

    #[inline]
    fn emit_sov_add_own_security_funds_event(
        &self,
        sov_address: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) {
        self.sov_add_own_security_funds_event(sov_address, payments);
    }

    #[inline]
    fn emit_sov_set_max_restaking_cap_event(&self, sov_address: ManagedAddress, max: BigUint) {
        self.sov_set_max_restaking_cap_event(sov_address, max);
    }

    // Events

    #[event("sovRegisterEvent")]
    fn sov_register_event(
        &self,
        #[indexed] sov_address: ManagedAddress,
        sov_info: SovereignInfo<Self::Api>,
    );

    #[event("sovUnregisterEvent")]
    fn sov_unregister_event(&self, #[indexed] sov_address: ManagedAddress);

    #[event("sovAddOwnSecurityFundsEvent")]
    fn sov_add_own_security_funds_event(
        &self,
        #[indexed] sov_address: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    );

    #[event("sovSetMaxRestakingCapEvent")]
    fn sov_set_max_restaking_cap_event(&self, #[indexed] sov_address: ManagedAddress, max: BigUint);
}
