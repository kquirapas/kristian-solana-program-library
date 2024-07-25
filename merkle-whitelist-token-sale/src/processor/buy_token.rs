use crate::error::TokenSaleError;
use crate::merkle::WhitelistProof;
use crate::pda::{find_buyer_facts_pda, find_token_base_pda};
use crate::state::{BuyerFacts, TokenBase};
use crate::{
    instruction::accounts::{BuyTokenAccounts, Context},
    require,
};
use borsh::BorshDeserialize;
use solana_program::program::invoke_signed;
use solana_program::{
    entrypoint::ProgramResult, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    system_instruction,
};
use spl_token::{
    error::TokenError,
    instruction,
    state::{Account, AccountState, Mint},
};

/// Buy N amount of Tokens
///
/// - Transfers SOL (lamports) from Buyer to Vault
/// - Mints Token to Buyer account
/// - Creates Buyer Facts
///
/// Accounts
/// 0. `[]`         `Token Base` config account, PDA generated offchain
/// 1. `[]`         `Mint` account, generated offchain
/// 2. `[WRITE]`    `Vault` account
/// 3. `[]`         `Sale Authority` account
/// 4. `[WRITE]`    `Buyer Token Account` account, PDA (ATA) generated offchain
/// 5. `[WRITE]`    `Buyer Facts` account, PDA generated offchain
/// 6. `[SIGNER]`   `Buyer` account
/// 7. `[]`         `Token Program`
///
/// Instruction Data
/// - amount: u64,
/// - proof: WhitelistProof
pub fn process_buy_token(
    program_id: &Pubkey,
    ctx: Context<BuyTokenAccounts>,
    amount: u64,
    proof: WhitelistProof,
) -> ProgramResult {
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
        ProgramError::UninitializedAccount,
        "token_base"
    );

    // - token_base seeds must be ["token_base", pubkey(sale_authority), pubkey(mint)]
    let (token_base_pda, _) = find_token_base_pda(
        program_id,
        ctx.accounts.sale_authority.key,
        ctx.accounts.mint.key,
    );
    require!(
        *ctx.accounts.token_base.key == token_base_pda,
        TokenSaleError::UnexpectedPDASeeds,
        "token_base"
    );

    // 1. mint
    //
    // - token_base mint is mint
    require!(
        token_base.mint == *ctx.accounts.mint.key,
        TokenSaleError::AccountsAndTokenBaseMismatch,
        "mint"
    );

    // 2. vault
    //
    // - token_base vault is vault
    require!(
        token_base.vault == *ctx.accounts.vault.key,
        TokenSaleError::AccountsAndTokenBaseMismatch,
        "vault"
    );

    // 3. sale_authority
    //
    // - sale_authority must also be mint_authority
    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    let mint = Mint::unpack(&mint_data)?;
    require!(
        token_base.sale_authority == mint.mint_authority.unwrap(),
        TokenSaleError::AccountsAndTokenBaseMismatch,
        "vault"
    );

    // 1. buyer_token_account
    //
    // - must be initialized
    // - mint must be token_base mint
    // - owner must be buyer
    let buyer_token_account_data = ctx.accounts.buyer_token_account.try_borrow_data()?;
    let buyer_token_account = Account::unpack(&buyer_token_account_data)?;

    // - must be initialized
    require!(
        buyer_token_account.state == AccountState::Initialized,
        TokenError::UninitializedState,
        "buyer_token_account"
    );

    // - mint must be token_base mint
    require!(
        buyer_token_account.mint == token_base.mint,
        TokenError::InvalidMint,
        "buyer_token_account"
    );

    // - owner must be buyer
    let buyer = *ctx.accounts.buyer.key;
    require!(
        buyer_token_account.owner == buyer,
        TokenError::OwnerMismatch,
        "buyer_token_account"
    );

    // 2. buyer_facts
    //
    // - owner is token_sale (this) program
    // - correct allocation length (BuyerFacts::LEN)
    // - buyer_facts seeds must be ['buyer_facts', `pubkey(buyer)`, `pubkey(mint)`]

    // - owner is token_sale (this) program
    require!(
        ctx.accounts.buyer_facts.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "buyer_facts"
    );

    // - correct allocation length (BuyerFacts::LEN)
    let buyer_facts_data = ctx.accounts.buyer_facts.try_borrow_mut_data()?;
    require!(
        buyer_facts_data.len() == BuyerFacts::LEN,
        TokenSaleError::InvalidAccountDataLength,
        "buyer_facts"
    );

    // - buyer_facts seeds must be ['buyer_facts', `pubkey(buyer)`, `pubkey(mint)`]
    let (buyer_facts_pda, bump) =
        find_buyer_facts_pda(program_id, ctx.accounts.buyer.key, &token_base.mint);

    require!(
        *ctx.accounts.buyer_facts.key == buyer_facts_pda,
        TokenSaleError::UnexpectedPDASeeds,
        "buyer_facts"
    );

    // 3. buyer
    //
    // - not executable
    // - must be signer
    let buyer = ctx.accounts.buyer;

    // - not executable
    require!(
        !buyer.executable,
        TokenSaleError::MustBeNonExecutable,
        "buyer"
    );

    // - must be signer
    require!(
        buyer.is_signer,
        TokenSaleError::SaleAuthorityNotSigner,
        "buyer"
    );

    // 4. token_program
    //
    // - key must be the same as official

    // - key must be the same as official SPL Token Program ID
    let token_program = ctx.accounts.token_program;
    require!(
        // HOHOHOHO! No doppelganger programs here.
        *token_program.key == spl_token::ID,
        TokenSaleError::InvalidTokenProgramID,
        "token_program"
    );

    //---------- Data Validations (if any) ----------

    // Whitelist Gate
    match token_base.is_whitelisted(buyer.key, proof) {
        Ok(whitelisted) => {
            if !whitelisted {
                return Err(TokenSaleError::NotWhitelisted.into());
            }
        }
        Err(e) => return Err(e),
    }

    //---------- Executing Instruction ----------

    // build transfer instruction
    let payment_ix = system_instruction::transfer(buyer.key, &token_base.vault, token_base.price);

    // build mint_to instruction
    let buyer_pubkey = ctx.accounts.buyer.key;
    let buyer_account_pubkey = ctx.accounts.buyer_token_account.key;
    let mint_to_ix = instruction::mint_to(
        &spl_token::ID,
        &token_base.mint,
        buyer_account_pubkey,
        buyer_pubkey,
        &[buyer_pubkey],
        amount,
    )?;

    // invoke instructions

    // - Transfers SOL (lamports) from Buyer to Vault
    let vault = ctx.accounts.vault;
    invoke_signed(
        &payment_ix,
        &[buyer.clone(), vault.clone()],
        // signer seeds = PDA seeds
        &[&["token_base".as_bytes(), token_base.mint.as_ref()]],
    )?;

    // - Mints Token to Buyer account
    let mint = ctx.accounts.mint;
    // mint_authority == sale_authority
    let mint_authority = ctx.accounts.sale_authority;
    invoke_signed(
        &mint_to_ix,
        &[mint.clone(), buyer.clone(), mint_authority.clone()],
        // signer seeds = PDA seeds
        &[&["token_base".as_bytes(), token_base.mint.as_ref()]],
    )?;

    // - Creates Buyer Facts
    let buyer_facts_data = ctx.accounts.buyer_facts.try_borrow_mut_data()?;
    let mut buyer_facts = BuyerFacts::try_from_slice(&buyer_facts_data)?;

    buyer_facts.token_account = *ctx.accounts.buyer_token_account.key;
    buyer_facts.purchase_limit = token_base.default_purchase_limit;
    buyer_facts.bump = bump;

    Ok(())
}
