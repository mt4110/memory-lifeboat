use anyhow::Result;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::bookmarks::Bookmark;
use crate::store::sha256_hex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceManifest {
    pub schema: String,
    pub id: String,
    pub source: ImportKind,
    pub capture_method: String,
    pub observed_items: usize,
    pub expected_items: Option<usize>,
    pub coverage: Coverage,
    pub evidence_sha256: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub completed_at: OffsetDateTime,
}

impl SourceManifest {
    pub fn new(source: ImportKind, capture_method: &str, coverage: Coverage) -> Self {
        Self {
            schema: "memory-lifeboat/source-manifest/v1".to_string(),
            id: stable_id(
                "manifest",
                &format!("{source:?}:{capture_method}:{}", now()),
            ),
            source,
            capture_method: capture_method.to_string(),
            observed_items: 0,
            expected_items: None,
            coverage,
            evidence_sha256: None,
            completed_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn with_evidence_sha256(mut self, sha256: String) -> Self {
        self.evidence_sha256 = Some(sha256);
        self
    }

    pub fn with_observed_items(mut self, count: usize) -> Self {
        self.observed_items = count;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportKind {
    ChatgptExport,
    AtlasMemoryText,
    Bookmarks,
    Urls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Coverage {
    Complete,
    Partial,
    Unverifiable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub schema: String,
    pub id: String,
    pub kind: MemoryKind,
    pub statement: String,
    pub instruction_class: InstructionClass,
    pub status: RecordStatus,
    pub source_manifest_id: String,
    pub evidence_sha256: String,
    #[serde(with = "time::serde::rfc3339")]
    pub captured_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Identity,
    Preference,
    Constraint,
    Workflow,
    ProjectFact,
    ArchitectureDecision,
    RejectedAlternative,
    OpenTask,
    Reference,
    BrowserAsset,
    ConversationArchive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionClass {
    Observation,
    Directive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordStatus {
    Candidate,
    Active,
}

pub fn records_from_atlas_text(text: &str, manifest_id: &str) -> Result<Vec<MemoryRecord>> {
    let mut records = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        records.push(candidate_record(
            MemoryKind::Reference,
            line,
            manifest_id,
            "atlas-memory-text",
        ));
    }
    Ok(records)
}

pub fn records_from_bookmarks(
    items: Vec<Bookmark>,
    manifest_id: &str,
) -> Result<Vec<MemoryRecord>> {
    Ok(items
        .into_iter()
        .map(|item| {
            let statement = match item.title.trim().is_empty() {
                true => item.url,
                false => format!("{} - {}", item.title.trim(), item.url),
            };
            candidate_record(
                MemoryKind::BrowserAsset,
                &statement,
                manifest_id,
                "bookmark",
            )
        })
        .collect())
}

pub fn parse_url_lines(text: &str) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if Url::parse(line).is_ok() {
            urls.push(line.to_string());
        }
    }
    urls.sort();
    urls.dedup();
    Ok(urls)
}

pub fn records_from_urls(urls: Vec<String>, manifest_id: &str) -> Result<Vec<MemoryRecord>> {
    Ok(urls
        .into_iter()
        .map(|url| candidate_record(MemoryKind::BrowserAsset, &url, manifest_id, "url"))
        .collect())
}

pub fn candidate_record(
    kind: MemoryKind,
    statement: &str,
    manifest_id: &str,
    namespace: &str,
) -> MemoryRecord {
    let material = format!("{namespace}:{manifest_id}:{statement}");
    MemoryRecord {
        schema: "memory-lifeboat/memory/v1".to_string(),
        id: stable_id("mem", &material),
        kind,
        statement: statement.to_string(),
        instruction_class: InstructionClass::Observation,
        status: RecordStatus::Candidate,
        source_manifest_id: manifest_id.to_string(),
        evidence_sha256: sha256_hex(statement.as_bytes()),
        captured_at: OffsetDateTime::now_utc(),
    }
}

pub fn stable_id(prefix: &str, material: &str) -> String {
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, material.as_bytes());
    format!("{prefix}_{uuid}")
}

fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown-time".to_string())
}
