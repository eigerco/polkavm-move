use ed25519_dalek::{Signature, SigningKey, VerifyingKey};

pub(crate) fn ed25519_public_key_validate(bytes: &[u8]) -> bool {
    if bytes.len() != 32 {
        return false;
    }
    let Ok(bytes_arr) = <[u8; 32]>::try_from(bytes) else {
        return false;
    };
    VerifyingKey::from_bytes(&bytes_arr).is_ok()
}

pub(crate) fn ed25519_signature_verify_strict(sig: &[u8], pk: &[u8], msg: &[u8]) -> bool {
    let Ok(signature) = Signature::from_slice(sig) else {
        return false;
    };
    let Ok(pk_bytes) = <[u8; 32]>::try_from(pk) else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pk_bytes) else {
        return false;
    };
    verifying_key.verify_strict(msg, &signature).is_ok()
}

pub(crate) fn ed25519_generate_keys() -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let verifying_key = signing_key.verifying_key();
    (
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    )
}

pub(crate) fn ed25519_sign(sk: &[u8], msg: &[u8]) -> Vec<u8> {
    let Ok(sk_bytes) = <[u8; 32]>::try_from(sk) else {
        return vec![];
    };
    let signing_key = SigningKey::from_bytes(&sk_bytes);
    use ed25519_dalek::Signer;
    let signature = signing_key.sign(msg);
    signature.to_bytes().to_vec()
}

// --- Multi-Ed25519 ---

const MAX_MULTI_ED25519_KEYS: usize = 32;

pub(crate) fn multi_ed25519_public_key_validate(bytes: &[u8]) -> bool {
    // Format: N * 32 bytes (sub-keys) + 1 byte (threshold)
    if bytes.is_empty() || !(bytes.len() - 1).is_multiple_of(32) {
        return false;
    }
    let n = (bytes.len() - 1) / 32;
    if n == 0 || n > MAX_MULTI_ED25519_KEYS {
        return false;
    }
    // Validate each sub-key
    for i in 0..n {
        let start = i * 32;
        let Ok(key_bytes) = <[u8; 32]>::try_from(&bytes[start..start + 32]) else {
            return false;
        };
        if VerifyingKey::from_bytes(&key_bytes).is_err() {
            return false;
        }
    }
    true
}

pub(crate) fn multi_ed25519_public_key_validate_v2(bytes: &[u8]) -> bool {
    // Same as validate + threshold range check
    if bytes.is_empty() || !(bytes.len() - 1).is_multiple_of(32) {
        return false;
    }
    let n = (bytes.len() - 1) / 32;
    if n == 0 || n > MAX_MULTI_ED25519_KEYS {
        return false;
    }
    let threshold = bytes[bytes.len() - 1] as usize;
    if threshold == 0 || threshold > n {
        return false;
    }
    for i in 0..n {
        let start = i * 32;
        let Ok(key_bytes) = <[u8; 32]>::try_from(&bytes[start..start + 32]) else {
            return false;
        };
        if VerifyingKey::from_bytes(&key_bytes).is_err() {
            return false;
        }
    }
    true
}

pub(crate) fn multi_ed25519_signature_verify_strict(sig: &[u8], pk: &[u8], msg: &[u8]) -> bool {
    // PK format: N * 32 bytes + 1 byte (threshold)
    if pk.is_empty() || !(pk.len() - 1).is_multiple_of(32) {
        return false;
    }
    let n = (pk.len() - 1) / 32;
    if n == 0 || n > MAX_MULTI_ED25519_KEYS {
        return false;
    }
    let threshold = pk[pk.len() - 1] as usize;
    if threshold == 0 || threshold > n {
        return false;
    }

    // Sig format: K * 64 bytes + 4 bytes (bitmap)
    if sig.len() < 4 || !(sig.len() - 4).is_multiple_of(64) {
        return false;
    }
    let k = (sig.len() - 4) / 64;
    if k < threshold || k > n {
        return false;
    }

    // Parse bitmap (last 4 bytes, big-endian)
    let bitmap_bytes = &sig[sig.len() - 4..];
    let bitmap = u32::from_be_bytes([
        bitmap_bytes[0],
        bitmap_bytes[1],
        bitmap_bytes[2],
        bitmap_bytes[3],
    ]);

    // Count set bits in bitmap
    let set_bits = bitmap.count_ones() as usize;
    if set_bits != k {
        return false;
    }

    // Verify each sub-signature against the corresponding sub-key
    let mut sig_idx = 0;
    for key_idx in 0..n {
        if bitmap & (1u32 << (31 - key_idx)) == 0 {
            continue;
        }
        let sig_start = sig_idx * 64;
        let key_start = key_idx * 32;
        let Ok(signature) = Signature::from_slice(&sig[sig_start..sig_start + 64]) else {
            return false;
        };
        let Ok(key_bytes) = <[u8; 32]>::try_from(&pk[key_start..key_start + 32]) else {
            return false;
        };
        let Ok(verifying_key) = VerifyingKey::from_bytes(&key_bytes) else {
            return false;
        };
        if verifying_key.verify_strict(msg, &signature).is_err() {
            return false;
        }
        sig_idx += 1;
    }

    sig_idx == k
}

