/// Onchain Merkle Tree utils
use borsh::{BorshDeserialize, BorshSerialize};
use merkletreers::{
    node::{Node, Side},
    {merkle_proof_check::merkle_proof_check, Leaf, Proof, Root},
};
use solana_program::{hash::hash, pubkey::Pubkey};

/// borsh de/serializable Merkle Proof primitive
pub type WhitelistProof = Vec<WhitelistNode>;

/// borsh de/serializable Merkle Side primitive
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum WhitelistSide {
    LEFT,
    RIGHT,
}

/// borsh de/serializable Merkle Node primitive
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct WhitelistNode {
    data: [u8; 32],
    side: WhitelistSide,
}

/// borsh de/serializable Merkle Root primitive
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct WhitelistRoot(pub Root);

/// Verify membership
pub fn verify_membership(root: Root, proof: Proof, member: Leaf) -> bool {
    root == merkle_proof_check(proof, member)
}

/// Converts a Solana Pubkey into a Merkle Tree Leaf
pub fn pubkey_to_sha256_leaf(pubkey: &Pubkey) -> Leaf {
    let decoded = hash(pubkey.as_ref());
    decoded.to_bytes()
}

/// Converts WhitelistProof into merkletreers::Proof
pub fn convert_whitelist_proof(w_proof: WhitelistProof) -> Proof {
    let mut merkle_proof = Proof::default();
    for w_node in w_proof {
        // default value
        let mut n = Node {
            data: [0u8; 32],
            side: Side::LEFT,
        };

        n.data = w_node.data;

        n.side = match w_node.side {
            WhitelistSide::LEFT => Side::LEFT,
            WhitelistSide::RIGHT => Side::RIGHT,
        };

        merkle_proof.push(n);
    }

    merkle_proof
}
