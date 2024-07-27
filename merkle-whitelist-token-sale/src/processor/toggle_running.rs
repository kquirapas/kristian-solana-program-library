use super::ToggleRunningAccounts;
use crate::error::TokenSaleError;
use crate::pda::TokenBasePDA;
use crate::state::TokenBase;
use crate::{instruction::accounts::*, require};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::{error::TokenError, state::Mint};

/// Start opened Token Sale (allow buying)
///
/// Accounts
/// 0. `[WRITE]`    `Token Base` config account, PDA generated offchain
/// 1. `[]`         `Mint` account
/// 2. `[SIGNER]`   `Sale Authority` account
///
/// Instruction Data
/// - (None)
///
/// Data Validations
/// - (None)
pub fn process_update_running(
    program_id: &Pubkey,
    ctx: Context<ToggleRunningAccounts>,
) -> ProgramResult {
    //---------- Account Validations ----------

    // 0. token_base
    //
    // - account is initialized
    // - token_base seeds must be ["token_base", pubkey(sale_authority), pubkey(mint)]

    // - account is initialized
    let mut token_base_data = ctx.accounts.token_base.try_borrow_mut_data()?;
    let mut token_base = TokenBase::try_from_slice(&token_base_data)?;
    require!(
        token_base.is_initialized(),
        ProgramError::UninitializedAccount,
        "token_base"
    );

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
        TokenSaleError::NeedSigner,
        "sale_authority"
    );

    //---------- Data Validations (if any) ----------

    //---------- Executing Instruction ----------
    // update is_running
    token_base.is_running = !token_base.is_running;

    // store new values
    token_base.serialize(&mut &mut token_base_data[..]).unwrap();

    Ok(())
}
