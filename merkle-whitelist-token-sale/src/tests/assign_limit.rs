use super::utils::TestHelper;
use crate::*;
use borsh::BorshSerialize;
use merkletreers::tree::MerkleTree;
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

    let buyer = Keypair::new();
    let (buyer_facts_pda, _) =
        TestHelper::initialize_buyer_facts(token_base_pda, buyer.pubkey(), program_id, &mut ctx)
            .await;

    let new_purchase_limit: u64 = 143;

    // ToggleRunning Instruction
    let instruction = crate::instruction::TokenSaleInstruction::AssignLimit { new_purchase_limit };

    let mut instruction_data = Vec::new();
    instruction.serialize(&mut instruction_data).unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(token_base_pda, false),
                AccountMeta::new(buyer_facts_pda, false),
                AccountMeta::new_readonly(buyer.pubkey(), false),
                AccountMeta::new(ctx.payer.pubkey(), true),
            ],
            data: instruction_data.clone(),
        }],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer.insecure_clone()],
        ctx.last_blockhash,
    );

    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // confirm state
    let buyer_facts = ctx
        .banks_client
        .get_account_data_with_borsh::<state::BuyerFacts>(buyer_facts_pda)
        .await
        .unwrap();

    assert_eq!(buyer_facts.purchase_limit, new_purchase_limit);
}
