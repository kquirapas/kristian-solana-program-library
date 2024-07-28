use super::AssignLimitAccounts;
use crate::error::TokenSaleError;
use crate::pda::BuyerFactsPDA;
use crate::state::BuyerFacts;
use crate::{instruction::accounts::*, require};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

/// Assign a user's purchase limit
///
/// - Changes the `purchase_limit` of a certain buyer's
/// BuyerFacts
///
/// For Token Sale Authority
///
/// Accounts
/// 0. `[]`         `Token Base` buyer config account, PDA generated offchain
/// 1. `[WRITE]`    `Buyer Facts` buyer config account, PDA generated offchain
/// 2. `[]`         `Buyer` account
/// 3. `[SIGNER]`   `Sale Authority` account
///
/// Instruction Data
/// - new_purchase_limit: u64,
///
/// Data Validations
/// - (None)
pub fn process_assign_limit(
    program_id: &Pubkey,
    ctx: Context<AssignLimitAccounts>,
    new_purchase_limit: u64,
) -> ProgramResult {
    //---------- Account Validations ----------

    // 0. buyer_facts
    //
    // - account is initialized
    // - buyer_facts seeds must be ["token_base", pubkey(token_base), pubkey(buyer)]

    // - account is initialized
    let mut buyer_facts_data = ctx.accounts.buyer_facts.try_borrow_mut_data()?;
    let mut buyer_facts = BuyerFacts::try_from_slice(&buyer_facts_data)?;
    require!(
        buyer_facts.is_initialized(),
        ProgramError::UninitializedAccount,
        "buyer_facts"
    );

    // - buyer_facts seeds must be ["buyer_facts", pubkey(token_base), pubkey(buyer)]
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

    // 1. buyer
    //
    // - not executable

    // - not executable
    require!(
        !ctx.accounts.buyer.executable,
        TokenSaleError::MustBeNonExecutable,
        "buyer"
    );

    // 3. sale_authority
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
    buyer_facts.purchase_limit = new_purchase_limit;

    // store new values
    buyer_facts
        .serialize(&mut &mut buyer_facts_data[..])
        .unwrap();

    Ok(())
}
