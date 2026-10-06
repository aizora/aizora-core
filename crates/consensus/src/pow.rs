use crate::block::{Block, BlockHeader, merkle_root};

pub fn compact_to_target(_bits: u32) -> [u8; 32] {
    [0xffu8; 32]
}

pub fn mine(header: &mut BlockHeader, target: [u8; 32]) -> u64 {
    loop {
        let hash = header.hash();
        if hash <= target {
            return header.nonce;
        }
        header.nonce += 1;
    }
}

pub fn create_next_block(prev_hash: [u8; 32], timestamp: u64, capsule: [u8; 32], txs: Vec<Vec<u8>>) -> Block {
    let root = merkle_root(&txs);

    let mut header = BlockHeader {
        version: 1,
        prev_block_hash: prev_hash,
        merkle_root: root,
        timestamp,
        bits: 0x1f00_0000,
        nonce: 0,
        capsule,
    };

    let target = compact_to_target(header.bits);
    mine(&mut header, target);

    Block {
        header,
        txs,
    }
}
