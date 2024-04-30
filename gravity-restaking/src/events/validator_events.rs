use crate::{
    unique_payments::PaymentsVec,
    user_actions::validator::{BlsKey, Percent},
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ValidatorEventsModule {
    #[inline]
    fn emit_validator_register_event(&self, validator: ManagedAddress, name: ManagedBuffer) {
        self.validator_register_event(validator, name);
    }

    #[inline]
    fn emit_validator_add_bls_keys_event(
        &self,
        validator: ManagedAddress,
        bls_keys: ManagedVec<BlsKey<Self::Api>>,
    ) {
        self.validator_add_bls_keys_event(validator, bls_keys);
    }

    #[inline]
    fn emit_validator_remove_bls_keys_event(
        &self,
        validator: ManagedAddress,
        bls_keys: ManagedVec<BlsKey<Self::Api>>,
    ) {
        self.validator_remove_bls_keys_event(validator, bls_keys);
    }

    #[inline]
    fn emit_validator_set_fee_event(&self, validator: ManagedAddress, fee_percent: Percent) {
        self.validator_set_fee_event(validator, fee_percent);
    }

    #[inline]
    fn emit_validator_set_max_delegation_event(
        &self,
        validator: ManagedAddress,
        max_delegation: BigUint,
    ) {
        self.validator_set_max_delegation_event(validator, max_delegation);
    }

    #[inline]
    fn emit_validator_add_own_delegation_event(
        &self,
        validator: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) {
        self.validator_add_own_delegation_event(validator, payments);
    }

    // Events

    #[event("validatorRegisterEvent")]
    fn validator_register_event(&self, #[indexed] validator: ManagedAddress, name: ManagedBuffer);

    #[event("validatorAddBlsKeysEvent")]
    fn validator_add_bls_keys_event(
        &self,
        #[indexed] validator: ManagedAddress,
        bls_keys: ManagedVec<BlsKey<Self::Api>>,
    );

    #[event("validatorRemoveBlsKeysEvent")]
    fn validator_remove_bls_keys_event(
        &self,
        #[indexed] validator: ManagedAddress,
        bls_keys: ManagedVec<BlsKey<Self::Api>>,
    );

    #[event("validatorSetFeeEvent")]
    fn validator_set_fee_event(&self, #[indexed] validator: ManagedAddress, fee_percent: Percent);

    #[event("validatorSetMaxDelegationEvent")]
    fn validator_set_max_delegation_event(
        &self,
        #[indexed] validator: ManagedAddress,
        max_delegation: BigUint,
    );

    #[event("validatorAddOwnDelegationEvent")]
    fn validator_add_own_delegation_event(
        &self,
        #[indexed] validator: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    );
}
