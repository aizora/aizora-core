use sha2::{Digest, Sha256};

pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub capsule: [u8; 32],
}

impl BlockHeader {
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.prev_block_hash);
        hasher.update(&self.merkle_root);
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.bits.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.capsule);

        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }
}

pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Vec<u8>>,
}
