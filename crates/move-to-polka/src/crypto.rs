use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::{Identity, VartimeMultiscalarMul},
};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use std::sync::{Arc, Mutex};

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

pub(crate) fn bls12381_aggregate_pubkeys(pubkeys: &[Vec<u8>]) -> (Vec<u8>, bool) {
    if pubkeys.is_empty() {
        return (vec![], false);
    }
    let mut agg = match blst::min_pk::PublicKey::from_bytes(&pubkeys[0]) {
        Ok(pk) => {
            if pk.validate().is_err() {
                return (vec![], false);
            }
            blst::min_pk::AggregatePublicKey::from_public_key(&pk)
        }
        Err(_) => return (vec![], false),
    };
    for pk_bytes in &pubkeys[1..] {
        let Ok(pk) = blst::min_pk::PublicKey::from_bytes(pk_bytes) else {
            return (vec![], false);
        };
        if pk.validate().is_err() {
            return (vec![], false);
        }
        agg.add_public_key(&pk, false).unwrap();
    }
    (agg.to_public_key().to_bytes().to_vec(), true)
}

pub(crate) fn bls12381_aggregate_signatures(sigs: &[Vec<u8>]) -> (Vec<u8>, bool) {
    if sigs.is_empty() {
        return (vec![], false);
    }
    let mut agg = match blst::min_pk::Signature::from_bytes(&sigs[0]) {
        Ok(sig) => blst::min_pk::AggregateSignature::from_signature(&sig),
        Err(_) => return (vec![], false),
    };
    for sig_bytes in &sigs[1..] {
        let Ok(sig) = blst::min_pk::Signature::from_bytes(sig_bytes) else {
            return (vec![], false);
        };
        agg.add_signature(&sig, false).unwrap();
    }
    (agg.to_signature().to_bytes().to_vec(), true)
}

pub(crate) fn bls12381_verify_aggregate_signature(
    aggsig: &[u8],
    pubkeys: &[Vec<u8>],
    messages: &[Vec<u8>],
) -> bool {
    if pubkeys.len() != messages.len() || pubkeys.is_empty() {
        return false;
    }
    let Ok(signature) = blst::min_pk::Signature::from_bytes(aggsig) else {
        return false;
    };
    let mut pks = Vec::with_capacity(pubkeys.len());
    for pk_bytes in pubkeys {
        let Ok(pk) = blst::min_pk::PublicKey::from_bytes(pk_bytes) else {
            return false;
        };
        pks.push(pk);
    }
    let pk_refs: Vec<&blst::min_pk::PublicKey> = pks.iter().collect();
    let msg_refs: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
    let result = signature.aggregate_verify(true, &msg_refs, BLS_SIG_DST, &pk_refs, true);
    result == blst::BLST_ERROR::BLST_SUCCESS
}

