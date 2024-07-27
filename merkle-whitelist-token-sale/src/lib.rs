#![forbid(unsafe_code)]

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod macros;
pub mod merkle;
pub mod pda;
pub mod processor;
pub mod state;
pub mod wasm;

// make sure tests don't affect binary
// #[cfg(feature = "test-sbf")]
// mark as test
#[cfg(test)]
pub mod tests;

solana_program::declare_id!("Aq2EAZ8i8UgKGaGzpSPhfvGxf4hkziymA4WqXrJ4NYu4");
