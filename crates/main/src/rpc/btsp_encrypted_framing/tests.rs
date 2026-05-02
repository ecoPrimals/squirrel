// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;

// ── Key derivation ──────────────────────────────────────────────────────

#[test]
fn session_keys_derive_deterministic() {
    let hk = b"handshake-key-material-32-bytes!";
    let cn = b"client-nonce-32-bytes-0000000000";
    let sn = b"server-nonce-32-bytes-0000000000";

    let k1 = SessionKeys::derive(hk, cn, sn).expect("derive");
    let k2 = SessionKeys::derive(hk, cn, sn).expect("derive");
    assert_eq!(k1.c2s_key, k2.c2s_key, "c2s must be deterministic");
    assert_eq!(k1.s2c_key, k2.s2c_key, "s2c must be deterministic");
}

#[test]
fn session_keys_c2s_differs_from_s2c() {
    let hk = b"handshake-key-material-32-bytes!";
    let cn = b"client-nonce-32-bytes-0000000000";
    let sn = b"server-nonce-32-bytes-0000000000";

    let keys = SessionKeys::derive(hk, cn, sn).expect("derive");
    assert_ne!(keys.c2s_key, keys.s2c_key, "directional keys must differ");
}

#[test]
fn session_keys_different_nonces_produce_different_keys() {
    let hk = b"handshake-key-material-32-bytes!";
    let cn1 = b"client-nonce-aaaa-bytes-00000000";
    let cn2 = b"client-nonce-bbbb-bytes-00000000";
    let sn = b"server-nonce-32-bytes-0000000000";

    let k1 = SessionKeys::derive(hk, cn1, sn).expect("derive");
    let k2 = SessionKeys::derive(hk, cn2, sn).expect("derive");
    assert_ne!(k1.c2s_key, k2.c2s_key, "different nonces → different keys");
}

#[test]
fn session_keys_debug_redacts() {
    let hk = b"handshake-key-material-32-bytes!";
    let cn = b"client-nonce-32-bytes-0000000000";
    let sn = b"server-nonce-32-bytes-0000000000";

    let keys = SessionKeys::derive(hk, cn, sn).expect("derive");
    let debug = format!("{keys:?}");
    assert!(debug.contains("REDACTED"), "Debug must not leak key bytes");
    assert!(
        !debug.contains("handshake"),
        "Debug must not leak key material"
    );
}

// ── Encrypt / decrypt roundtrip ─────────────────────────────────────────

