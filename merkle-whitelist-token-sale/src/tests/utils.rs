pub struct TestHelper {}

use crate::*;
use borsh::BorshSerialize;
use spl_token::state::Mint;
use {
    solana_program_test::*,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        signature::Keypair,
        signature::Signer,
        system_instruction,
        system_program::ID as SYSTEM_PROGRAM_ID,
        transaction::Transaction,
    },
};

impl TestHelper {
    pub async fn new_mint(decimals: u8, ctx: &mut ProgramTestContext) -> Pubkey {
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

    pub async fn initialize_token_base(
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

    pub async fn initialize_buyer_facts(
        token_base: Pubkey,
        buyer: Pubkey,
        program_id: Pubkey,
        ctx: &mut ProgramTestContext,
    ) -> (Pubkey, u8) {
        // create buyer_facts
        let (buyer_facts_pda, buyer_facts_canonical_bump) =
            pda::BuyerFactsPDA::find_pda(&program_id, &token_base, &buyer);

        let instruction = crate::instruction::TokenSaleInstruction::RegisterBuyer;

        let mut instruction_data = Vec::new();
        instruction.serialize(&mut instruction_data).unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new_readonly(token_base, false),
                    AccountMeta::new(buyer_facts_pda, false),
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

        (buyer_facts_pda, buyer_facts_canonical_bump)
    }
}
