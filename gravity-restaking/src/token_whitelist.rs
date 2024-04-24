multiversx_sc::imports!();

pub const DEFAULT_TOKEN_DECIMALS: usize = 18;

#[multiversx_sc::module]
pub trait TokenWhitelistModule {
    #[only_owner]
    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        staked_egld_for_one_token: BigUint,
        opt_custom_token_decimals: OptionalValue<usize>,
    ) {
        self.staked_egld_for_one_token(&token_id)
            .set(staked_egld_for_one_token);

        if let OptionalValue::Some(custom_token_decimals) = opt_custom_token_decimals {
            self.custom_token_decimals(&token_id)
                .set(custom_token_decimals);
        }

        let is_new = self.token_whitelist().insert(token_id);
        require!(is_new, "Token already whitelisted");
    }

    #[only_owner]
    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        let was_removed = self.token_whitelist().swap_remove(&token_id);
        require!(was_removed, "Unknown token ID");

        self.staked_egld_for_one_token(&token_id).clear();
        self.custom_token_decimals(&token_id).clear();
    }

    #[only_owner]
    #[endpoint(setStakedEgldAmount)]
    fn set_staked_egld_amount(
        &self,
        token_id: TokenIdentifier,
        staked_egld_for_one_token: BigUint,
    ) {
        self.require_token_in_whitelist(&token_id);

        self.staked_egld_for_one_token(&token_id)
            .set(staked_egld_for_one_token);
    }

    fn get_total_staked_egld(&self, token_id: &TokenIdentifier, amount: &BigUint) -> BigUint {
        let staked_egld_one_token = self.staked_egld_for_one_token(token_id).get();
        let decimals = self.get_token_decimals(token_id);

        staked_egld_one_token * amount / BigUint::from(10u32).pow(decimals as u32)
    }

    fn get_token_decimals(&self, token_id: &TokenIdentifier) -> usize {
        let decimals_mapper = self.custom_token_decimals(token_id);
        if decimals_mapper.is_empty() {
            return DEFAULT_TOKEN_DECIMALS;
        }

        decimals_mapper.get()
    }

    fn require_token_in_whitelist(&self, token_id: &TokenIdentifier) {
        require!(self.token_whitelist().contains(token_id), "Invalid token");
    }

    #[view(getTokenWhitelist)]
    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getStakedEgldForOneToken)]
    #[storage_mapper("stkEgldTok")]
    fn staked_egld_for_one_token(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("customTokenDecimals")]
    fn custom_token_decimals(&self, token_id: &TokenIdentifier) -> SingleValueMapper<usize>;
}
