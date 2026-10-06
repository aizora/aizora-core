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

/// Простой Merkle-root из списка транзакций: хешируем каждую транзакцию,
/// потом попарно хешируем результаты, пока не останется один хеш.
pub fn merkle_root(txs: &[Vec<u8>]) -> [u8; 32] {
    if txs.is_empty() {
        // Если транзакций нет — возвращаем нулевой корень (как раньше)
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

    // Попарное хеширование до одного корня
    while leaves.len() > 1 {
        let mut next_level = Vec::with_capacity((leaves.len() + 1) / 2);
        for i in (0..leaves.len()).step_by(2) {
            if i + 1 < leaves.len() {
                // Пары: хешируем A || B
                let mut hasher = Sha256::new();
                hasher.update(&leaves[i]);
                hasher.update(&leaves[i + 1]);
                next_level.push(hasher.finalize().into());
            } else {
                // Непарный элемент: дублируем его, чтобы дерево было полным
                next_level.push(leaves[i]);
            }
        }
        leaves = next_level;
    }

    leaves[0]
}
