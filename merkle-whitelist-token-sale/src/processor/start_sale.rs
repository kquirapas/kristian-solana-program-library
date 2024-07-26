use crate::error::TokenSaleError;
use crate::merkle::WhitelistRoot;
use crate::pda::TokenBasePDA;
use crate::state::TokenBase;
use crate::{instruction::accounts::*, require};
use borsh::BorshDeserialize;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use super::StartSaleAccounts;

pub fn process_start_sale(program_id: &Pubkey, ctx: Context<StartSaleAccounts>) -> ProgramResult {
    // //---------- Account Validations ----------
    //
    // // 0. token_base
    // //
    // // - owner is token_sale (this) program
    // // - correct allocation length (TokenBase::LEN)
    // // - account is initialized
    // // - token_base seeds must be ["token_base", pubkey(mint)]
    //
    // // - owner is token_sale (this) program
    // require!(
    //     ctx.accounts.token_base.owner == program_id,
    //     ProgramError::InvalidAccountOwner,
    //     "token_base"
    // );
    //
    // // - correct allocation length (TokenBase::LEN)
    // let token_base_data = ctx.accounts.token_base.try_borrow_mut_data()?;
    // require!(
    //     token_base_data.len() == TokenBase::LEN,
    //     TokenSaleError::InvalidAccountDataLength,
    //     "token_base"
    // );
    //
    // // - account is initialized
    // let mut token_base = TokenBase::try_from_slice(&token_base_data)?;
    // require!(
    //     token_base.is_initialized(),
    //     TokenSaleError::AccountUninitialized,
    //     "token_base"
    // );
    //
    // // - token_base seeds must be ["token_base", pubkey(sale_authority), pubkey(mint)]
    // let (token_base_pda, _) =
    //     find_token_base_pda(program_id, &token_base.sale_authority, &token_base.mint);
    // require!(
    //     *ctx.accounts.token_base.key == token_base_pda,
    //     TokenSaleError::UnexpectedPDASeeds,
    //     "token_base"
    // );
    //
    // // 1. sale_authority
    // //
    // // - not executable
    // // - must be signer
    // let sale_authority = ctx.accounts.sale_authority;
    //
    // // - not executable
    // require!(
    //     !sale_authority.executable,
    //     TokenSaleError::MustBeNonExecutable,
    //     "sale_authority"
    // );
    //
    // // - must be signer
    // require!(
    //     sale_authority.is_signer,
    //     TokenSaleError::SaleAuthorityNotSigner,
    //     "sale_authority"
    // );
    //
    // // 2. new_vault
    // //
    // // - not executable
    // let new_vault = ctx.accounts.new_vault;
    //
    // // - not executable
    // require!(
    //     !new_vault.executable,
    //     TokenSaleError::MustBeNonExecutable,
    //     "new_vault"
    // );
    //
    // //---------- Data Validations (if any) ----------
    //
    // //---------- Executing Instruction ----------
    //
    // // - vault
    // token_base.vault = *new_vault.key;
    // // - price
    // token_base.price = price;
    // // - whitelist_root
    // token_base.whitelist_root = whitelist_root;
    // // - purchase_limit
    // token_base.default_purchase_limit = purchase_limit;
    //
    Ok(())
}
