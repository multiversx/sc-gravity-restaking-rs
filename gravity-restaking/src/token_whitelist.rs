multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait TokenWhitelistModule {
    #[only_owner]
    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        staked_egld_for_one_token: BigUint,
    ) {
        self.staked_egld_for_one_token(&token_id)
            .set(staked_egld_for_one_token);

        let is_new = self.token_whitelist().insert(token_id);
        require!(is_new, "Token already whitelisted");
    }

    #[only_owner]
    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        let was_removed = self.token_whitelist().swap_remove(&token_id);
        require!(was_removed, "Unknown token ID");

        self.staked_egld_for_one_token(&token_id).clear();
    }

    #[view(getTokenWhitelist)]
    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getStakedEgldForOneToken)]
    #[storage_mapper("stkEgldTok")]
    fn staked_egld_for_one_token(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;
}
