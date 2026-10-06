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
                // Непарный элемент: дублируем, чтобы дерево было полным
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

/// Проверяет всю цепочку блоков:
/// 1. Merkle-корень каждого блока совпадает с его транзакциями.
/// 2. prev_block_hash каждого блока (начиная со второго) совпадает с хешем предыдущего блока.
pub fn verify_chain(chain: &[Block]) -> bool {
    if chain.is_empty() {
        return true; // Пустая цепочка — валидна по определению
    }

    // Проверяем первый блок: только Merkle-корень
    if !verify_merkle_root(&chain[0]) {
        return false;
    }

    for i in 1..chain.len() {
        let current = &chain[i];
        let prev = &chain[i - 1];

        // 1. Проверяем Merkle-корень текущего блока
        if !verify_merkle_root(current) {
            return false;
        }

        // 2. Проверяем, что prev_block_hash текущего блока равен хешу предыдущего
        let prev_hash = prev.header.hash();
        if current.header.prev_block_hash != prev_hash {
            return false;
        }
    }

    true
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

    #[test]
    fn test_verify_chain_valid() {
        // Генерируем валидную цепочку из 3 блоков
        let genesis_txs = vec![b"Genesis tx".to_vec()];
        let genesis_root = merkle_root(&genesis_txs);
        let genesis = Block {
            header: BlockHeader {
                version: 1,
                prev_block_hash: [0u8; 32],
                merkle_root: genesis_root,
                timestamp: 1000,
                bits: 0x1f00_0000,
                nonce: 0,
                capsule: [0u8; 32],
            },
            txs: genesis_txs,
        };

        let second_txs = vec![b"Second tx".to_vec()];
        let second_root = merkle_root(&second_txs);
        let second = Block {
            header: BlockHeader {
                version: 1,
                prev_block_hash: genesis.header.hash(),
                merkle_root: second_root,
                timestamp: 2000,
                bits: 0x1f00_0000,
                nonce: 0,
                capsule: [0u8; 32],
            },
            txs: second_txs,
        };

        let third_txs = vec![b"Third tx".to_vec()];
        let third_root = merkle_root(&third_txs);
        let third = Block {
            header: BlockHeader {
                version: 1,
                prev_block_hash: second.header.hash(),
                merkle_root: third_root,
                timestamp: 3000,
                bits: 0x1f00_0000,
                nonce: 0,
                capsule: [0u8; 32],
            },
            txs: third_txs,
        };

        let chain = vec![genesis, second, third];
        assert!(verify_chain(&chain), "Валидная цепочка должна пройти проверку");
    }

    #[test]
    fn test_verify_chain_broken_link() {
        // Цепочка с «разрывом»: второй блок ссылается не на хеш первого
        let genesis_txs = vec![b"Genesis tx".to_vec()];
        let genesis_root = merkle_root(&genesis_txs);
        let genesis = Block {
            header: BlockHeader {
                version: 1,
                prev_block_hash: [0u8; 32],
                merkle_root: genesis_root,
                timestamp: 1000,
                bits: 0x1f00_0000,
                nonce: 0,
                capsule: [0u8; 32],
            },
            txs: genesis_txs,
        };

        let second_txs = vec![b"Second tx".to_vec()];
        let second_root = merkle_root(&second_txs);
        let second = Block {
            header: BlockHeader {
                version: 1,
                // Специально ломаем ссылку: ставим мусор вместо хеша первого блока
                prev_block_hash: [0xffu8; 32],
                merkle_root: second_root,
                timestamp: 2000,
                bits: 0x1f00_0000,
                nonce: 0,
                capsule: [0u8; 32],
            },
            txs: second_txs,
        };

        let chain = vec![genesis, second];
        assert!(!verify_chain(&chain), "Цепочка с разорванной ссылкой должна быть отклонена");
    }

    #[test]
    fn test_verify_chain_broken_merkle() {
        // Цепочка, где у второго блока подменён Merkle-корень
        let genesis_txs = vec![b"Genesis tx".to_vec()];
        let genesis_root = merkle_root(&genesis_txs);
        let genesis = Block {
            header: BlockHeader {
                version: 1,
                prev_block_hash: [0u8; 32],
                merkle_root: genesis_root,
                timestamp: 1000,
                bits: 0x1f00_0000,
                nonce: 0,
                capsule: [0u8; 32],
            },
            txs: genesis_txs,
        };

        let second_txs = vec![b"Second tx".to_vec()];
        let second_root = merkle_root(&second_txs);
        let second = Block {
            header: BlockHeader {
                version: 1,
                prev_block_hash: genesis.header.hash(),
                // Подменяем Merkle-корень на мусор
                merkle_root: [0xaau8; 32],
                timestamp: 2000,
                bits: 0x1f00_0000,
                nonce: 0,
                capsule: [0u8; 32],
            },
            txs: second_txs,
        };

        let chain = vec![genesis, second];
        assert!(!verify_chain(&chain), "Цепочка с неверным Merkle-корнем должна быть отклонена");
    }
}
