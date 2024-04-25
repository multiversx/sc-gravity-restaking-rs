multiversx_sc::imports!();
multiversx_sc::derive_imports!();

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

    #[view(getSovInfo)]
    fn get_sov_info(&self, sov_address: ManagedAddress) -> SovereignInfo<Self::Api> {
        let sov_id = self.sovereign_id().get_id_non_zero(&sov_address);

        self.sovereign_info(sov_id).get()
    }

    #[storage_mapper("sovId")]
    fn sovereign_id(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("sovInfo")]
    fn sovereign_info(&self, sov_id: AddressId) -> SingleValueMapper<SovereignInfo<Self::Api>>;

    #[storage_mapper("sovForName")]
    fn sov_chain_for_name(&self, name: &ManagedBuffer) -> SingleValueMapper<AddressId>;
}
