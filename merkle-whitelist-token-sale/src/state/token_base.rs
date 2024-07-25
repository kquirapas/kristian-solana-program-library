use crate::merkle::WhitelistProof;
use crate::merkle::{convert_whitelist_proof, pubkey_to_sha256_leaf, verify_membership};
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};

// OPT-OUT: didn't use #[seeds()] because ShankAccount seeds
// helper attribute is buggy. PDA is generated offchain
// instead and seeds are validated on OpenSale

// TODO: Cache-line optimization (if I have time left)

#[repr(C)]
#[rustfmt::skip] // ensure manual struct ordering
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount, SplDiscriminate)]
#[discriminator_hash_input("token_sale::state::token_base")]
/// TokenBase holding the token sale configuraiton
pub struct TokenBase {
    /// Authority that can configure token sale after initialization
    pub sale_authority: Pubkey,
    /// Mint created external to this program
    pub mint: Pubkey,
    /// Account holding the SOL from token sale
    pub vault: Pubkey,
    /// Merkle root hash used to verify passed Merkle proof
    /// for whitelist gating
    pub whitelist_root: [u8; 32],
    /// Identifier for this specific structure
    pub discriminator: [u8; 8],
    /// Amount of lamports to transfer from Buyer to Vault 
    /// when purchasing tokens
    pub price: u64,
    /// Default purchase limit per user can be changed
    /// per wallet via AssignLimit
    pub default_purchase_limit: u64,
    /// Canonical bump for TokenBase PDA
    pub bump: u8,

    /// Padding to remove SLOP in C memory layout alignment
    /// Widest scalar = 32bytes
    _padding: [u8; 7]
}

impl TokenBase {
    /// Get known size of TokenBase
    pub const LEN: usize = std::mem::size_of::<TokenBase>();

    /// Is `true` if TokenBase is initialized
    pub fn is_initialized(&self) -> bool {
        self.discriminator.as_slice() == TokenBase::SPL_DISCRIMINATOR_SLICE
    }

    /// Is `true` if TokenBase is uninitialized
    pub fn is_uninitialized(&self) -> bool {
        self.discriminator.as_slice() == ArrayDiscriminator::UNINITIALIZED.as_slice()
    }

    /// Is `true` if buyer is in Merkle Tree whitelist.
    pub fn is_whitelisted(
        &self,
        buyer: &Pubkey,
        proof: WhitelistProof,
    ) -> Result<bool, ProgramError> {
        let member = pubkey_to_sha256_leaf(buyer);
        let merkle_proof = convert_whitelist_proof(proof);
        Ok(verify_membership(self.whitelist_root, merkle_proof, member))
    }
}
