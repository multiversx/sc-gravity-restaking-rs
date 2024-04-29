use mergeable::Mergeable;

use crate::unique_payments::UniquePayments;

use super::sovereign::Epoch;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct UnbondInfo<M: ManagedTypeApi> {
    pub tokens: UniquePayments<M>,
    pub unbond_epoch: Epoch,
}

impl<M: ManagedTypeApi> UnbondInfo<M> {
    #[inline]
    pub fn new(tokens: UniquePayments<M>, unbond_epoch: Epoch) -> Self {
        Self {
            tokens,
            unbond_epoch,
        }
    }
}

impl<M: ManagedTypeApi> Mergeable<M> for UnbondInfo<M> {
    fn can_merge_with(&self, other: &Self) -> bool {
        self.unbond_epoch == other.unbond_epoch
    }

    fn merge_with(&mut self, other: Self) {
        self.error_if_not_mergeable(&other);

        self.tokens.merge_with(other.tokens);
    }
}

#[multiversx_sc::module]
pub trait UnbondModule {
    #[only_owner]
    #[endpoint(setUnbondEpochs)]
    fn set_unbond_epochs(&self, unbond_epochs: Epoch) {
        self.unbond_epochs().set(unbond_epochs);
    }

    fn add_unbond_tokens(&self, user_id: AddressId, tokens: UniquePayments<Self::Api>) {
        let unbond_epochs = self.unbond_epochs().get();
        let current_epoch = self.blockchain().get_block_epoch();
        let final_unbond_epoch = current_epoch + unbond_epochs;
        let mut current_unbond_info = UnbondInfo::new(tokens, final_unbond_epoch);

        let unbond_mapper = self.unbond_info(user_id);
        if unbond_mapper.is_empty() {
            unbond_mapper.set(ManagedVec::from_single_item(current_unbond_info));

            return;
        }

        let mut all_user_unbonds = unbond_mapper.get();
        for (i, unbond_info) in all_user_unbonds.iter().enumerate() {
            if !current_unbond_info.can_merge_with(&unbond_info) {
                continue;
            }

            current_unbond_info.merge_with(unbond_info);
            let _ = all_user_unbonds.set(i, &current_unbond_info);
            unbond_mapper.set(all_user_unbonds);

            return;
        }

        all_user_unbonds.push(current_unbond_info);
        unbond_mapper.set(all_user_unbonds);
    }

    fn unbond_tokens_common(&self, user_id: AddressId) -> UniquePayments<Self::Api> {
        let mut result = UniquePayments::new();

        let current_epoch = self.blockchain().get_block_epoch();
        self.unbond_info(user_id).update(|user_unbond_info| {
            let mut i = 0;
            let mut vec_len = user_unbond_info.len();
            while !user_unbond_info.is_empty() && i < vec_len {
                let current_unbond_info = user_unbond_info.get(i);
                if current_unbond_info.unbond_epoch > current_epoch {
                    i += 1;

                    continue;
                }

                result.merge_with(current_unbond_info.tokens);
                user_unbond_info.remove(0);
                vec_len -= 1;
            }
        });

        result
    }

    #[storage_mapper("unbondEpochs")]
    fn unbond_epochs(&self) -> SingleValueMapper<Epoch>;

    #[storage_mapper("unbondTokens")]
    fn unbond_info(
        &self,
        user_id: AddressId,
    ) -> SingleValueMapper<ManagedVec<UnbondInfo<Self::Api>>>;
}
