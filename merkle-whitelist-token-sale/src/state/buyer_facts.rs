use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};

// OPT-OUT: didn't use #[seeds()] because ShankAccount seeds
// helper attribute is buggy. PDA is generated offchain
// instead and seeds are validated on OpenSale

// TODO: Cache-line optimization (if I have time left)

#[repr(C)]
#[rustfmt::skip] // ensure manual struct ordering
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount, SplDiscriminate)]
#[discriminator_hash_input("token_sale::state::buyer_facts")]
/// BuyerFacts holding per wallet buyer stats
pub struct BuyerFacts {
    /// Token account holding buyer's tokens
    pub token_account: Pubkey,
    /// Identifier for this specific structure
    pub discriminator: [u8; 8],
    /// Amount of tokens allowed for this specific buyer
    pub purchase_limit: u64,
    /// Canonical bump for BuyerFacts
    pub bump: u8,

    /// Padding to remove SLOP in C memory layout alignment
    /// Widest scalar = 32bytes
    _padding: [u8; 17]
}

impl BuyerFacts {
    /// Get known size of BuyerFacts
    pub const LEN: usize = std::mem::size_of::<BuyerFacts>();

    /// Is `true` if BuyerFacts is initialized
    pub fn is_initialized(&self) -> bool {
        self.discriminator.as_slice() == BuyerFacts::SPL_DISCRIMINATOR_SLICE
    }

    /// Is `true` if BuyerFacts is uninitialized
    pub fn is_uninitialized(&self) -> bool {
        self.discriminator.as_slice() == ArrayDiscriminator::UNINITIALIZED.as_slice()
    }
}
