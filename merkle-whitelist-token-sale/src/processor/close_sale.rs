use crate::error::TokenSaleError;
use crate::pda::TokenBasePDA;
use crate::state::TokenBase;
use crate::{
    instruction::accounts::{CloseSaleAccounts, Context},
    require,
};
use solana_program::{
    entrypoint::ProgramResult, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::{error::TokenError, state::Mint};

/// Close the token sale
///
/// - Relinquishes rent lamports
/// - Closes the [`TokenBase`] account
///
/// Accounts
/// 0. `[WRITE]`    `Token Base` config account, PDA generated offchain
/// 1. `[]`         `Mint` account
/// 2. `[SIGNER]`   `Sale Authority` account
///
/// Instruction Data
/// - (Empty, None, Nada! HAHAHA)
pub fn process_close_sale(program_id: &Pubkey, ctx: Context<CloseSaleAccounts>) -> ProgramResult {
    //---------- Account Validations ----------

    // 0. token_base
    //
    // - account is initialized
    // - token_base seeds must be ["token_base", pubkey(sale_authority), pubkey(mint)]

    // - account is initialized
    let token_base_data = ctx.accounts.token_base.try_borrow_data()?;
    require!(
        token_base_data.len() == TokenBase::LEN,
        ProgramError::UninitializedAccount,
        "token_base"
    );
    drop(token_base_data);

    // - token_base seeds must be ["token_base", pubkey(mint)]
    let (token_base_pda, _) = TokenBasePDA::find_pda(
        program_id,
        ctx.accounts.sale_authority.key,
        ctx.accounts.mint.key,
    );
    require!(
        *ctx.accounts.token_base.key == token_base_pda,
        ProgramError::InvalidSeeds,
        "token_base"
    );

    // 1. mint
    //
    // - is_initialized is true
    // - mint_authority is token_base sale_authority
    let mint = ctx.accounts.mint;
    let mint_data = mint.try_borrow_data()?;
    let mint_state = Mint::unpack(&mint_data)?;

    // - is_initialized is true
    require!(
        mint_state.is_initialized,
        TokenError::UninitializedState,
        "mint"
    );

    // - mint_authority is token_base sale_authority
    require!(
        mint_state.mint_authority.unwrap() == *ctx.accounts.sale_authority.key,
        TokenSaleError::MintAndSaleAuthorityMismatch,
        "mint"
    );

    // 2. sale_authority
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
    // fill with 0s = no data
    token_base_data.fill(0);

    Ok(())
}
