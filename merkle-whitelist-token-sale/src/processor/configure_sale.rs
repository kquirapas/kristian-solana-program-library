use super::ConfigureSaleAccounts;
use crate::error::TokenSaleError;
use crate::merkle::WhitelistRoot;
use crate::pda::TokenBasePDA;
use crate::state::{self, TokenBase};
use crate::{instruction::accounts::*, require};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::{error::TokenError, state::Mint};

/// Update your Token Sale configuration
///
/// - Modifies one or more than from: price, default_purchase_limit, whitelist_root
/// of [`TokenBase`]
///
/// For Token Sale Authority
///
/// Accounts
/// 0. `[WRITE]`    `Token Base` config account, PDA generated offchain
/// 1. `[]`         `Mint` account
/// 2. `[SIGNER]`   `Sale Authority` account
///
/// Instruction Data
/// - price: Option<u64>,
/// - purchase_limit: Option<u64>,
/// - whitelist_root: Option<WhitelistRoot>,
///
/// Data Validations
/// - (None)
pub fn process_configure_sale(
    program_id: &Pubkey,
    ctx: Context<ConfigureSaleAccounts>,
    price: Option<u64>,
    default_purchase_limit: Option<u64>,
    whitelist_root: Option<WhitelistRoot>,
) -> ProgramResult {
    //---------- Account Validations ----------

    // 0. token_base
    //
    // - account is initialized
    // - token_base seeds must be ["token_base", pubkey(sale_authority), pubkey(mint)]

    // - account is initialized
    let token_base_data = ctx.accounts.token_base.try_borrow_data()?;
    require!(
        token_base_data.len() == state::TokenBase::LEN,
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

    // ensure fail if none of price, default_purchase_limit, and whitelist_root
    // is Some(). It means it'll be a wasteful tx. No state change needed
    assert!(price.is_some() || default_purchase_limit.is_some() || whitelist_root.is_some());

    //---------- Executing Instruction ----------
    let mut token_base_data = ctx.accounts.token_base.try_borrow_mut_data()?;
    let mut token_base = TokenBase::try_from_slice(&token_base_data)?;

    // configure sale

    if let Some(price) = price {
        token_base.price = price;
    }

    if let Some(dpl) = default_purchase_limit {
        token_base.default_purchase_limit = dpl;
    }

    if let Some(root) = whitelist_root {
        token_base.whitelist_root = root;
    }

    // store new values
    token_base.serialize(&mut &mut token_base_data[..]).unwrap();

    Ok(())
}
