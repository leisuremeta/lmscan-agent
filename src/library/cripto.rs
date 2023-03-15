
// use hex_literal::hex;
use sha3::{Digest, Keccak256};

fn keccak_256(data: impl AsRef<[u8]>) -> Vec<u8> {
  let mut hasher = Keccak256::new();
  hasher.update(data);
  hasher.finalize().to_ascii_lowercase()
}