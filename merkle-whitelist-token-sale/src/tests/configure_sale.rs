use super::utils::TestHelper;
use crate::*;
use borsh::BorshSerialize;
use merkletreers::{tree::MerkleTree, Leaf};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signature::Signer,
    transaction::Transaction,
};

/// Test Happy Path
#[tokio::test]
async fn test_configure_sale() {
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
    let new_leaf: Leaf = Keypair::new().pubkey().to_bytes();
    let whitelist_root = crate::merkle::WhitelistRoot(MerkleTree::new(vec![new_leaf]).root);

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

    let new_price: u64 = 100000000000;
    let new_default_purchase_limit: u64 = 100;
    let new_leaf: Leaf = Keypair::new().pubkey().to_bytes();
    let new_whitelist_root = crate::merkle::WhitelistRoot(MerkleTree::new(vec![new_leaf]).root);

    let instruction = crate::instruction::TokenSaleInstruction::ConfigureSale {
        price: Some(new_price),
        default_purchase_limit: Some(new_default_purchase_limit),
        whitelist_root: Some(new_whitelist_root.clone()),
    };

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
    assert_eq!(token_base.price, new_price);
    assert_eq!(
        token_base.default_purchase_limit,
        new_default_purchase_limit
    );
    assert_eq!(token_base.whitelist_root.0, new_whitelist_root.0);
}

#[tokio::test]
async fn test_configure_price_only() {
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
    let new_leaf: Leaf = Keypair::new().pubkey().to_bytes();
    let whitelist_root = crate::merkle::WhitelistRoot(MerkleTree::new(vec![new_leaf]).root);

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

    let new_price: u64 = 100000000000;

    let instruction = crate::instruction::TokenSaleInstruction::ConfigureSale {
        price: Some(new_price),
        default_purchase_limit: None,
        whitelist_root: None,
    };

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
    assert_eq!(token_base.price, new_price);
    assert_eq!(token_base.default_purchase_limit, default_purchase_limit);
    assert_eq!(token_base.whitelist_root.0, whitelist_root.0);
}

#[tokio::test]
async fn test_configure_dpl_only() {
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
    let new_leaf: Leaf = Keypair::new().pubkey().to_bytes();
    let whitelist_root = crate::merkle::WhitelistRoot(MerkleTree::new(vec![new_leaf]).root);

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

    let new_default_purchase_limit: u64 = 100;

    let instruction = crate::instruction::TokenSaleInstruction::ConfigureSale {
        price: None,
        default_purchase_limit: Some(new_default_purchase_limit),
        whitelist_root: None,
    };

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
    assert_eq!(token_base.price, price);
    assert_eq!(
        token_base.default_purchase_limit,
        new_default_purchase_limit
    );
    assert_eq!(token_base.whitelist_root.0, whitelist_root.0);
}

#[tokio::test]
async fn test_configure_root_only() {
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
    let new_leaf: Leaf = Keypair::new().pubkey().to_bytes();
    let whitelist_root = crate::merkle::WhitelistRoot(MerkleTree::new(vec![new_leaf]).root);

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

    let new_whitelist_root = crate::merkle::WhitelistRoot(MerkleTree::new(vec![new_leaf]).root);

    let instruction = crate::instruction::TokenSaleInstruction::ConfigureSale {
        price: None,
        default_purchase_limit: None,
        whitelist_root: Some(new_whitelist_root.clone()),
    };

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
    assert_eq!(token_base.price, price);
    assert_eq!(token_base.default_purchase_limit, default_purchase_limit);
    assert_eq!(token_base.whitelist_root.0, new_whitelist_root.0);
}
