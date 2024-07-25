use solana_program::pubkey::Pubkey;

/// Finds the [`TokenBase`] PDA with canonical bump
///
/// - Used for validating TokenBase seeds
pub fn find_token_base_pda(
    program_id: &Pubkey,
    sale_authority: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "token_base".as_bytes(),
            sale_authority.as_ref(),
            mint.as_ref(),
        ],
        program_id,
    )
}

/// Finds the [`BuyerFacts`] PDA with canonical bump
///
/// - Used for validating BuyerFacts seeds
pub fn find_buyer_facts_pda(program_id: &Pubkey, buyer: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &["buyer_facts".as_bytes(), buyer.as_ref(), mint.as_ref()],
        program_id,
    )
}
