use alloy_primitives::keccak256;

fn main() {
    let encoded = alloy_rlp::encode([]);
    let hash = keccak256(&encoded);
    println!("{:02x?}", encoded);
    println!("{:?}", hash);
}
