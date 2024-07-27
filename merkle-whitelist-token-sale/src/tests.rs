// #![cfg(feature = "test-sbf")]
#![cfg(test)]

use std::assert_eq;

use crate::*;
use borsh::{BorshDeserialize, BorshSerialize};
use merkletreers::{merkle_root, tree::MerkleTree};
use spl_token::{
    id, instruction,
    state::{Account, Mint},
    ID,
};
use {
    solana_program::hash::Hash,
    solana_program_test::*,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        message::Message,
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        signature::Keypair,
        signature::Signer,
        system_instruction,
        system_program::ID as SYSTEM_PROGRAM_ID,
        sysvar::rent::ID as RENT_SYSVAR_ID,
        transaction::Transaction,
    },
    spl_discriminator::discriminator::ArrayDiscriminator,
};

#[tokio::test]
async fn test_sanity() {
    assert_eq!(true, true)
}

#[tokio::test]
async fn test_open_sale() {
    let program_id = Pubkey::new_unique();
    let program_test = ProgramTest::new(
        // .so fixture is  retrieved from /target/deploy
        "merkle_whitelist_token_sale",
        program_id,
        // shank is incompatible with instantiating the BuiltInFunction
        None,
    );

    let mut ctx = program_test.start_with_context().await;

    // create Mint
    let mint = TestHelper::new_mint(0, &mut ctx).await;

    let vault = Keypair::new();
    let price: u64 = 100000000000;
    let default_purchase_limit: u64 = 100;
    let whitelist_root = crate::merkle::WhitelistRoot(MerkleTree::new(Vec::new()).root);

    // create TokenBase
    let (token_base_pda, token_base_canonical_bump) = TestHelper::initialize_token_base(
        price,
        default_purchase_limit,
        mint,
        vault.pubkey(),
        &whitelist_root,
        program_id,
        &mut ctx,
    )
    .await;

    // confirm state
    let token_base = ctx
        .banks_client
        .get_account_data_with_borsh::<state::TokenBase>(token_base_pda)
        .await
        .unwrap();

    // instruction went through
    assert_eq!(token_base.sale_authority, ctx.payer.pubkey());
    assert_eq!(token_base.mint, mint);
    assert_eq!(token_base.vault, vault.pubkey());
    assert_eq!(token_base.whitelist_root.0, whitelist_root.0);
    assert_eq!(token_base.price, price);
    assert_eq!(token_base.default_purchase_limit, default_purchase_limit);
    assert_eq!(token_base.bump, token_base_canonical_bump);
    assert!(!token_base.is_running);
    assert!(token_base.discriminator != ArrayDiscriminator::UNINITIALIZED.as_slice());
}

#[tokio::test]
async fn test_toggle_running() {
    let program_id = Pubkey::new_unique();
    let program_test = ProgramTest::new(
        // .so fixture is  retrieved from /target/deploy
        "merkle_whitelist_token_sale",
        program_id,
        // shank is incompatible with instantiating the BuiltInFunction
        None,
    );

    let mut ctx = program_test.start_with_context().await;

    // create Mint
    let mint = TestHelper::new_mint(0, &mut ctx).await;

    let vault = Keypair::new();
    let price: u64 = 100000000000;
    let default_purchase_limit: u64 = 100;
    let whitelist_root = crate::merkle::WhitelistRoot(MerkleTree::new(Vec::new()).root);

    // create TokenBase
    let (token_base_pda, _) = TestHelper::initialize_token_base(
        price,
        default_purchase_limit,
        mint,
        vault.pubkey(),
        &whitelist_root,
        program_id,
        &mut ctx,
    )
    .await;

    // ToggleRunning Instruction
    let instruction = crate::instruction::TokenSaleInstruction::ToggleRunning;

    let mut instruction_data = Vec::new();
    instruction.serialize(&mut instruction_data).unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(token_base_pda, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(ctx.payer.pubkey(), true),
            ],
            data: instruction_data.clone(),
        }],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer.insecure_clone()],
        ctx.last_blockhash,
    );

    // is_running: false -> true
    println!("TX 1");
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // confirm state
    let token_base = ctx
        .banks_client
        .get_account_data_with_borsh::<state::TokenBase>(token_base_pda)
        .await
        .unwrap();

    // instruction went through
    assert!(token_base.is_running);

    let new_blockhash = ctx.get_new_latest_blockhash().await.unwrap();

    // is_running: true -> false
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(token_base_pda, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(ctx.payer.pubkey(), true),
            ],
            data: instruction_data,
        }],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        new_blockhash,
    );

    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // refetch new state
    let token_base = ctx
        .banks_client
        .get_account_data_with_borsh::<state::TokenBase>(token_base_pda)
        .await
        .unwrap();

    assert!(!token_base.is_running);
}

//---------- CONSTRUCTORS ----------
pub struct TestHelper {}

impl TestHelper {
    async fn new_mint(decimals: u8, ctx: &mut ProgramTestContext) -> Pubkey {
        // create mint
        let mint = Keypair::new();
        let rent = ctx.banks_client.get_rent().await.unwrap();

        // Setup the mint
        let transaction = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &ctx.payer.pubkey(),
                    &mint.pubkey(),
                    rent.minimum_balance(Mint::LEN),
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_mint(
                    &spl_token::id(),
                    &mint.pubkey(),
                    &ctx.payer.pubkey(),
                    None,
                    decimals,
                )
                .unwrap(),
            ],
            Some(&ctx.payer.pubkey()),
            &[ctx.payer.insecure_clone(), mint.insecure_clone()],
            ctx.last_blockhash,
        );
        ctx.banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        mint.pubkey()
    }

    async fn initialize_token_base(
        price: u64,
        default_purchase_limit: u64,
        mint: Pubkey,
        vault: Pubkey,
        whitelist_root: &crate::merkle::WhitelistRoot,
        program_id: Pubkey,
        ctx: &mut ProgramTestContext,
    ) -> (Pubkey, u8) {
        // create token_base
        let (token_base_pda, token_base_canonical_bump) =
            pda::TokenBasePDA::find_pda(&program_id, &ctx.payer.pubkey(), &mint);

        let instruction = crate::instruction::TokenSaleInstruction::OpenSale {
            price,
            purchase_limit: default_purchase_limit,
            whitelist_root: whitelist_root.clone(),
        };

        let mut instruction_data = Vec::new();
        instruction.serialize(&mut instruction_data).unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(token_base_pda, false),
                    AccountMeta::new_readonly(mint, false),
                    AccountMeta::new_readonly(vault, false),
                    AccountMeta::new(ctx.payer.pubkey(), true),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ],
                data: instruction_data,
            }],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer.insecure_clone()],
            ctx.last_blockhash,
        );

        ctx.banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        (token_base_pda, token_base_canonical_bump)
    }
}
