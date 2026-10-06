pub mod block;
pub mod pow;

#[cfg(test)]
mod tests {
    use super::block::BlockHeader;
    use super::pow::{compact_to_target, create_next_block, mine};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn mine_genesis_block() {
        let mut genesis = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1f00_0000,
            nonce: 0,
            capsule: [0u8; 32], // обязательно добавляем капсулу
        };

        let target = compact_to_target(genesis.bits);
        mine(&mut genesis, target);
        let hash = genesis.hash();

        assert!(hash <= target);
        println!("⛏ GENESIS MINED! nonce = {}, hash = {}", genesis.nonce, hex(&hash));
    }

    #[test]
    fn mine_second_block() {
        // Создаём генезис (для этого теста)
        let mut genesis = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1f00_0000,
            nonce: 0,
            capsule: [0u8; 32],
        };

        let genesis_target = compact_to_target(genesis.bits);
        mine(&mut genesis, genesis_target);
        let genesis_hash = genesis.hash();

        println!("🪨 Genesis block mined. Hash: {}", hex(&genesis_hash));

        // Готовим капсулу: сообщение в блокчейне
        let message = b"Aizora forever!";
        let mut capsule = [0u8; 32];
        capsule[..message.len()].copy_from_slice(message);

        let second_block = create_next_block(genesis_hash, genesis.timestamp + 10, capsule);

        assert_eq!(second_block.header.prev_block_hash, genesis_hash);

        let second_target = compact_to_target(second_block.header.bits);
        assert!(second_block.header.hash() <= second_target);

        println!("⛓️ Second block mined!");
        println!("Capsule message: \"{}\"", String::from_utf8_lossy(&capsule));
        println!("Block hash: {}", hex(&second_block.header.hash()));
    }
}