pub(crate) fn bls12381_generate_keys() -> (Vec<u8>, Vec<u8>) {
    let ikm: [u8; 32] = rand::random();
    let sk = blst::min_pk::SecretKey::key_gen(&ikm, &[]).expect("key_gen failed");
    let pk = sk.sk_to_pk();
    let pk_bytes = pk.to_bytes(); // 48 bytes
                                  // Generate PoP: sign(sk, pk_bytes) with PoP DST
    let pop = sk.sign(&pk_bytes, BLS_POP_DST, &[]);
    let pop_bytes = pop.to_bytes(); // 96 bytes
                                    // pk_with_pop = pk(48) || pop(96) = 144 bytes
    let mut pk_with_pop = pk_bytes.to_vec();
    pk_with_pop.extend_from_slice(&pop_bytes);
    (sk.to_bytes().to_vec(), pk_with_pop)
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

// --- Ristretto255 ---

/// A store of RistrettoPoints indexed by u64 handles.
/// Uses Arc<Mutex<...>> to satisfy Send + Sync requirements of the linker.
pub type PointStore = Arc<Mutex<Vec<RistrettoPoint>>>;

pub(crate) fn new_point_store() -> PointStore {
    Arc::new(Mutex::new(Vec::new()))
}

fn push_point(store: &PointStore, point: RistrettoPoint) -> u64 {
    let mut s = store.lock().unwrap();
    s.push(point);
    (s.len() - 1) as u64
}

fn get_point(store: &PointStore, handle: u64) -> RistrettoPoint {
    store.lock().unwrap()[handle as usize]
}

fn set_point(store: &PointStore, handle: u64, point: RistrettoPoint) {
    store.lock().unwrap()[handle as usize] = point;
}

fn scalar_from_bytes(bytes: &[u8]) -> Option<Scalar> {
    if bytes.len() != 32 {
        return None;
    }
    let arr: [u8; 32] = bytes.try_into().ok()?;
    Scalar::from_canonical_bytes(arr)
}

// --- Scalar operations ---

pub(crate) fn ristretto255_scalar_is_canonical(bytes: &[u8]) -> bool {
    if bytes.len() != 32 {
        return false;
    }
    let arr: [u8; 32] = bytes.try_into().unwrap();
    Scalar::from_canonical_bytes(arr).is_some()
}

pub(crate) fn ristretto255_scalar_from_u64(v: u64) -> Vec<u8> {
    Scalar::from(v).to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_from_u128(v: u128) -> Vec<u8> {
    Scalar::from(v).to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_reduced_from_32_bytes(bytes: &[u8]) -> Vec<u8> {
    let arr: [u8; 32] = bytes.try_into().expect("expected 32 bytes");
    Scalar::from_bytes_mod_order(arr).to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_uniform_from_64_bytes(bytes: &[u8]) -> Vec<u8> {
    let arr: [u8; 64] = bytes.try_into().expect("expected 64 bytes");
    Scalar::from_bytes_mod_order_wide(&arr).to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_from_sha512(bytes: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    let hash = sha2::Sha512::digest(bytes);
    let arr: [u8; 64] = hash.into();
    Scalar::from_bytes_mod_order_wide(&arr).to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_invert(bytes: &[u8]) -> Vec<u8> {
    let s = scalar_from_bytes(bytes).expect("invalid scalar");
    s.invert().to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_mul(a: &[u8], b: &[u8]) -> Vec<u8> {
    let sa = scalar_from_bytes(a).expect("invalid scalar a");
    let sb = scalar_from_bytes(b).expect("invalid scalar b");
    (sa * sb).to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_add(a: &[u8], b: &[u8]) -> Vec<u8> {
    let sa = scalar_from_bytes(a).expect("invalid scalar a");
    let sb = scalar_from_bytes(b).expect("invalid scalar b");
    (sa + sb).to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_sub(a: &[u8], b: &[u8]) -> Vec<u8> {
    let sa = scalar_from_bytes(a).expect("invalid scalar a");
    let sb = scalar_from_bytes(b).expect("invalid scalar b");
    (sa - sb).to_bytes().to_vec()
}

pub(crate) fn ristretto255_scalar_neg(a: &[u8]) -> Vec<u8> {
    let sa = scalar_from_bytes(a).expect("invalid scalar");
    (-sa).to_bytes().to_vec()
}

// --- Point operations ---

pub(crate) fn ristretto255_point_identity(store: &PointStore) -> u64 {
    push_point(store, RistrettoPoint::identity())
}

pub(crate) fn ristretto255_point_is_canonical(bytes: &[u8]) -> bool {
    if bytes.len() != 32 {
        return false;
    }
    CompressedRistretto::from_slice(bytes)
        .decompress()
        .is_some()
}

pub(crate) fn ristretto255_point_decompress(store: &PointStore, bytes: &[u8]) -> (u64, bool) {
    if bytes.len() != 32 {
        return (0, false);
    }
    match CompressedRistretto::from_slice(bytes).decompress() {
        Some(point) => (push_point(store, point), true),
        None => (0, false),
    }
}

pub(crate) fn ristretto255_point_clone(store: &PointStore, handle: u64) -> u64 {
    let point = get_point(store, handle);
    push_point(store, point)
}

pub(crate) fn ristretto255_point_compress(store: &PointStore, handle: u64) -> Vec<u8> {
    get_point(store, handle).compress().to_bytes().to_vec()
}

pub(crate) fn ristretto255_point_mul(
    store: &PointStore,
    handle: u64,
    scalar_bytes: &[u8],
    in_place: bool,
) -> u64 {
    let s = scalar_from_bytes(scalar_bytes).expect("invalid scalar");
    let result = get_point(store, handle) * s;
    if in_place {
        set_point(store, handle, result);
        handle
    } else {
        push_point(store, result)
    }
}

pub(crate) fn ristretto255_point_add(store: &PointStore, h1: u64, h2: u64, in_place: bool) -> u64 {
    let result = get_point(store, h1) + get_point(store, h2);
    if in_place {
        set_point(store, h1, result);
        h1
    } else {
        push_point(store, result)
    }
}

pub(crate) fn ristretto255_point_sub(store: &PointStore, h1: u64, h2: u64, in_place: bool) -> u64 {
    let result = get_point(store, h1) - get_point(store, h2);
    if in_place {
        set_point(store, h1, result);
        h1
    } else {
        push_point(store, result)
    }
}

pub(crate) fn ristretto255_point_neg(store: &PointStore, handle: u64, in_place: bool) -> u64 {
    let result = -get_point(store, handle);
    if in_place {
        set_point(store, handle, result);
        handle
    } else {
        push_point(store, result)
    }
}

pub(crate) fn ristretto255_point_equals(store: &PointStore, h1: u64, h2: u64) -> bool {
    get_point(store, h1) == get_point(store, h2)
}

pub(crate) fn ristretto255_basepoint_mul(store: &PointStore, scalar_bytes: &[u8]) -> u64 {
    let s = scalar_from_bytes(scalar_bytes).expect("invalid scalar");
    push_point(store, RISTRETTO_BASEPOINT_POINT * s)
}

pub(crate) fn ristretto255_basepoint_double_mul(
    store: &PointStore,
    a_bytes: &[u8],
    handle: u64,
    b_bytes: &[u8],
) -> u64 {
    let a = scalar_from_bytes(a_bytes).expect("invalid scalar a");
    let b = scalar_from_bytes(b_bytes).expect("invalid scalar b");
    let point = get_point(store, handle);
    let result =
        RistrettoPoint::vartime_multiscalar_mul(&[a, b], &[point, RISTRETTO_BASEPOINT_POINT]);
    push_point(store, result)
}

pub(crate) fn ristretto255_double_scalar_mul(
    store: &PointStore,
    h1: u64,
    h2: u64,
    s1_bytes: &[u8],
    s2_bytes: &[u8],
) -> u64 {
    let s1 = scalar_from_bytes(s1_bytes).expect("invalid scalar s1");
    let s2 = scalar_from_bytes(s2_bytes).expect("invalid scalar s2");
    let p1 = get_point(store, h1);
    let p2 = get_point(store, h2);
    let result = RistrettoPoint::vartime_multiscalar_mul(&[s1, s2], &[p1, p2]);
    push_point(store, result)
}

pub(crate) fn ristretto255_new_point_from_sha512(store: &PointStore, bytes: &[u8]) -> u64 {
    use sha2::Digest;
    let hash = sha2::Sha512::digest(bytes);
    let arr: [u8; 64] = hash.into();
    push_point(store, RistrettoPoint::from_uniform_bytes(&arr))
}

pub(crate) fn ristretto255_new_point_from_64_uniform_bytes(
    store: &PointStore,
    bytes: &[u8],
) -> u64 {
    let arr: [u8; 64] = bytes.try_into().expect("expected 64 bytes");
    push_point(store, RistrettoPoint::from_uniform_bytes(&arr))
}

pub(crate) fn ristretto255_multi_scalar_mul(
    store: &PointStore,
    handles: &[u64],
    scalar_bytes_list: &[Vec<u8>],
) -> u64 {
    let points: Vec<RistrettoPoint> = handles.iter().map(|&h| get_point(store, h)).collect();
    let scalars: Vec<Scalar> = scalar_bytes_list
        .iter()
        .map(|b| scalar_from_bytes(b).expect("invalid scalar"))
        .collect();
    let result = RistrettoPoint::vartime_multiscalar_mul(&scalars, &points);
    push_point(store, result)
}

// --- Bulletproofs ---

/// Leaks a byte slice to get a `'static` reference.
/// merlin v3's `Transcript::new` requires `&'static [u8]`.
fn leak_dst(dst: &[u8]) -> &'static [u8] {
    Box::leak(dst.to_vec().into_boxed_slice())
}

pub(crate) fn ristretto255_bulletproofs_verify_range_proof(
    store: &PointStore,
    com_bytes: &[u8],
    val_base_handle: u64,
    rand_base_handle: u64,
    proof_bytes: &[u8],
    num_bits: u64,
    dst: &[u8],
) -> bool {
    use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};

    if com_bytes.len() != 32 {
        return false;
    }
    let commitment = CompressedRistretto::from_slice(com_bytes);
    let val_base = get_point(store, val_base_handle);
    let rand_base = get_point(store, rand_base_handle);

    let Ok(proof) = RangeProof::from_bytes(proof_bytes) else {
        return false;
    };

    let pg = PedersenGens {
        B: val_base,
        B_blinding: rand_base,
    };
    let bp_gens = BulletproofGens::new(num_bits as usize, 1);
    let mut transcript = merlin::Transcript::new(leak_dst(dst));

    proof
        .verify_single(
            &bp_gens,
            &pg,
            &mut transcript,
            &commitment,
            num_bits as usize,
        )
        .is_ok()
}

pub(crate) fn ristretto255_bulletproofs_verify_batch_range_proof(
    store: &PointStore,
    com_bytes_list: &[Vec<u8>],
    val_base_handle: u64,
    rand_base_handle: u64,
    proof_bytes: &[u8],
    num_bits: u64,
    dst: &[u8],
) -> bool {
    use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};

    let commitments: Vec<CompressedRistretto> = com_bytes_list
        .iter()
        .map(|b| CompressedRistretto::from_slice(b))
        .collect();
    let val_base = get_point(store, val_base_handle);
    let rand_base = get_point(store, rand_base_handle);

    let Ok(proof) = RangeProof::from_bytes(proof_bytes) else {
        return false;
    };

    let pg = PedersenGens {
        B: val_base,
        B_blinding: rand_base,
    };
    let bp_gens = BulletproofGens::new(num_bits as usize, com_bytes_list.len());
    let mut transcript = merlin::Transcript::new(leak_dst(dst));

    proof
        .verify_multiple(
            &bp_gens,
            &pg,
            &mut transcript,
            &commitments,
            num_bits as usize,
        )
        .is_ok()
}

pub(crate) fn ristretto255_bulletproofs_prove_range(
    store: &PointStore,
    val_bytes: &[u8],
    r_bytes: &[u8],
    num_bits: u64,
    dst: &[u8],
    val_base_handle: u64,
    rand_base_handle: u64,
) -> (Vec<u8>, Vec<u8>) {
    use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};

    let val_scalar = scalar_from_bytes(val_bytes).expect("invalid value scalar");
    // Extract u64 value from scalar bytes (little-endian)
    let val = u64::from_le_bytes(val_bytes[..8].try_into().unwrap());
    let r = scalar_from_bytes(r_bytes).expect("invalid randomness scalar");
    let _ = val_scalar; // val_scalar validates canonical form
    let val_base = get_point(store, val_base_handle);
    let rand_base = get_point(store, rand_base_handle);

    let pg = PedersenGens {
        B: val_base,
        B_blinding: rand_base,
    };
    let bp_gens = BulletproofGens::new(num_bits as usize, 1);
    let mut transcript = merlin::Transcript::new(leak_dst(dst));

    let (proof, commitment) =
        RangeProof::prove_single(&bp_gens, &pg, &mut transcript, val, &r, num_bits as usize)
            .expect("prove_single failed");

    (proof.to_bytes(), commitment.to_bytes().to_vec())
}

pub(crate) fn ristretto255_bulletproofs_prove_batch_range(
    store: &PointStore,
    val_bytes_list: &[Vec<u8>],
    r_bytes_list: &[Vec<u8>],
    num_bits: u64,
    dst: &[u8],
    val_base_handle: u64,
    rand_base_handle: u64,
) -> (Vec<u8>, Vec<Vec<u8>>) {
    use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};

    let vals: Vec<u64> = val_bytes_list
        .iter()
        .map(|b| u64::from_le_bytes(b[..8].try_into().unwrap()))
        .collect();
    let rs: Vec<Scalar> = r_bytes_list
        .iter()
        .map(|b| scalar_from_bytes(b).expect("invalid randomness scalar"))
        .collect();
    let val_base = get_point(store, val_base_handle);
    let rand_base = get_point(store, rand_base_handle);

    let pg = PedersenGens {
        B: val_base,
        B_blinding: rand_base,
    };
    let bp_gens = BulletproofGens::new(num_bits as usize, vals.len());
    let mut transcript = merlin::Transcript::new(leak_dst(dst));

    let (proof, commitments) = RangeProof::prove_multiple(
        &bp_gens,
        &pg,
        &mut transcript,
        &vals,
        &rs,
        num_bits as usize,
    )
    .expect("prove_multiple failed");

    let com_bytes_list: Vec<Vec<u8>> = commitments.iter().map(|c| c.to_bytes().to_vec()).collect();
    (proof.to_bytes(), com_bytes_list)
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
    fn test_bls12381_aggregate_pubkeys() {
        let ikm1 = [1u8; 32];
        let ikm2 = [2u8; 32];
        let sk1 = blst::min_pk::SecretKey::key_gen(&ikm1, &[]).unwrap();
        let sk2 = blst::min_pk::SecretKey::key_gen(&ikm2, &[]).unwrap();
        let pk1 = sk1.sk_to_pk().to_bytes().to_vec();
        let pk2 = sk2.sk_to_pk().to_bytes().to_vec();

        let (agg_pk, success) = bls12381_aggregate_pubkeys(&[pk1.clone(), pk2.clone()]);
        assert!(success);
        assert_eq!(agg_pk.len(), 48);

        // Empty input should fail
        let (_, success) = bls12381_aggregate_pubkeys(&[]);
        assert!(!success);
    }

    #[test]
    fn test_bls12381_aggregate_signatures() {
        let ikm1 = [1u8; 32];
        let ikm2 = [2u8; 32];
        let sk1 = blst::min_pk::SecretKey::key_gen(&ikm1, &[]).unwrap();
        let sk2 = blst::min_pk::SecretKey::key_gen(&ikm2, &[]).unwrap();

        let msg = b"test aggregate";
        let sig1 = bls12381_sign(&sk1.to_bytes(), msg);
        let sig2 = bls12381_sign(&sk2.to_bytes(), msg);

        let (agg_sig, success) = bls12381_aggregate_signatures(&[sig1, sig2]);
        assert!(success);
        assert_eq!(agg_sig.len(), 96);
    }

    #[test]
    fn test_bls12381_verify_aggregate_signature() {
        let ikm1 = [1u8; 32];
        let ikm2 = [2u8; 32];
        let sk1 = blst::min_pk::SecretKey::key_gen(&ikm1, &[]).unwrap();
        let sk2 = blst::min_pk::SecretKey::key_gen(&ikm2, &[]).unwrap();
        let pk1 = sk1.sk_to_pk().to_bytes().to_vec();
        let pk2 = sk2.sk_to_pk().to_bytes().to_vec();

        let msg1 = b"message one".to_vec();
        let msg2 = b"message two".to_vec();
        let sig1 = bls12381_sign(&sk1.to_bytes(), &msg1);
        let sig2 = bls12381_sign(&sk2.to_bytes(), &msg2);

        let (agg_sig, success) = bls12381_aggregate_signatures(&[sig1, sig2]);
        assert!(success);

        let valid = bls12381_verify_aggregate_signature(
            &agg_sig,
            &[pk1.clone(), pk2.clone()],
            &[msg1.clone(), msg2.clone()],
        );
        assert!(valid);

        // Wrong message should fail
        let valid =
            bls12381_verify_aggregate_signature(&agg_sig, &[pk1, pk2], &[msg1, b"wrong".to_vec()]);
        assert!(!valid);
    }

    #[test]
    fn test_bls12381_generate_keys() {
        let (sk, pk_with_pop) = bls12381_generate_keys();
        assert_eq!(sk.len(), 32);
        assert_eq!(pk_with_pop.len(), 144); // 48 (pk) + 96 (pop)

        // Validate the generated key
        let pk_bytes = &pk_with_pop[..48];
        let pop_bytes = &pk_with_pop[48..];
        assert!(bls12381_validate_pubkey(pk_bytes));
        assert!(bls12381_verify_proof_of_possession(pk_bytes, pop_bytes));
    }

    #[test]
    fn test_secp256k1_generate_test_vector() {
        use k256::ecdsa::SigningKey as K256SigningKey;
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
