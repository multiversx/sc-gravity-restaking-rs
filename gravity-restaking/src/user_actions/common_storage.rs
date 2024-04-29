use crate::unique_payments::UniquePayments;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CommonStorageModule {
    #[storage_mapper("userIds")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[view(getUserTokens)]
    #[storage_mapper("userTokens")]
    fn user_tokens(&self, user_id: AddressId) -> SingleValueMapper<UniquePayments<Self::Api>>;
}
