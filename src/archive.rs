use std::fs::File;
use std::io::Read;
use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use zip::ZipArchive;

use crate::types::{MemoryKind, MemoryRecord, candidate_record};

const MAX_ENTRIES: usize = 20_000;
const MAX_TOTAL_UNCOMPRESSED: u64 = 2 * 1024 * 1024 * 1024;
const MAX_CONVERSATIONS_JSON: u64 = 256 * 1024 * 1024;

#[derive(Debug)]
pub struct ExportSummary {
    pub entries: Vec<ArchiveEntry>,
    pub conversation_count: Option<usize>,
}

#[derive(Debug)]
pub struct ArchiveEntry {
    pub path: String,
    pub uncompressed_size: u64,
}

impl ExportSummary {
    pub fn into_records(self, manifest_id: &str) -> Result<Vec<MemoryRecord>> {
        let mut records = Vec::new();
        if let Some(count) = self.conversation_count {
            records.push(candidate_record(
                MemoryKind::ConversationArchive,
                &format!("ChatGPT export contains {count} conversation records"),
                manifest_id,
                "chatgpt-export-summary",
            ));
        }
        for entry in self.entries {
            records.push(candidate_record(
                MemoryKind::Reference,
                &format!(
                    "Archive entry: {} ({} bytes)",
                    entry.path, entry.uncompressed_size
                ),
                manifest_id,
                "chatgpt-export-entry",
            ));
        }
        Ok(records)
    }
}

pub fn inspect_chatgpt_export(path: &Path) -> Result<ExportSummary> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    if archive.len() > MAX_ENTRIES {
        bail!("zip has too many entries: {}", archive.len());
    }

    let mut entries = Vec::new();
    let mut total_uncompressed = 0_u64;
    let mut conversation_count = None;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        validate_zip_entry(&file)?;
        total_uncompressed = total_uncompressed.saturating_add(file.size());
        if total_uncompressed > MAX_TOTAL_UNCOMPRESSED {
            bail!("zip uncompressed size exceeds limit");
        }

        let path = file.name().to_string();
        if path == "conversations.json" {
            if file.size() > MAX_CONVERSATIONS_JSON {
                bail!("conversations.json exceeds size limit");
            }
            let mut data = String::new();
            file.read_to_string(&mut data)
                .context("conversations.json is not valid utf-8")?;
            let conversations: Vec<ConversationStub> =
                serde_json::from_str(&data).context("failed to parse conversations.json")?;
            conversation_count = Some(conversations.len());
        }

        entries.push(ArchiveEntry {
            path,
            uncompressed_size: file.size(),
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(ExportSummary {
        entries,
        conversation_count,
    })
}

fn validate_zip_entry(file: &zip::read::ZipFile<'_, File>) -> Result<()> {
    let name = file.name();
    let path = Path::new(name);
    if path.is_absolute() {
        bail!("zip entry uses absolute path: {name}");
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            bail!("zip entry uses unsafe path: {name}");
        }
    }
    if is_symlink(file) {
        bail!("zip entry is a symlink: {name}");
    }
    Ok(())
}

fn is_symlink(file: &zip::read::ZipFile<'_, File>) -> bool {
    file.unix_mode()
        .map(|mode| (mode & 0o170000) == 0o120000)
        .unwrap_or(false)
}

#[derive(Debug, Deserialize)]
struct ConversationStub {
    #[allow(dead_code)]
    title: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempfile::tempdir;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::inspect_chatgpt_export;

    #[test]
    fn inspects_a_minimal_chatgpt_export() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("export.zip");
        let file = File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        zip.start_file("conversations.json", SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut zip, br#"[{"title":"A conversation"}]"#).unwrap();
        zip.finish().unwrap();

        let summary = inspect_chatgpt_export(&path).unwrap();
        assert_eq!(summary.conversation_count, Some(1));
        assert_eq!(summary.entries.len(), 1);
    }
}
