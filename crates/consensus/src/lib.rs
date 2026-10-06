pub mod block;
pub mod pow;

#[cfg(test)]
mod tests {
    use crate::block::BlockHeader;
    use crate::pow::{compact_to_target, mine};
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
            bits: 0x01ff_ffff,
            nonce: 0,
        };

        let target = compact_to_target(genesis.bits);
        let nonce = mine(&mut genesis, target);
        let hash = genesis.hash();

        assert!(hash.as_slice() <= target.as_slice());
        println!("⛏ GENESIS MINED! nonce = {}, hash = {}", nonce, hex(&hash));
    }
}
