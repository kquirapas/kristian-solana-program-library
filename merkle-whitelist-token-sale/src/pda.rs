use solana_program::pubkey::Pubkey;

/// Finds the [`TokenBase`] PDA with canonical bump
///
/// - Used for validating TokenBase seeds
/// - Used for CPIs
pub struct TokenBasePDA {}

impl TokenBasePDA {
    pub const NAME: &'static str = "token_base";

    pub fn find_pda(program_id: &Pubkey, sale_authority: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::NAME.as_bytes(),
                sale_authority.as_ref(),
                mint.as_ref(),
            ],
            program_id,
        )
    }
}

/// Finds the [`BuyerFacts`] PDA with canonical bump
///
/// - Used for validating BuyerFacts seeds
/// - Used for CPIs
pub struct BuyerFactsPDA {}

impl BuyerFactsPDA {
    pub const NAME: &'static str = "buyer_facts";

    pub fn find_pda(program_id: &Pubkey, token_base: &Pubkey, buyer: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::NAME.as_bytes(), token_base.as_ref(), buyer.as_ref()],
            program_id,
        )
    }
}
