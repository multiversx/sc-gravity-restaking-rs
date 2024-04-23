multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const BLS_KEY_LEN: usize = 96;
const BLS_SIG_LEN: usize = 48;
const MAX_PERCENT: Percent = 10_000;

pub type BlsKey<M> = ManagedByteArray<M, BLS_KEY_LEN>;
pub type BlsSignature<M> = ManagedByteArray<M, BLS_SIG_LEN>;
pub type Percent = u32;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct ValidatorConfig<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub bls_keys: ManagedVec<M, BlsKey<M>>,
    pub fee: Percent,
    pub opt_max_delegation: Option<BigUint<M>>,
}

impl<M: ManagedTypeApi> ValidatorConfig<M> {
    pub fn new(name: ManagedBuffer<M>) -> Self {
        Self {
            name,
            bls_keys: ManagedVec::new(),
            fee: 0,
            opt_max_delegation: None,
        }
    }
}

#[multiversx_sc::module]
pub trait ValidatorModule: utils::UtilsModule {
    #[endpoint]
    fn register(&self, name: ManagedBuffer) {
        self.require_not_empty_buffer(&name);

        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().insert_new(&caller);

        let id_for_name_mapper = self.id_for_name(&name);
        require!(id_for_name_mapper.is_empty(), "Name already taken");

        self.validator_config(caller_id)
            .set(ValidatorConfig::new(name));
        id_for_name_mapper.set(caller_id);

        // TODO: event
    }

    /// pairs of bls_key and signed message of own address
    #[endpoint(addKeys)]
    fn add_keys(
        &self,
        pairs: MultiValueEncoded<MultiValue2<BlsKey<Self::Api>, BlsSignature<Self::Api>>>,
    ) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().get_id_non_zero(&caller);

        let mut new_bls_keys = ManagedVec::<Self::Api, _>::new();
        for pair in pairs {
            let (bls_key, bls_sig) = pair.into_tuple();
            let valid_sig = self.crypto().verify_bls(
                bls_key.as_managed_buffer(),
                caller.as_managed_buffer(),
                bls_sig.as_managed_buffer(),
            );
            require!(valid_sig, "Invalid BLS signature");

            new_bls_keys.push(bls_key);
        }

        self.validator_config(caller_id).update(|config| {
            config.bls_keys.extend(&new_bls_keys);
        });

        // TODO: event
    }

    #[endpoint(removeKeys)]
    fn remove_keys(&self, keys: MultiValueEncoded<BlsKey<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().get_id_non_zero(&caller);

        let mapper = self.validator_config(caller_id);
        let mut config = mapper.get();
        for key in keys {
            let opt_index = config.bls_keys.find(&key);
            require!(opt_index.is_some(), "Key not found");

            let index = unsafe { opt_index.unwrap_unchecked() };
            config.bls_keys.remove(index);
        }

        mapper.set(config);

        // TODO: event
    }

    // TODO: validateFor@projectID@LIST<BLSKEYS>@LISTOFStakeEGLDAssets

    #[endpoint(setUpFee)]
    fn set_up_fee(&self, fee: Percent) {
        require!(fee <= MAX_PERCENT, "Invalid fee percent");

        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().get_id_non_zero(&caller);
        self.validator_config(caller_id)
            .update(|config| config.fee = fee);

        // TODO: event
    }

    #[endpoint(setMaxDelegation)]
    fn set_max_delegation(&self, max_delegation: BigUint) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.validator_id().get_id_non_zero(&caller);
        self.validator_config(caller_id)
            .update(|config| config.opt_max_delegation = Some(max_delegation));

        // TODO: event
    }

    #[view(getValidatorConfig)]
    fn get_validator_config(&self, address: ManagedAddress) -> ValidatorConfig<Self::Api> {
        let validator_id = self.validator_id().get_id_non_zero(&address);

        self.validator_config(validator_id).get()
    }

    // TODO: addOwnDelegation - multiTransfer of stakedEGLD assets - register these to the validator.

    #[storage_mapper("validatorId")]
    fn validator_id(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("validatorConfig")]
    fn validator_config(
        &self,
        validator_id: AddressId,
    ) -> SingleValueMapper<ValidatorConfig<Self::Api>>;

    #[storage_mapper("idForName")]
    fn id_for_name(&self, name: &ManagedBuffer) -> SingleValueMapper<AddressId>;
}
