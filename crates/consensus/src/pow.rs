use crate::block::BlockHeader;

pub fn compact_to_target(bits: u32) -> [u8; 32] {
    let exponent = (bits >> 24) as usize;
    let mantissa = (bits & 0x00ff_ffff).to_be_bytes();

    if exponent == 0 {
        return [0u8; 32];
    }
    if exponent > 32 {
        return [0xffu8; 32];
    }

    let mut target = [0u8; 32];
    for i in 0..3 {
        let pos = 32 - exponent + i;
        if pos < 32 {
            target[pos] = mantissa[i + 1];
        }
    }
    target
}

pub fn mine(header: &mut BlockHeader, target: [u8; 32]) -> u64 {
    loop {
        let hash = header.hash();
        if hash.as_slice() <= target.as_slice() {
            return header.nonce;
        }
        header.nonce += 1;
    }
}
