use crate::merkle::WhitelistRoot;
use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

/// TokenSale Instruction List
///
/// For Token Sale Authority:
/// - OpenSale (Initialize)
/// - ConfigureSale
/// - CloseSale
/// - ToggleRunning
/// - AssignLimit
///
/// For Buyer:
/// - RegisterBuyer (Initialize)
/// - BuyToken
/// - CloseFacts
///
#[derive(BorshDeserialize, BorshSerialize, Debug, ShankContext, ShankInstruction)]
pub enum TokenSaleInstruction {
    /// Open a Token Sale with the given config
    ///
    /// - Initializes the [`TokenBase`] PDA account (config)
    ///
    /// For Token Sale Authority
    #[account(
        0,
        writable,
        name = "token_base",
        desc = "Account (TokenBase PDA) holding token sale configuration. Seeds ['token_base',  `pubkey(sale_authority)`, `pubkey(mint)`]"
    )]
    #[account(
        1,
        name = "mint",
        desc = "Account for holding the mint details of the token being sold"
    )]
    #[account(
        2,
        name = "vault",
        desc = "Account for holding the funds raised from token sale"
    )]
    #[account(
        3,
        signer,
        name = "sale_authority",
        desc = "Account who has authority to manage the token sale"
    )]
    #[account(4, name = "system_program", desc = "System Program")]
    OpenSale {
        /// Price of token
        price: u64,
        /// Amount of tokens allowed per buyer wallet
        purchase_limit: u64,
        /// Merkle tree root of whitelist
        whitelist_root: WhitelistRoot,
    },

    /// Start/Pause Token Sale
    ///
    /// - Flips the value of [`TokenBase`] `.is_running` property
    ///
    /// For Token Sale Authority
    #[account(
        0,
        writable,
        name = "token_base",
        desc = "Account (TokenBase PDA) holding token sale configuration. Seeds ['token_base', `pubkey(sale_authority)`, `pubkey(mint)`]"
    )]
    #[account(
        1,
        name = "mint",
        desc = "Account for holding the mint details of the token being sold"
    )]
    #[account(
        2,
        signer,
        name = "sale_authority",
        desc = "Account who has authority to manage the token sale"
    )]
    ToggleRunning,

    /// Update your Token Sale configuration
    ///
    /// - Modifies one or more than from: price, default_purchase_limit, whitelist_root
    /// of [`TokenBase`]
    ///
    /// For Token Sale Authority
    #[account(
        0,
        writable,
        name = "token_base",
        desc = "Account (TokenBase PDA) holding token sale configuration. Seeds ['token_base', `pubkey(sale_authority)`, `pubkey(mint)`]"
    )]
    #[account(
        1,
        name = "mint",
        desc = "Account for holding the mint details of the token being sold"
    )]
    #[account(
        2,
        signer,
        name = "sale_authority",
        desc = "Account who has authority to manage the token sale"
    )]
    ConfigureSale {
        price: Option<u64>,
        default_purchase_limit: Option<u64>,
        whitelist_root: Option<WhitelistRoot>,
    },

    /// Close the token sale
    ///
    /// - Closes the [`TokenBase`] account
    /// - Relinquishes rent lamports
    ///
    /// For Token Sale Authority
    #[account(
        0,
        writable,
        name = "token_base",
        desc = "Account (TokenBase PDA) holding token sale configuration. Seeds ['token_base', pubkey(sale_authority), pubkey(mint)]"
    )]
    #[account(
        1,
        name = "mint",
        desc = "Account for holding the mint details of the token being sold"
    )]
    #[account(
        2,
        signer,
        name = "sale_authority",
        desc = "Account who has authority to manage the token sale"
    )]
    CloseSale,

    /// Assign a user's purchase limit
    ///
    /// - Changes the `purchase_limit` of a certain buyer's
    /// BuyerFacts
    ///
    /// For Token Sale Authority
    #[account(
        0,
        name = "token_base",
        desc = "Account (BuyerFacts PDA) holding a buyer's configuration. Seeds ['token_base', `pubkey(sale_authority)`, `pubkey(mint)`]"
    )]
    #[account(
        1,
        writable,
        name = "buyer_facts",
        desc = "Account (BuyerFacts PDA) holding a buyer's configuration. Seeds ['token_base', `pubkey(sale_authority)`, `pubkey(mint)`]"
    )]
    #[account(
        2,
        name = "buyer",
        desc = "Account who owns the BuyerFacts PDA to be assigned a new purchase limit to"
    )]
    #[account(
        3,
        signer,
        name = "sale_authority",
        desc = "Account who has authority to manage the token sale"
    )]
    AssignLimit { new_purchase_limit: u64 },

    /// Register as a Buyer
    ///
    /// - Generates the buyer's BuyerFacts
    ///
    /// For Buyer
    #[account(
        0,
        name = "token_base",
        desc = "Account (BuyerFacts PDA) holding a buyer's configuration. Seeds ['token_base', `pubkey(sale_authority)`, `pubkey(mint)`]"
    )]
    #[account(
        1,
        writable,
        name = "buyer_facts",
        desc = "Account (BuyerFacts PDA) holding a buyer's configuration. Seeds ['token_base', `pubkey(sale_authority)`, `pubkey(mint)`]"
    )]
    #[account(
        2,
        signer,
        name = "buyer",
        desc = "Account who owns the BuyerFacts PDA to be assigned a new purchase limit to"
    )]
    #[account(3, name = "system_program", desc = "System_program")]
    RegisterBuyer,
    // /// Buy N amount of Tokens
    // ///
    // /// - Initializes Associated Token Account for Buyer
    // /// - Transfers SOL (lamports) from Buyer to Vault
    // /// - Mints Token to Buyer account
    // ///
    // /// For Buyers
    // #[account(
    //     0,
    //     name = "token_base",
    //     desc = "Account (TokenBase PDA) holding token sale configuration. Seeds ['token_base', `token_base::mint`]"
    // )]
    // #[account(
    //     1,
    //     name = "mint",
    //     desc = "Account for holding the mint details of the token being sold"
    // )]
    // #[account(
    //     2,
    //     writable,
    //     name = "vault",
    //     desc = "Account for holding the funds raised from token sale"
    // )]
    // #[account(
    //     3,
    //     name = "sale_authority",
    //     desc = "Account who has authority to manage the token sale"
    // )]
    // #[account(
    //     4,
    //     name = "buyer_token_account",
    //     desc = "Account owned by the buyer where newly bought tokens get transferred to"
    // )]
    // #[account(
    //     5,
    //     name = "buyer_facts",
    //     desc = "Account (BuyerFacts PDA) holding user specific statistics. Seeds ['buyer_facts', `pubkey(buyer)`, `pubkey(mint)`]"
    // )]
    // #[account(
    //     6,
    //     signer,
    //     name = "buyer",
    //     desc = "Account who is buying from token sale and will pay for the fees"
    // )]
    // #[account(7, name = "token_program", desc = "Official SPL Token Program")]
    // BuyToken { amount: u64, proof: WhitelistProof },
}
