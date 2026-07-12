# Security

## Non-goals

- no cookie or session migration
- no Atlas internal database parsing
- no hidden API calls
- no automatic writes to native agent memory stores

## Local storage

Records are encrypted with ChaCha20-Poly1305. The symmetric key is stored as a generic password in macOS Keychain under service `memory-lifeboat`.

Source manifests and audit reports are plaintext metadata and should not contain imported memory text.
