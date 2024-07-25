use crate::error::TokenSaleError;
use crate::pda::find_token_base_pda;
use crate::state::TokenBase;
use crate::{
    instruction::accounts::{CloseSaleAccounts, Context},
    require,
};
use borsh::BorshDeserialize;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

/// Close the token sale
///
/// - Relinquishes rent lamports
/// - Closes the [`TokenBase`] account
///
/// Accounts
/// 0. `[WRITE]`    `Token Base` config account, PDA generated offchain
/// 1. `[SIGNER]`   `Sale Authority` account
///
/// Instruction Data
/// - (Empty, None, Nada! HAHAHA)
pub fn process_close_sale(program_id: &Pubkey, ctx: Context<CloseSaleAccounts>) -> ProgramResult {
    //---------- Account Validations ----------

    // 0. token_base
    //
    // - owner is token_sale (this) program
    // - correct allocation length (TokenBase::LEN)
    // - account is initialized
    // - token_base seeds must be ["token_base", pubkey(mint)]

    // - owner is token_sale (this) program
    require!(
        ctx.accounts.token_base.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "token_base"
    );

    // - correct allocation length (TokenBase::LEN)
    let token_base_data = ctx.accounts.token_base.try_borrow_mut_data()?;
    require!(
        token_base_data.len() == TokenBase::LEN,
        TokenSaleError::InvalidAccountDataLength,
        "token_base"
    );

    // - account is initialized
    let token_base = TokenBase::try_from_slice(&token_base_data)?;
    require!(
        token_base.is_initialized(),
        TokenSaleError::AccountUninitialized,
        "token_base"
    );

    // - token_base seeds must be ["token_base", pubkey(sale_authority), pubkey(mint)]
    let (token_base_pda, _) =
        find_token_base_pda(program_id, &token_base.sale_authority, &token_base.mint);
    require!(
        *ctx.accounts.token_base.key == token_base_pda,
        TokenSaleError::UnexpectedPDASeeds,
        "token_base"
    );

    // 1. sale_authority
    //
    // - not executable
    // - must be signer
    let sale_authority = ctx.accounts.sale_authority;

    // - not executable
    require!(
        !sale_authority.executable,
        TokenSaleError::MustBeNonExecutable,
        "sale_authority"
    );

    // - must be signer
    require!(
        sale_authority.is_signer,
        TokenSaleError::SaleAuthorityNotSigner,
        "sale_authority"
    );

    //---------- Data Validations (if any) ----------

    //---------- Executing Instruction ----------

    // token_base
    let token_base_account_info = ctx.accounts.token_base;
    let token_base_lamports = token_base_account_info.lamports();

    // sale_authority
    let sale_authority_account_info = ctx.accounts.sale_authority;
    let sale_authority_lamports = sale_authority_account_info.lamports();

    // - Relinquishes rent lamports

    // direct transfer token_base (PDA) lamports into sale_authority
    // NOTE: Direct transfer is okay since token_base is a PDA owned by sale_authority
    **sale_authority_account_info.try_borrow_mut_lamports()? = sale_authority_lamports
        .checked_add(token_base_lamports) // None if overflow
        .unwrap();

    // zero out token_base (PDA) lamports
    **token_base_account_info.try_borrow_mut_lamports()? = 0;

    // - Closes the [`TokenBase`] account
    let mut token_base_data = token_base_account_info.try_borrow_mut_data()?;
    token_base_data.fill(0);

    Ok(())
}
