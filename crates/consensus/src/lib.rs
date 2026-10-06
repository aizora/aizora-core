pub mod block;
pub mod pow;

#[cfg(test)]
mod tests {
    use super::block::{Block, BlockHeader, verify_merkle_root, verify_chain};
    use super::pow::{compact_to_target, create_next_block, mine};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn mine_genesis_block() {
        let txs = vec![b"Alice pays Bob 10 AZR".to_vec()];
        let root = super::block::merkle_root(&txs);

        let mut genesis = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: root,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1f00_0000,
            nonce: 0,
            capsule: [0u8; 32],
        };

        let target = compact_to_target(genesis.bits);
        mine(&mut genesis, target);
        let hash = genesis.hash();

        assert!(hash <= target);

        let genesis_block = Block {
            header: genesis,
            txs,
        };
        assert!(verify_merkle_root(&genesis_block), "Merkle-корень генезиса не совпадает!");

        println!("⛏ GENESIS MINED! nonce = {}, hash = {}", genesis_block.header.nonce, hex(&hash));
        println!("Merkle root: {}", hex(&genesis_block.header.merkle_root));
    }

    #[test]
    fn mine_second_block() {
        let genesis_txs = vec![b"Alice pays Bob 10 AZR".to_vec()];
        let genesis_root = super::block::merkle_root(&genesis_txs);

        let mut genesis = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: genesis_root,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1f00_0000,
            nonce: 0,
            capsule: [0u8; 32],
        };

        let genesis_target = compact_to_target(genesis.bits);
        mine(&mut genesis, genesis_target);
        
        // ВАЖНО: сначала сохраняем то, что нам понадобится дальше
        let genesis_hash = genesis.hash();
        let genesis_timestamp = genesis.timestamp;

        println!("🪨 Genesis block mined. Hash: {}", hex(&genesis_hash));

        let second_txs = vec![
            b"Bob pays Carol 5 AZR".to_vec(),
            b"Carol pays Dave 2 AZR".to_vec(),
        ];

        let message = b"Aizora forever!";
        let mut capsule = [0u8; 32];
        capsule[..message.len()].copy_from_slice(message);

        // Используем сохранённые значения, а не перемещённый `genesis`
        let second_block = create_next_block(genesis_hash, genesis_timestamp + 10, capsule, second_txs);

        assert_eq!(second_block.header.prev_block_hash, genesis_hash);

        let second_target = compact_to_target(second_block.header.bits);
        assert!(second_block.header.hash() <= second_target);

        assert!(verify_merkle_root(&second_block), "Merkle-корень второго блока не совпадает!");

        println!("⛓️ Second block mined!");
        println!("Capsule message: \"{}\"", String::from_utf8_lossy(&capsule));
        println!("Block hash: {}", hex(&second_block.header.hash()));
        println!("Merkle root (2 txs): {}", hex(&second_block.header.merkle_root));
    }

    #[test]
    fn verify_full_chain() {
        // Собираем цепочку и проверяем её целиком через verify_chain
        let genesis_txs = vec![b"Alice pays Bob 10 AZR".to_vec()];
        let genesis_root = super::block::merkle_root(&genesis_txs);

        let mut genesis = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: genesis_root,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1f00_0000,
            nonce: 0,
            capsule: [0u8; 32],
        };

        let genesis_target = compact_to_target(genesis.bits);
        mine(&mut genesis, genesis_target);
        
        // Сначала сохраняем нужные поля
        let genesis_hash = genesis.hash();
        let genesis_timestamp = genesis.timestamp;
        
        let genesis_block = Block {
            header: genesis, // Здесь `genesis` уезжает, но мы уже всё сохранили
            txs: genesis_txs,
        };

        let second_txs = vec![
            b"Bob pays Carol 5 AZR".to_vec(),
            b"Carol pays Dave 2 AZR".to_vec(),
        ];
        let message = b"Aizora forever!";
        let mut capsule = [0u8; 32];
        capsule[..message.len()].copy_from_slice(message);

        // Используем сохранённые `genesis_hash` и `genesis_timestamp`
        let second_block = create_next_block(genesis_hash, genesis_timestamp + 10, capsule, second_txs);
        assert!(verify_merkle_root(&second_block));

        // ГЛАВНОЕ: собираем цепочку и проверяем целиком
        let chain = vec![genesis_block, second_block];
        assert!(verify_chain(&chain), "Цепочка должна быть валидной!");

        println!("✅ Full chain verified successfully!");
    }
}
