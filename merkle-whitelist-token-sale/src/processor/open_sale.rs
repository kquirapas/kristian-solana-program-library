use crate::error::TokenSaleError;
use crate::merkle::WhitelistRoot;
use crate::pda::find_token_base_pda;
use crate::state::TokenBase;
use crate::{
    instruction::accounts::{Context, OpenSaleAccounts},
    require,
};
use borsh::BorshDeserialize;
use solana_program::{
    entrypoint::ProgramResult, msg, program::invoke_signed, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction,
    system_program::ID as SYSTEM_PROGRAM_ID, sysvar::Sysvar,
};
use spl_token::{error::TokenError, state::Mint};

/// Open a Token Sale with the given config
///
/// Validates the accounts and data passed then
/// initializes the [`TokenBase`] (config)
///
/// Accounts
/// 0. `[WRITE]`    `Token Base` config account, PDA generated offchain
/// 1. `[]`         `Mint` account
/// 2. `[]`         `Vault` account
/// 3. `[SIGNER]`   `Sale Authority` account
/// 4. `[]`         `System Program`
///
/// Instruction Data
/// - price: u64,
/// - purchase_limit: u64,
/// - whitelist_root: [u8; 32],
///
/// Data Validations
/// -
pub fn process_open_sale(
    program_id: &Pubkey,
    ctx: Context<OpenSaleAccounts>,
    price: u64,
    purchase_limit: u64,
    whitelist_root: WhitelistRoot,
) -> ProgramResult {
    //---------- Account Validations ----------

    // 0. token_base
    //
    // - account is uninitialized
    // - token_base seeds must be ["token_base", pubkey(sale_authority), pubkey(mint)]

    // - account is uninitialized
    let token_base_data = ctx.accounts.token_base.try_borrow_data()?;
    require!(
        token_base_data.len() == 0,
        ProgramError::AccountAlreadyInitialized,
        "token_base"
    );
    drop(token_base_data);

    // - token_base seeds must be ["token_base", pubkey(mint)]
    let (token_base_pda, token_base_bump) = find_token_base_pda(
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

    // 2. vault
    //
    // - not executable
    let vault = ctx.accounts.vault;

    // - not executable
    require!(
        !vault.executable,
        TokenSaleError::MustBeNonExecutable,
        "vault"
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
        TokenSaleError::SaleAuthorityNotSigner,
        "sale_authority"
    );

    // 4. system_program
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
            ctx.accounts.sale_authority.key,
            ctx.accounts.token_base.key,
            rent_sysvar.minimum_balance(TokenBase::LEN),
            TokenBase::LEN as u64,
            program_id,
        ),
        &[
            ctx.accounts.sale_authority.clone(),
            ctx.accounts.token_base.clone(),
        ],
        &[&[
            b"token_base",
            ctx.accounts.sale_authority.key.as_ref(),
            ctx.accounts.mint.key.as_ref(),
            &[token_base_bump],
        ]],
    )?;

    let token_base_data = ctx.accounts.token_base.try_borrow_mut_data()?;
    let mut token_base = TokenBase::try_from_slice(&token_base_data)?;

    token_base.mint = *mint.key;
    token_base.vault = *vault.key;
    token_base.sale_authority = *sale_authority.key;
    token_base.whitelist_root = whitelist_root;
    token_base.price = price;
    token_base.default_purchase_limit = purchase_limit;
    token_base.bump = token_base_bump; // store canonical bump

    Ok(())
}
