use crate::error::TokenSaleError;
use crate::pda::BuyerFactsPDA;
use crate::state::{BuyerFacts, TokenBase};
use crate::{instruction::accounts::*, require};
use borsh::BorshDeserialize;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

/// Close a buyer's BuyerFacts
///
/// - Closes the [`BuyerFacts`] account
/// - Relinquishes rent lamports
///
/// For Buyer
///
/// Accounts
/// 0. `[]`         `Token Base` config account, PDA generated offchain
/// 1. `[WRITE]`    `Buyer Facts` buyer config account, PDA generated offchain
/// 2. `[SIGNER]`   `Buyer` account
///
/// Instruction Data
/// - (None)
///
/// Data Validations
/// - (None)
pub fn process_deregister_buyer(
    program_id: &Pubkey,
    ctx: Context<DeregisterBuyerAccounts>,
) -> ProgramResult {
    //---------- Account Validations ----------

    // 0. token_base
    //
    // - account is initialized

    // - account is initialized
    let token_base_data = ctx.accounts.token_base.try_borrow_mut_data()?;
    let token_base = TokenBase::try_from_slice(&token_base_data)?;
    require!(
        token_base.is_initialized(),
        ProgramError::UninitializedAccount,
        "token_base"
    );

    // 1. buyer_facts
    //
    // - account is initialized
    // - seeds must be ["buyer_facts", pubkey(token_base), pubkey(buyer)]

    // - account is initialized
    let mut buyer_facts_data = ctx.accounts.buyer_facts.try_borrow_mut_data()?;
    let buyer_facts = BuyerFacts::try_from_slice(&buyer_facts_data)?;
    require!(
        buyer_facts.is_initialized(),
        ProgramError::UninitializedAccount,
        "buyer_facts"
    );

    // - seeds must be ["buyer_facts", pubkey(token_base), pubkey(buyer)]
    let (buyer_facts_pda, _) = BuyerFactsPDA::find_pda(
        program_id,
        ctx.accounts.token_base.key,
        ctx.accounts.buyer.key,
    );
    require!(
        *ctx.accounts.buyer_facts.key == buyer_facts_pda,
        ProgramError::InvalidSeeds,
        "buyer_facts"
    );

    // 2. buyer
    //
    // - not executable
    // - must be signer

    // - not executable
    require!(
        !ctx.accounts.buyer.executable,
        TokenSaleError::MustBeNonExecutable,
        "buyer"
    );

    // - must be signer
    require!(
        ctx.accounts.buyer.is_signer,
        TokenSaleError::NeedSigner,
        "buyer"
    );

    //---------- Data Validations (if any) ----------

    //---------- Executing Instruction ----------

    // buyer_facts
    let buyer_facts_account_info = ctx.accounts.buyer_facts;
    let buyer_facts_lamports = buyer_facts_account_info.lamports();

    // buyer
    let buyer_account_info = ctx.accounts.buyer;
    let buyer_lamports = buyer_account_info.lamports();

    // - Relinquishes rent lamports

    // direct transfer buyer_facts (PDA) lamports into buyer
    // NOTE: Direct transfer is okay since token_base is a PDA owned by buyer
    **buyer_account_info.try_borrow_mut_lamports()? = buyer_lamports
        .checked_add(buyer_facts_lamports) // None if overflow
        .unwrap();

    // zero out token_base (PDA) lamports
    **buyer_facts_account_info.try_borrow_mut_lamports()? = 0;

    // - Closes the [`BuyerFacts`] account
    // fill with 0s = no data
    buyer_facts_data.fill(0);

    Ok(())
}
