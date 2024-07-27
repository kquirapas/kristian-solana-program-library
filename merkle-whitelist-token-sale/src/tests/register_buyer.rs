use super::utils::TestHelper;
use crate::*;
use merkletreers::tree::MerkleTree;
use std::assert_eq;
use {
    solana_program_test::*,
    solana_sdk::{pubkey::Pubkey, signature::Keypair, signature::Signer},
    spl_discriminator::discriminator::ArrayDiscriminator,
};

/// Test Happy Path
#[tokio::test]
async fn test_register_buyer() {
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

    let (buyer_facts_pda, buyer_facts_canonical_bump) = TestHelper::initialize_buyer_facts(
        token_base_pda,
        ctx.payer.pubkey(),
        program_id,
        &mut ctx,
    )
    .await;

    // confirm state
    let buyer_facts = ctx
        .banks_client
        .get_account_data_with_borsh::<state::BuyerFacts>(buyer_facts_pda)
        .await
        .unwrap();

    // instruction went through
    assert_eq!(buyer_facts.purchase_limit, default_purchase_limit);
    assert_eq!(buyer_facts.bump, buyer_facts_canonical_bump);
    assert!(buyer_facts.discriminator != ArrayDiscriminator::UNINITIALIZED.as_slice());
}
