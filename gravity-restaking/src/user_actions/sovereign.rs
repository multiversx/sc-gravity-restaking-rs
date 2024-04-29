use crate::unique_payments::UniquePayments;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type Epoch = u64;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct SovereignInfo<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
}

impl<M: ManagedTypeApi> SovereignInfo<M> {
    #[inline]
    pub fn new(name: ManagedBuffer<M>, description: ManagedBuffer<M>) -> Self {
        Self { name, description }
    }
}

#[multiversx_sc::module]
pub trait SovereignModule: utils::UtilsModule {
    #[endpoint(registerSov)]
    fn register_sov(&self, name: ManagedBuffer, description: ManagedBuffer) {
        self.require_not_empty_buffer(&name);

        let caller = self.blockchain().get_caller();
        let caller_id = self.sovereign_id().insert_new(&caller);

        let id_for_name_mapper = self.sov_chain_for_name(&name);
        require!(id_for_name_mapper.is_empty(), "Name already taken");

        self.sovereign_info(caller_id)
            .set(SovereignInfo::new(name, description));
        id_for_name_mapper.set(caller_id);

        // TODO: event
    }

    #[endpoint(setUpRewards)]
    fn set_up_rewards(
        &self,
        _start_epoch: Epoch,
        _end_epoch: Epoch,
        _total_value: BigUint,
        /* _computation: ???, */
    ) {
        let caller = self.blockchain().get_caller();
        let _caller_id = self.sovereign_id().get_id_non_zero(&caller);
        // TODO: Unsure what to do with all this info yet
    }

    #[endpoint(unRegister)]
    fn unregister(&self) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.sovereign_id().remove_by_address(&caller);
        require!(caller_id != NULL_ID, "Unknown sovereign chain");

        let sov_info = self.sovereign_info(caller_id).take();
        self.sov_chain_for_name(&sov_info.name).clear();
    }

    #[payable("*")]
    #[endpoint(addRewards)]
    fn add_rewards(&self) {
        // TODO
    }

    #[view(getSovInfo)]
    fn get_sov_info(&self, sov_address: ManagedAddress) -> SovereignInfo<Self::Api> {
        let sov_id = self.sovereign_id().get_id_non_zero(&sov_address);

        self.sovereign_info(sov_id).get()
    }

    fn require_valid_sov_id(&self, sov_id: AddressId) {
        require!(sov_id != NULL_ID, "Invalid chain name");
    }

    #[storage_mapper("sovId")]
    fn sovereign_id(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("sovInfo")]
    fn sovereign_info(&self, sov_id: AddressId) -> SingleValueMapper<SovereignInfo<Self::Api>>;

    #[storage_mapper("sovForName")]
    fn sov_chain_for_name(&self, name: &ManagedBuffer) -> SingleValueMapper<AddressId>;

    #[storage_mapper("allSovDelegators")]
    fn all_sov_delegators(&self, sov_id: AddressId) -> UnorderedSetMapper<AddressId>;

    #[storage_mapper("delegatedSovBy")]
    fn delegated_sov_by(
        &self,
        user_id: AddressId,
        sov_id: AddressId,
    ) -> SingleValueMapper<UniquePayments<Self::Api>>;

    #[storage_mapper("totalDelegatedSovAmount")]
    fn total_delegated_sov_amount(&self, sov_id: AddressId) -> SingleValueMapper<BigUint>;

    #[storage_mapper("totalSovByUser")]
    fn total_sov_by_user(
        &self,
        user_id: AddressId,
        sov_id: AddressId,
    ) -> SingleValueMapper<BigUint>;
}
