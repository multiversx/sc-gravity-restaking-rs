use crate::call_delegation::EGLD_TOKEN_ID;

multiversx_sc::imports!();

pub const BASE_FOR_DECIMALS: u32 = 10;
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

    /// Note: Don't remove and add to set the staked_egld_for_one_token parameter, you'll ruin the internal consistency
    #[only_owner]
    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        let was_removed = self.token_whitelist().swap_remove(&token_id);
        require!(was_removed, "Unknown token ID");

        self.staked_egld_for_one_token(&token_id).clear();
        self.custom_token_decimals(&token_id).clear();
    }

    #[view(getTokenDecimals)]
    fn get_token_decimals(&self, token_id: &TokenIdentifier) -> usize {
        let decimals_mapper = self.custom_token_decimals(token_id);
        if decimals_mapper.is_empty() {
            return DEFAULT_TOKEN_DECIMALS;
        }

        decimals_mapper.get()
    }

    fn get_total_staked_egld(&self, token_id: &TokenIdentifier, amount: &BigUint) -> BigUint {
        if token_id == &TokenIdentifier::from_esdt_bytes(EGLD_TOKEN_ID) {
            return amount.clone();
        }

        let staked_egld_one_token = self.staked_egld_for_one_token(token_id).get();
        let decimals = self.get_token_decimals(token_id);

        staked_egld_one_token * amount / BigUint::from(BASE_FOR_DECIMALS).pow(decimals as u32)
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

    #[storage_mapper("custTokDec")]
    fn custom_token_decimals(&self, token_id: &TokenIdentifier) -> SingleValueMapper<usize>;
}
