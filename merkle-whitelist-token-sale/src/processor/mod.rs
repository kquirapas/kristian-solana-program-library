use crate::instruction::{accounts::*, TokenSaleInstruction};
use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub mod open_sale;
use open_sale::*;

pub mod toggle_running;
use toggle_running::*;

pub mod configure_sale;
use configure_sale::*;

pub mod close_sale;
use close_sale::*;

pub mod assign_limit;
use assign_limit::*;

pub mod register_buyer;
use register_buyer::*;

pub mod deregister_buyer;
use deregister_buyer::*;

// pub mod buy_token;
// use buy_token::*;

/// Program state processor
pub struct Processor {}

impl<'a> Processor {
    /// Process the transaction
    ///
    /// - Deserializes the instruction data
    /// - Routes transaction data to the proper handler
    pub fn process(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // get instruction
        let instruction = TokenSaleInstruction::try_from_slice(instruction_data)?;
        match instruction {
            TokenSaleInstruction::OpenSale {
                price,
                purchase_limit,
                whitelist_root,
            } => {
                process_open_sale(
                    program_id,
                    OpenSaleAccounts::context(accounts)?,
                    price,
                    purchase_limit,
                    whitelist_root,
                )?;
            }

            TokenSaleInstruction::ToggleRunning => {
                process_update_running(program_id, ToggleRunningAccounts::context(accounts)?)?;
            }

            TokenSaleInstruction::ConfigureSale {
                price,
                default_purchase_limit,
                whitelist_root,
            } => {
                process_configure_sale(
                    program_id,
                    ConfigureSaleAccounts::context(accounts)?,
                    price,
                    default_purchase_limit,
                    whitelist_root,
                )?;
            }

            TokenSaleInstruction::CloseSale => {
                process_close_sale(program_id, CloseSaleAccounts::context(accounts)?)?;
            }

            TokenSaleInstruction::AssignLimit { new_purchase_limit } => {
                process_assign_limit(
                    program_id,
                    AssignLimitAccounts::context(accounts)?,
                    new_purchase_limit,
                )?;
            }

            TokenSaleInstruction::RegisterBuyer => {
                process_register_buyer(program_id, RegisterBuyerAccounts::context(accounts)?)?;
            }

            TokenSaleInstruction::DeregisterBuyer => {
                process_deregister_buyer(program_id, DeregisterBuyerAccounts::context(accounts)?)?;
            }
        }

        Ok(())
    }
}