#[test]
fn encrypt_decrypt_roundtrip() {
    let key = [42u8; 32];
    let plaintext = b"hello encrypted BTSP frame";

    let frame = encrypt_frame(&key, plaintext).expect("encrypt");

    // Parse the frame: [4B len][payload]
    let len = u32::from_be_bytes(frame[..4].try_into().unwrap()) as usize;
    assert_eq!(len, frame.len() - 4);

    let payload = &frame[4..];
    let decrypted = decrypt_frame(&key, payload).expect("decrypt");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn decrypt_wrong_key_fails() {
    let key1 = [42u8; 32];
    let key2 = [99u8; 32];
    let plaintext = b"secret message";

    let frame = encrypt_frame(&key1, plaintext).expect("encrypt");
    let payload = &frame[4..];

    let result = decrypt_frame(&key2, payload);
    assert!(result.is_err(), "wrong key must fail decryption");
}

#[test]
fn decrypt_truncated_frame_fails() {
    let key = [42u8; 32];
    let plaintext = b"test";

    let frame = encrypt_frame(&key, plaintext).expect("encrypt");
    let payload = &frame[4..];

    // Truncate to less than nonce + tag + 1
    let truncated = &payload[..NONCE_SIZE + TAG_SIZE];
    let result = decrypt_frame(&key, truncated);
    assert!(result.is_err());
}

#[test]
fn decrypt_empty_payload_fails() {
    let key = [42u8; 32];
    let result = decrypt_frame(&key, &[]);
    assert!(result.is_err());
}

#[test]
fn encrypt_frame_has_correct_wire_format() {
    let key = [1u8; 32];
    let plaintext = b"wire format test";

    let frame = encrypt_frame(&key, plaintext).expect("encrypt");

    // First 4 bytes: big-endian u32 length
    let len = u32::from_be_bytes(frame[..4].try_into().unwrap()) as usize;

    // After header: 12-byte nonce + ciphertext + 16-byte tag
    assert_eq!(len, NONCE_SIZE + plaintext.len() + TAG_SIZE);
    assert_eq!(frame.len(), 4 + len);
}

#[test]
fn encrypt_unique_nonces() {
    let key = [7u8; 32];
    let plaintext = b"same plaintext";

    let f1 = encrypt_frame(&key, plaintext).expect("encrypt 1");
    let f2 = encrypt_frame(&key, plaintext).expect("encrypt 2");

    // Nonces are bytes 4..16
    let n1 = &f1[4..4 + NONCE_SIZE];
    let n2 = &f2[4..4 + NONCE_SIZE];
    assert_ne!(n1, n2, "each frame must use a unique random nonce");
}

// ── Async frame I/O ─────────────────────────────────────────────────────

#[tokio::test]
async fn async_read_write_roundtrip() {
    let key = [55u8; 32];
    let plaintext = b"async encrypted roundtrip";

    let mut buf = Vec::new();
    write_encrypted_frame(&mut buf, &key, plaintext)
        .await
        .expect("write");

    let mut cursor = std::io::Cursor::new(buf);
    let decrypted = read_encrypted_frame(&mut cursor, &key).await.expect("read");
    assert_eq!(decrypted, plaintext);
}

#[tokio::test]
async fn async_read_oversized_frame_rejected() {
    // Craft a frame header claiming > MAX_ENCRYPTED_FRAME
    let huge_len = (MAX_ENCRYPTED_FRAME as u32) + 1;
    let mut data = Vec::new();
    data.extend_from_slice(&huge_len.to_be_bytes());
    data.extend_from_slice(&[0u8; 64]); // dummy payload

    let key = [0u8; 32];
    let mut cursor = std::io::Cursor::new(data);
    let result = read_encrypted_frame(&mut cursor, &key).await;
    assert!(matches!(result, Err(FrameError::FrameTooLarge { .. })));
}

#[tokio::test]
async fn async_read_too_short_frame_rejected() {
    // Frame claims length of 5 bytes — less than nonce + tag + 1
    let short_len: u32 = 5;
    let mut data = Vec::new();
    data.extend_from_slice(&short_len.to_be_bytes());
    data.extend_from_slice(&[0u8; 5]);

    let key = [0u8; 32];
    let mut cursor = std::io::Cursor::new(data);
    let result = read_encrypted_frame(&mut cursor, &key).await;
    assert!(matches!(result, Err(FrameError::FrameTooShort { .. })));
}

// ── Nonce generation ────────────────────────────────────────────────────

#[test]
fn server_nonce_is_base64_and_unique() {
    let n1 = generate_server_nonce();
    let n2 = generate_server_nonce();

    assert_ne!(n1, n2, "nonces must be unique");

    let decoded = BASE64.decode(&n1).expect("must be valid base64");
    assert_eq!(decoded.len(), 32, "server nonce must be 32 bytes");
}

// ── Nonce decode ────────────────────────────────────────────────────────

#[test]
fn decode_nonce_base64() {
    let raw = [0xABu8; 32];
    let encoded = BASE64.encode(raw);
    let decoded = decode_nonce(&encoded).expect("decode");
    assert_eq!(decoded, raw);
}

#[test]
fn decode_nonce_invalid_fails() {
    let result = decode_nonce("!!!not-base64!!!");
    assert!(result.is_err());
}

// ── Multiple frames on same stream ──────────────────────────────────────

#[tokio::test]
async fn multiple_frames_sequential() {
    let key = [88u8; 32];
    let messages: &[&[u8]] = &[b"first", b"second", b"third frame with more data"];

    let mut buf = Vec::new();
    for msg in messages {
        write_encrypted_frame(&mut buf, &key, msg)
            .await
            .expect("write");
    }

    let mut cursor = std::io::Cursor::new(buf);
    for expected in messages {
        let decrypted = read_encrypted_frame(&mut cursor, &key).await.expect("read");
        assert_eq!(decrypted, *expected);
    }
}
