use std::process::Command;

use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use rand::RngCore;

const SERVICE: &str = "memory-lifeboat";
const ACCOUNT: &str = "default-store-key";

pub fn load_or_create_key() -> Result<[u8; 32]> {
    if let Some(key) = read_keychain_key()? {
        return Ok(key);
    }

    let mut key = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    write_keychain_key(&key)?;
    Ok(key)
}

pub fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>> {
    let key = load_or_create_key()?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .context("failed to encrypt data")?;

    let envelope = serde_json::json!({
        "schema": "memory-lifeboat/encrypted-data/v1",
        "algorithm": "CHACHA20-POLY1305",
        "nonce": STANDARD.encode(nonce),
        "ciphertext": STANDARD.encode(ciphertext),
    });
    Ok(serde_json::to_vec_pretty(&envelope)?)
}

pub fn decrypt(envelope: &[u8]) -> Result<Vec<u8>> {
    let key = load_or_create_key()?;
    let value: serde_json::Value = serde_json::from_slice(envelope)?;
    let nonce_text = value
        .get("nonce")
        .and_then(|value| value.as_str())
        .context("encrypted data missing nonce")?;
    let ciphertext_text = value
        .get("ciphertext")
        .and_then(|value| value.as_str())
        .context("encrypted data missing ciphertext")?;
    let nonce_bytes = STANDARD.decode(nonce_text)?;
    let ciphertext = STANDARD.decode(ciphertext_text)?;
    if nonce_bytes.len() != 12 {
        bail!("encrypted data nonce has invalid length");
    }
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
        .context("failed to decrypt data")?;
    Ok(plaintext)
}

fn read_keychain_key() -> Result<Option<[u8; 32]>> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", SERVICE, "-a", ACCOUNT, "-w"])
        .output()
        .context("failed to call macOS security command")?;

    if !output.status.success() {
        return Ok(None);
    }

    let text = String::from_utf8(output.stdout)?;
    let bytes = STANDARD.decode(text.trim())?;
    if bytes.len() != 32 {
        bail!("keychain key has invalid length");
    }
    let mut key = [0_u8; 32];
    key.copy_from_slice(&bytes);
    Ok(Some(key))
}

fn write_keychain_key(key: &[u8; 32]) -> Result<()> {
    let encoded = STANDARD.encode(key);
    let status = Command::new("security")
        .args([
            "add-generic-password",
            "-U",
            "-s",
            SERVICE,
            "-a",
            ACCOUNT,
            "-w",
            &encoded,
        ])
        .status()
        .context("failed to call macOS security command")?;
    if !status.success() {
        bail!("failed to write encryption key to macOS Keychain");
    }
    Ok(())
}
