use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Vec<u8>>,
}

pub fn merkle_root(txs: &[Vec<u8>]) -> [u8; 32] {
    if txs.is_empty() {
        return [0u8; 32];
    }

    let mut leaves: Vec<[u8; 32]> = txs
        .iter()
        .map(|tx| {
            let mut hasher = Sha256::new();
            hasher.update(tx);
            hasher.finalize().into()
        })
        .collect();

    while leaves.len() > 1 {
        let mut next_level = Vec::with_capacity((leaves.len() + 1) / 2);
        for i in (0..leaves.len()).step_by(2) {
            if i + 1 < leaves.len() {
                let mut hasher = Sha256::new();
                hasher.update(&leaves[i]);
                hasher.update(&leaves[i + 1]);
                next_level.push(hasher.finalize().into());
            } else {
                next_level.push(leaves[i]);
            }
        }
        leaves = next_level;
    }

    leaves[0]
}

pub fn verify_merkle_root(block: &Block) -> bool {
    merkle_root(&block.txs) == block.header.merkle_root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_and_verify() {
        let txs = vec![b"Alice pays Bob 10 AZR".to_vec()];
        let root = merkle_root(&txs);

        let block = Block {
            header: BlockHeader {
                version: 1,
                prev_block_hash: [0u8; 32],
                merkle_root: root,
                timestamp: 0,
                bits: 0x1f00_0000,
                nonce: 0,
                capsule: [0u8; 32],
            },
            txs,
        };

        assert!(verify_merkle_root(&block));

        let mut bad_block = block.clone();
        bad_block.header.merkle_root = [0xffu8; 32];
        assert!(!verify_merkle_root(&bad_block));
    }
}
