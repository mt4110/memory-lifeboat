use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::crypto;
use crate::types::{MemoryRecord, SourceManifest};

const DEFAULT_STORE: &str = "Library/Application Support/Memory Lifeboat";

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn open(root: Option<PathBuf>) -> Result<Self> {
        let root = match root {
            Some(root) => root,
            None => {
                let home = std::env::var_os("HOME").context("HOME is not set")?;
                PathBuf::from(home).join(DEFAULT_STORE)
            }
        };
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifests_dir(&self) -> PathBuf {
        self.root.join("manifests")
    }

    pub fn audit_dir(&self) -> PathBuf {
        self.root.join("audit")
    }

    pub fn blobs_dir(&self) -> PathBuf {
        self.root.join("store").join("blobs").join("sha256")
    }

    fn records_path(&self) -> PathBuf {
        self.root.join("store").join("records.jsonl.enc")
    }

    pub fn ensure_layout(&self) -> Result<()> {
        fs::create_dir_all(self.manifests_dir())?;
        fs::create_dir_all(self.audit_dir())?;
        fs::create_dir_all(self.blobs_dir())?;
        fs::create_dir_all(self.root.join("transactions"))?;
        fs::create_dir_all(self.root.join("backups"))?;
        Ok(())
    }

    pub fn encrypt_blob(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let sha = sha256_hex(&bytes);
        let target = self.blobs_dir().join(format!("{sha}.enc"));
        if !target.exists() {
            write_atomic(&target, &crypto::encrypt(&bytes)?)?;
        }
        Ok(sha)
    }

    pub fn write_manifest(&self, manifest: &SourceManifest) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(manifest)?;
        write_atomic(
            &self.manifests_dir().join(format!("{}.json", manifest.id)),
            &bytes,
        )
    }

    pub fn read_manifests(&self) -> Result<Vec<SourceManifest>> {
        let mut manifests = Vec::new();
        if !self.manifests_dir().exists() {
            return Ok(manifests);
        }
        for entry in fs::read_dir(self.manifests_dir())? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let bytes = fs::read(&path)?;
                manifests.push(serde_json::from_slice(&bytes)?);
            }
        }
        manifests.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(manifests)
    }

    pub fn append_records(&self, new_records: Vec<MemoryRecord>) -> Result<()> {
        let mut records = self.read_records()?;
        records.extend(new_records);
        records.sort_by(|a, b| a.id.cmp(&b.id));

        let mut jsonl = Vec::new();
        for record in records {
            serde_json::to_writer(&mut jsonl, &record)?;
            jsonl.push(b'\n');
        }
        let encrypted = crypto::encrypt(&jsonl)?;
        write_atomic(&self.records_path(), &encrypted)
    }

    pub fn read_records(&self) -> Result<Vec<MemoryRecord>> {
        let path = self.records_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let envelope = fs::read(&path)?;
        let plaintext = crypto::decrypt(&envelope)?;
        let mut records = Vec::new();
        for line in plaintext.split(|byte| *byte == b'\n') {
            if line.iter().all(|byte| byte.is_ascii_whitespace()) {
                continue;
            }
            records.push(serde_json::from_slice(line)?);
        }
        Ok(records)
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("target path has no parent")?;
    fs::create_dir_all(parent)?;
    let mut temp = NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut temp, bytes)?;
    temp.persist(path)?;
    Ok(())
}
