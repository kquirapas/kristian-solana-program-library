use crate::error::TokenSaleError;
use crate::pda::BuyerFactsPDA;
use crate::state::{BuyerFacts, TokenBase};
use crate::{instruction::accounts::*, require};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program::ID as SYSTEM_PROGRAM_ID, sysvar::Sysvar,
};
use spl_discriminator::SplDiscriminate;

/// Register as a Buyer
///
/// - Generates the buyer's BuyerFacts
///
/// For Buyer
///
/// Accounts
/// 0. `[]`         `Token Base` config account, PDA generated offchain
/// 1. `[WRITE]`    `Buyer Facts` buyer config account, PDA generated offchain
/// 2. `[SIGNER]`   `Buyer` account
/// 3. `[]`         `System Program`
///
/// Instruction Data
/// - (None)
///
/// Data Validations
/// - (None)
pub fn process_register_buyer(
    program_id: &Pubkey,
    ctx: Context<RegisterBuyerAccounts>,
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
    // - account is uninitialized
    // - seeds must be ["buyer_facts", pubkey(token_base), pubkey(buyer)]

    // - account is uninitialized
    let buyer_facts_data = ctx.accounts.buyer_facts.try_borrow_data()?;
    require!(
        buyer_facts_data.len() == 0,
        ProgramError::AccountAlreadyInitialized,
        "buyer_facts"
    );
    drop(buyer_facts_data);

    // - token_base seeds must be ["token_base", pubkey(mint)]
    let (buyer_facts_pda, buyer_facts_canonical_bump) = BuyerFactsPDA::find_pda(
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

    // 3. system_program
    //
    // - must be official system program
    require!(
        *ctx.accounts.system_program.key == SYSTEM_PROGRAM_ID,
        ProgramError::IncorrectProgramId,
        "system_program"
    );

    //---------- Data Validations (if any) ----------

    //---------- Executing Instruction ----------

    // inititalize token_base
    let rent_sysvar = &Rent::get()?;

    invoke_signed(
        &system_instruction::create_account(
            ctx.accounts.buyer.key,
            ctx.accounts.buyer_facts.key,
            rent_sysvar.minimum_balance(BuyerFacts::LEN),
            BuyerFacts::LEN as u64,
            program_id,
        ),
        &[ctx.accounts.buyer.clone(), ctx.accounts.buyer_facts.clone()],
        &[&[
            BuyerFactsPDA::NAME.as_bytes(),
            ctx.accounts.token_base.key.as_ref(),
            ctx.accounts.buyer.key.as_ref(),
            &[buyer_facts_canonical_bump],
        ]],
    )?;

    let mut buyer_facts_data = ctx.accounts.buyer_facts.try_borrow_mut_data()?;
    let mut buyer_facts = BuyerFacts::try_from_slice(&buyer_facts_data)?;

    // update values
    buyer_facts.discriminator = BuyerFacts::SPL_DISCRIMINATOR.into();
    buyer_facts.purchase_limit = token_base.default_purchase_limit;
    buyer_facts.bump = buyer_facts_canonical_bump;

    // store new values
    buyer_facts
        .serialize(&mut &mut buyer_facts_data[..])
        .unwrap();

    Ok(())
}
