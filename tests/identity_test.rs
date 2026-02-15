use net4::identity::{self, NodeIdentity};
use net4::types::Address;

#[test]
fn generate_produces_unique_identities() {
    let id1 = NodeIdentity::generate();
    let id2 = NodeIdentity::generate();
    assert_ne!(id1.address, id2.address);
}

#[test]
fn address_is_not_zero() {
    let id = NodeIdentity::generate();
    assert_ne!(id.address, Address::zero());
}

#[test]
fn sign_and_verify_valid() {
    let id = NodeIdentity::generate();
    let msg = b"test message";
    let sig = id.sign(msg);
    assert!(identity::verify(id.verifying_key.as_bytes(), msg, &sig));
}

#[test]
fn verify_fails_with_wrong_message() {
    let id = NodeIdentity::generate();
    let sig = id.sign(b"correct message");
    assert!(!identity::verify(id.verifying_key.as_bytes(), b"wrong message", &sig));
}

#[test]
fn verify_fails_with_wrong_key() {
    let id1 = NodeIdentity::generate();
    let id2 = NodeIdentity::generate();
    let sig = id1.sign(b"hello");
    assert!(!identity::verify(id2.verifying_key.as_bytes(), b"hello", &sig));
}

#[test]
fn verify_fails_with_garbage_signature() {
    let id = NodeIdentity::generate();
    let garbage = vec![0u8; 64];
    assert!(!identity::verify(id.verifying_key.as_bytes(), b"hello", &garbage));
}

#[test]
fn verify_fails_with_short_signature() {
    let id = NodeIdentity::generate();
    let short = vec![1u8; 10];
    assert!(!identity::verify(id.verifying_key.as_bytes(), b"hello", &short));
}

#[test]
fn address_derived_from_pubkey_is_deterministic() {
    use sha2::{Sha256, Digest};
    let id = NodeIdentity::generate();
    let expected = Address::from_bytes(
        Sha256::digest(id.verifying_key.as_bytes()).into(),
    );
    assert_eq!(id.address, expected);
}