pub(crate) fn multi_ed25519_sign(sk: &[u8], msg: &[u8]) -> Vec<u8> {
    // Sign with first sub-key (32 bytes), return sig (64 bytes) + bitmap (4 bytes, bit 0 set)
    let Ok(sk_bytes) = <[u8; 32]>::try_from(sk) else {
        return vec![];
    };
    let signing_key = SigningKey::from_bytes(&sk_bytes);
    use ed25519_dalek::Signer;
    let signature = signing_key.sign(msg);
    let mut result = signature.to_bytes().to_vec();
    // Bitmap: bit 0 set (big-endian) = 0x80000000
    result.extend_from_slice(&[0x80, 0x00, 0x00, 0x00]);
    result
}

// --- BLS12-381 ---

const BLS_SIG_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
const BLS_POP_DST: &[u8] = b"BLS_POP_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

pub(crate) fn bls12381_validate_pubkey(bytes: &[u8]) -> bool {
    if bytes.len() != 48 {
        return false;
    }
    // Deserialize and validate (subgroup check)
    let pk = blst::min_pk::PublicKey::from_bytes(bytes);
    match pk {
        Ok(pk) => {
            // key_validate checks: not identity, on curve, in prime-order subgroup
            pk.validate().is_ok()
        }
        Err(_) => false,
    }
}

pub(crate) fn bls12381_signature_subgroup_check(bytes: &[u8]) -> bool {
    if bytes.len() != 96 {
        return false;
    }
    let sig = blst::min_pk::Signature::from_bytes(bytes);
    match sig {
        Ok(sig) => sig.validate(false).is_ok(),
        Err(_) => false,
    }
}

pub(crate) fn bls12381_verify_normal_signature(sig: &[u8], pk: &[u8], msg: &[u8]) -> bool {
    let Ok(signature) = blst::min_pk::Signature::from_bytes(sig) else {
        return false;
    };
    let Ok(public_key) = blst::min_pk::PublicKey::from_bytes(pk) else {
        return false;
    };
    // verify with pk subgroup check
    let result = signature.verify(true, msg, BLS_SIG_DST, &[], &public_key, true);
    result == blst::BLST_ERROR::BLST_SUCCESS
}

pub(crate) fn bls12381_verify_multisignature(sig: &[u8], pk: &[u8], msg: &[u8]) -> bool {
    let Ok(signature) = blst::min_pk::Signature::from_bytes(sig) else {
        return false;
    };
    let Ok(public_key) = blst::min_pk::PublicKey::from_bytes(pk) else {
        return false;
    };
    // verify without pk subgroup check (aggregated key assumed already validated)
    let result = signature.verify(false, msg, BLS_SIG_DST, &[], &public_key, true);
    result == blst::BLST_ERROR::BLST_SUCCESS
}

pub(crate) fn bls12381_verify_proof_of_possession(pk: &[u8], pop: &[u8]) -> bool {
    let Ok(public_key) = blst::min_pk::PublicKey::from_bytes(pk) else {
        return false;
    };
    // Validate pk first (must be valid)
    if public_key.validate().is_err() {
        return false;
    }
    let Ok(signature) = blst::min_pk::Signature::from_bytes(pop) else {
        return false;
    };
    // Verify PoP: sign(sk, pk_bytes) with PoP DST
    let result = signature.verify(false, pk, BLS_POP_DST, &[], &public_key, true);
    result == blst::BLST_ERROR::BLST_SUCCESS
}

