#![forbid(unsafe_code)]
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod macros;
pub mod merkle;
pub mod pda;
pub mod processor;
pub mod state;
pub mod test;
pub mod wasm;

solana_program::declare_id!("Aq2EAZ8i8UgKGaGzpSPhfvGxf4hkziymA4WqXrJ4NYu4");
