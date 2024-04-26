use multiversx_sc::api::StorageMapperApi;

use crate::{
    unique_payments::{PaymentsVec, UniquePayments},
    validator::ValidatorConfig,
};

multiversx_sc::imports!();

pub struct AddDelegationArgs<'a, M: StorageMapperApi> {
    pub total_amount_mapper: SingleValueMapper<M, BigUint<M>>,
    pub total_by_user_mapper: SingleValueMapper<M, BigUint<M>>,
    pub all_delegators_mapper: &'a mut UnorderedSetMapper<M, AddressId>,
    pub delegated_by_mapper: SingleValueMapper<M, UniquePayments<M>>,
    pub opt_validator_config_mapper: Option<SingleValueMapper<M, ValidatorConfig<M>>>,
    pub payments_to_add: PaymentsVec<M>,
    pub total_amount: BigUint<M>,
    pub caller_id: AddressId,
}

#[multiversx_sc::module]
pub trait CommonActionsModule {
    fn add_delegation(&self, args: AddDelegationArgs<Self::Api>) {
        args.total_amount_mapper.update(|total_del| {
            *total_del += &args.total_amount;

            if let Some(config_mapper) = args.opt_validator_config_mapper {
                let config = config_mapper.get();
                if let Some(max_amt) = config.opt_max_delegation {
                    require!(*total_del <= max_amt, "Max delegated amount exceeded");
                }
            }
        });

        args.total_by_user_mapper
            .update(|total_user| *total_user += args.total_amount);

        let _ = args.all_delegators_mapper.insert(args.caller_id);

        let mut tokens_delegated_by_user = if !args.delegated_by_mapper.is_empty() {
            args.delegated_by_mapper.get()
        } else {
            UniquePayments::new()
        };
        for payment in &args.payments_to_add {
            tokens_delegated_by_user.add_payment(payment);
        }
        args.delegated_by_mapper.set(tokens_delegated_by_user);
    }
}