pub(crate) fn bls12381_verify_signature_share(sig: &[u8], pk: &[u8], msg: &[u8]) -> bool {
    // Same as multisignature (no pk subgroup check)
    bls12381_verify_multisignature(sig, pk, msg)
}

pub(crate) fn bls12381_sign(sk: &[u8], msg: &[u8]) -> Vec<u8> {
    if sk.len() != 32 {
        return vec![];
    }
    let secret_key = blst::min_pk::SecretKey::from_bytes(sk);
    let Ok(secret_key) = secret_key else {
        return vec![];
    };
    let signature = secret_key.sign(msg, BLS_SIG_DST, &[]);
    signature.to_bytes().to_vec()
}

pub(crate) fn bls12381_generate_proof_of_possession(sk: &[u8]) -> Vec<u8> {
    if sk.len() != 32 {
        return vec![];
    }
    let Ok(secret_key) = blst::min_pk::SecretKey::from_bytes(sk) else {
        return vec![];
    };
    // PoP = sign(sk, pk_bytes) with PoP DST
    let pk = secret_key.sk_to_pk();
    let pk_bytes = pk.to_bytes();
    let signature = secret_key.sign(&pk_bytes, BLS_POP_DST, &[]);
    signature.to_bytes().to_vec()
}

pub(crate) fn secp256k1_ecdsa_recover(msg: &[u8], recovery_id: u8, sig: &[u8]) -> (Vec<u8>, bool) {
    use k256::ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey};

    let Some(rec_id) = RecoveryId::from_byte(recovery_id) else {
        return (vec![0u8; 64], false);
    };

    let Ok(signature) = K256Signature::from_slice(sig) else {
        return (vec![0u8; 64], false);
    };

    match VerifyingKey::recover_from_prehash(msg, &signature, rec_id) {
        Ok(key) => {
            let encoded = key.to_encoded_point(false);
            // Uncompressed point is 65 bytes (0x04 prefix + 64 bytes)
            let uncompressed = encoded.as_bytes();
            if uncompressed.len() == 65 {
                (uncompressed[1..].to_vec(), true)
            } else {
                (vec![0u8; 64], false)
            }
        }
        Err(_) => (vec![0u8; 64], false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_sign_and_verify() {
        let (sk, pk) = ed25519_generate_keys();
        let msg = b"test message";
        let sig = ed25519_sign(&sk, msg);
        assert!(!sig.is_empty());
        assert!(ed25519_signature_verify_strict(&sig, &pk, msg));
    }

    #[test]
    fn test_ed25519_validate_and_verify() {
        // Use RFC 8032 TEST 1 secret key to generate a valid test vector
        let sk_bytes =
            hex::decode("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .unwrap();
        let signing_key = SigningKey::from_bytes(&sk_bytes.clone().try_into().unwrap());
        let pk_bytes = signing_key.verifying_key().to_bytes().to_vec();

        assert!(ed25519_public_key_validate(&pk_bytes));

        let msg = b"test message";
        let sig = ed25519_sign(&sk_bytes, msg);
        assert!(ed25519_signature_verify_strict(&sig, &pk_bytes, msg));
        // Wrong message should fail
        assert!(!ed25519_signature_verify_strict(&sig, &pk_bytes, b"wrong"));
    }

    #[test]
    fn test_multi_ed25519_sign_and_verify() {
        // Generate a single-key multi_ed25519 key (threshold=1, n=1)
        let (sk, pk) = ed25519_generate_keys();
        // Multi-ed25519 PK: pk_bytes || threshold_byte
        let mut multi_pk = pk.clone();
        multi_pk.push(1); // threshold = 1

        assert!(multi_ed25519_public_key_validate(&multi_pk));
        assert!(multi_ed25519_public_key_validate_v2(&multi_pk));

        let msg = b"test multi_ed25519 message";
        let multi_sig = multi_ed25519_sign(&sk, msg);

        assert!(multi_ed25519_signature_verify_strict(
            &multi_sig, &multi_pk, msg
        ));
        // Wrong message should fail
        assert!(!multi_ed25519_signature_verify_strict(
            &multi_sig, &multi_pk, b"wrong"
        ));
    }

    #[test]
    fn test_multi_ed25519_validate_rejects_bad_threshold() {
        let (_, pk) = ed25519_generate_keys();
        // threshold = 0 should fail v2
        let mut multi_pk = pk.clone();
        multi_pk.push(0);
        assert!(!multi_ed25519_public_key_validate_v2(&multi_pk));

        // threshold = 2 with only 1 key should fail v2
        let mut multi_pk2 = pk;
        multi_pk2.push(2);
        assert!(!multi_ed25519_public_key_validate_v2(&multi_pk2));
    }

    #[test]
    fn test_bls12381_sign_and_verify() {
        // Generate a BLS key
        let ikm = [42u8; 32];
        let sk = blst::min_pk::SecretKey::key_gen(&ikm, &[]).unwrap();
        let pk = sk.sk_to_pk();
        let sk_bytes = sk.to_bytes();
        let pk_bytes = pk.to_bytes();

        assert!(bls12381_validate_pubkey(&pk_bytes));

        let msg = b"test bls12381 message";
        let sig = bls12381_sign(&sk_bytes, msg);
        assert!(!sig.is_empty());
        assert!(bls12381_signature_subgroup_check(&sig));

        assert!(bls12381_verify_normal_signature(&sig, &pk_bytes, msg));
        assert!(bls12381_verify_multisignature(&sig, &pk_bytes, msg));
        assert!(bls12381_verify_signature_share(&sig, &pk_bytes, msg));

        // Wrong message should fail
        assert!(!bls12381_verify_normal_signature(&sig, &pk_bytes, b"wrong"));
    }

    #[test]
    fn test_bls12381_proof_of_possession() {
        let ikm = [99u8; 32];
        let sk = blst::min_pk::SecretKey::key_gen(&ikm, &[]).unwrap();
        let sk_bytes = sk.to_bytes();
        let pk = sk.sk_to_pk();
        let pk_bytes = pk.to_bytes();

        let pop = bls12381_generate_proof_of_possession(&sk_bytes);
        assert!(!pop.is_empty());
        assert!(bls12381_verify_proof_of_possession(&pk_bytes, &pop));

        // Wrong pk should fail
        let wrong_pk = [0u8; 48];
        assert!(!bls12381_verify_proof_of_possession(&wrong_pk, &pop));
    }

    #[test]
    fn test_secp256k1_generate_test_vector() {
        use k256::ecdsa::{signature::hazmat::PrehashSigner, SigningKey as K256SigningKey};
        use sha2::Digest;

        // Generate a deterministic key from a known seed
        let sk_bytes =
            hex::decode("4c0883a69102937d6231471b5dbb6204fe512961708279f78a9b89a0e5a58d38")
                .unwrap();
        let signing_key = K256SigningKey::from_slice(&sk_bytes).unwrap();
        let verifying_key = signing_key.verifying_key();
        let encoded = verifying_key.to_encoded_point(false);
        let pk_uncompressed = &encoded.as_bytes()[1..]; // strip 0x04 prefix

        // Create a message and hash it (ecdsa_recover expects pre-hashed message)
        let msg = b"test secp256k1 recovery";
        let msg_hash = sha2::Sha256::digest(msg);

        // Sign with recovery
        let (signature, recovery_id) = signing_key
            .sign_prehash_recoverable(msg_hash.as_slice())
            .unwrap();

        let sig_bytes = signature.to_bytes();
        let rec_id = recovery_id.to_byte();

        println!("msg_hash: {}", hex::encode(msg_hash));
        println!("sig: {}", hex::encode(sig_bytes));
        println!("recovery_id: {}", rec_id);
        println!("pk (64 bytes): {}", hex::encode(pk_uncompressed));

        // Verify recovery works
        let (recovered_pk, success) =
            secp256k1_ecdsa_recover(msg_hash.as_slice(), rec_id, sig_bytes.as_slice());
        assert!(success);
        assert_eq!(recovered_pk, pk_uncompressed);
    }
}
