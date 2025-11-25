pub(crate) enum Algorithm {
    Sha2_256,
    Sha3_256,
    SipHash,
    Keccak256,
    Sha2_512,
    Sha3_512,
    Blake2b256,
    Ripemd160,
}

pub(crate) fn hash(bytes: &[u8], algorithm: Algorithm) -> Vec<u8> {
    match algorithm {
        Algorithm::Sha2_256 => {
            use sha2::Digest;
            let output = sha2::Sha256::digest(bytes);
            output.to_vec()
        }
        Algorithm::Sha3_256 => {
            use sha3::Digest;
            let output = sha3::Sha3_256::digest(bytes);
            output.to_vec()
        }
        Algorithm::SipHash => {
            use siphasher::sip::SipHasher13;
            use std::hash::Hasher;
            let mut hasher = SipHasher13::new();
            hasher.write(bytes);
            let hash = hasher.finish();
            hash.to_le_bytes().to_vec()
        }
        Algorithm::Keccak256 => {
            use tiny_keccak::{Hasher as KeccakHasher, Keccak};
            let mut hasher = Keccak::v256();
            hasher.update(bytes);
            let mut output = [0u8; 32];
            hasher.finalize(&mut output);
            output.to_vec()
        }
        Algorithm::Sha2_512 => {
            use sha2::{Digest, Sha512};
            let output = Sha512::digest(bytes);
            output.to_vec()
        }
        Algorithm::Sha3_512 => {
            use sha3::{Digest, Sha3_512};
            let output = Sha3_512::digest(bytes);
            output.to_vec()
        }
        Algorithm::Blake2b256 => {
            let output = blake2_rfc::blake2b::blake2b(32, &[], bytes);
            output.as_bytes().to_vec()
        }
        Algorithm::Ripemd160 => {
            use ripemd::Digest;
            let mut hasher = ripemd::Ripemd160::new();
            hasher.update(bytes);
            hasher.finalize().to_vec()
        }
    }
}
