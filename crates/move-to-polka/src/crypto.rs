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

pub(crate) fn secp256k1_ecdsa_recover(
    msg: &[u8],
    recovery_id: u8,
    sig: &[u8],
) -> (Vec<u8>, bool) {
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
}
