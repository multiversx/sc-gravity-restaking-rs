use crate::unique_payments::{PaymentsVec, UniquePayments};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UserModule: crate::token_whitelist::TokenWhitelistModule + utils::UtilsModule {
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) {
        let payments = self.get_non_empty_payments();
        let caller = self.blockchain().get_caller();
        let ids_mapper = self.user_ids();
        let mut caller_id = ids_mapper.get_id(&caller);
        let mut user_tokens = if caller_id == NULL_ID {
            caller_id = ids_mapper.insert_new(&caller);

            UniquePayments::new()
        } else {
            self.user_tokens(caller_id).get()
        };

        let whitelist_mapper = self.token_whitelist();
        for payment in &payments {
            require!(
                whitelist_mapper.contains(&payment.token_identifier),
                "Invalid token"
            );

            user_tokens.add_payment(payment);
        }

        self.user_tokens(caller_id).set(user_tokens);
    }

    /// Pairs of (token_id, nonce, amount)
    #[endpoint]
    fn withdraw(&self, tokens: MultiValueEncoded<MultiValue3<TokenIdentifier, u64, BigUint>>) {
        require!(!tokens.is_empty(), "No arguments");

        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);

        let mut output_payments = PaymentsVec::new();
        self.user_tokens(caller_id).update(|user_tokens| {
            for token_tuple in tokens {
                let (token_id, nonce, amount) = token_tuple.into_tuple();
                require!(amount > 0, "Can't withdraw 0");

                let payment = EsdtTokenPayment::new(token_id, nonce, amount);
                let deduct_result = user_tokens.deduct_payment(&payment);
                require!(deduct_result.is_ok(), "Trying to withdraw too many tokens");

                output_payments.push(payment);
            }
        });

        self.send().direct_multi(&caller, &output_payments);
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self) {
        let caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&caller);
        let user_tokens = self.user_tokens(caller_id).update(|user_tokens| {
            let output = (*user_tokens).clone();
            *user_tokens = UniquePayments::new();

            output
        });

        let output_payments = user_tokens.into_payments();
        require!(!output_payments.is_empty(), "Nothing to withdraw");

        self.send().direct_multi(&caller, &output_payments);
    }

    #[storage_mapper("userIds")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[view(getUserTokens)]
    #[storage_mapper("userTokens")]
    fn user_tokens(&self, user_id: AddressId) -> SingleValueMapper<UniquePayments<Self::Api>>;
}
