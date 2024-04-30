use crate::unique_payments::{PaymentsVec, UniquePayments};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UserEventsModule {
    #[inline]
    fn emit_user_deposit_event(&self, caller: ManagedAddress, payments: PaymentsVec<Self::Api>) {
        self.user_deposit_event(caller, payments);
    }

    #[inline]
    fn emit_move_stake_event(
        &self,
        caller: ManagedAddress,
        delegation: ManagedAddress,
        value: BigUint,
    ) {
        self.move_stake_event(caller, delegation, value);
    }

    #[inline]
    fn emit_delegate_validator_event(
        &self,
        caller: ManagedAddress,
        validator: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) {
        self.delegate_validator_event(caller, validator, payments);
    }

    #[inline]
    fn emit_delgate_sov_event(
        &self,
        caller: ManagedAddress,
        sov: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) {
        self.delegate_sov_event(caller, sov, payments);
    }

    #[inline]
    fn emit_revoke_validator_event(
        &self,
        caller: ManagedAddress,
        validator: ManagedAddress,
        payments: UniquePayments<Self::Api>,
    ) {
        self.revoke_validator_event(caller, validator, payments);
    }

    #[inline]
    fn emit_revoke_sov_event(
        &self,
        caller: ManagedAddress,
        sov: ManagedAddress,
        payments: UniquePayments<Self::Api>,
    ) {
        self.revoke_sov_event(caller, sov, payments);
    }

    #[inline]
    fn emit_unbond_tokens_caller_event(
        &self,
        caller: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) {
        self.unbond_tokens_caller_event(caller, payments);
    }

    #[inline]
    fn emit_unbond_tokens_gravity_restaking_event(
        &self,
        caller: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) {
        self.unbond_tokens_gravity_restaking_event(caller, payments);
    }

    // Events

    #[event("userDepositEvent")]
    fn user_deposit_event(
        &self,
        #[indexed] caller: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    );

    #[event("moveStakeEvent")]
    fn move_stake_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] delegation: ManagedAddress,
        value: BigUint,
    );

    #[event("delegateValidatorEvent")]
    fn delegate_validator_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] validator: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    );

    #[event("delegateSovEvent")]
    fn delegate_sov_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] sov: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    );

    #[event("revokeValidatorEvent")]
    fn revoke_validator_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] validator: ManagedAddress,
        payments: UniquePayments<Self::Api>,
    );

    #[event("revokeSovEvent")]
    fn revoke_sov_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] sov: ManagedAddress,
        payments: UniquePayments<Self::Api>,
    );

    #[event("unbondTokensCallerEvent")]
    fn unbond_tokens_caller_event(
        &self,
        #[indexed] caller: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    );

    #[event("unbondTokensGravityRestakingEvent")]
    fn unbond_tokens_gravity_restaking_event(
        &self,
        #[indexed] caller: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    );
}
