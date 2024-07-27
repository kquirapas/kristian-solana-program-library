use super::utils::TestHelper;
use crate::*;
use assert_matches::assert_matches;
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
async fn test_close_sale() {
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

    let instruction = crate::instruction::TokenSaleInstruction::CloseSale;

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
    let old_balance = ctx
        .banks_client
        .get_balance(ctx.payer.pubkey())
        .await
        .unwrap();

    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let new_balance = ctx
        .banks_client
        .get_balance(ctx.payer.pubkey())
        .await
        .unwrap();

    // confirm state
    let token_base = ctx
        .banks_client
        .get_account_data_with_borsh::<state::TokenBase>(token_base_pda)
        .await;

    // must be account not found
    assert_matches!(
        token_base,
        Err(BanksClientError::ClientError("Account not found"))
    );
    // REVIEW: possible have < 0 delta on balance?
    // must have more sol due to rent relinquished
    assert!(new_balance - old_balance > 0);
}
