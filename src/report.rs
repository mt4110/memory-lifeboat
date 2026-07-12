use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;

use crate::{store::Store, types::SourceManifest};

pub fn write_run_log(store: &Store, manifest: &SourceManifest) -> Result<PathBuf> {
    store.ensure_layout()?;
    let runs_dir = store.audit_dir().join("runs");
    fs::create_dir_all(&runs_dir)?;
    let path = runs_dir.join(format!("{}.md", manifest.id));

    let mut body = String::new();
    body.push_str("# Memory Lifeboat Run Log\n\n");
    body.push_str("## Result\n\n");
    body.push_str("- Status: `completed`\n");
    body.push_str(&format!("- Completed at: `{}`\n", manifest.completed_at));
    body.push_str(&format!("- Source: `{:?}`\n", manifest.source));
    body.push_str(&format!(
        "- Capture method: `{}`\n",
        manifest.capture_method
    ));
    body.push_str(&format!("- Observed items: {}\n", manifest.observed_items));
    body.push_str(&format!("- Coverage: `{:?}`\n", manifest.coverage));
    if let Some(sha256) = &manifest.evidence_sha256 {
        body.push_str(&format!("- Evidence SHA-256: `{sha256}`\n"));
    }
    body.push_str("\n## Safety Checks\n\n");
    body.push_str("- No network transfer was performed.\n");
    body.push_str(
        "- No cookies, Keychain browser secrets, or internal browser databases were read.\n",
    );
    body.push_str("- Candidate data was not written to native Codex memory.\n");
    body.push_str("- This log intentionally excludes source paths and imported content.\n");

    fs::write(&path, body)?;
    append_event(store, manifest)?;
    Ok(path)
}

fn append_event(store: &Store, manifest: &SourceManifest) -> Result<()> {
    write_event(
        store,
        &format!(
            "time={} status=completed source={:?} observed_items={} coverage={:?} manifest={}",
            manifest.completed_at,
            manifest.source,
            manifest.observed_items,
            manifest.coverage,
            manifest.id,
        ),
    )
}

pub fn write_status_event(store: &Store, action: &str) -> Result<()> {
    write_event(
        store,
        &format!(
            "time={} status=completed action={action}",
            time::OffsetDateTime::now_utc(),
        ),
    )
}

fn write_event(store: &Store, line: &str) -> Result<()> {
    let path = store.audit_dir().join("events.log");
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")?;
    file.sync_data()?;
    Ok(())
}

pub fn write_report(store: &Store) -> Result<PathBuf> {
    store.ensure_layout()?;
    let manifests = store.read_manifests()?;
    let records = store.read_records()?;
    let path = store.audit_dir().join("latest-report.md");

    let mut body = String::new();
    body.push_str("# Memory Lifeboat Audit Report\n\n");
    body.push_str(
        "This report is local-only. Candidate records remain observations until reviewed.\n\n",
    );
    body.push_str("## Summary\n\n");
    body.push_str(&format!("- Manifests: {}\n", manifests.len()));
    body.push_str(&format!("- Candidate records: {}\n\n", records.len()));
    body.push_str(&format!("- Run logs: {}\n\n", run_log_count(store)?));
    body.push_str("## Sources\n\n");
    for manifest in &manifests {
        body.push_str(&format!(
            "- `{}`: {:?}, method `{}`, observed {}, coverage {:?}\n",
            manifest.id,
            manifest.source,
            manifest.capture_method,
            manifest.observed_items,
            manifest.coverage
        ));
    }
    body.push_str("\n## Safety Notes\n\n");
    body.push_str("- This tool does not read cookies, Keychain browser secrets, Atlas internal databases, or Chromium profile stores.\n");
    body.push_str("- This tool does not write to native Codex memories.\n");
    body.push_str(
        "- Imported text is stored as candidate observation data, not as executable instruction.\n",
    );

    fs::write(&path, body)?;
    Ok(path)
}

fn run_log_count(store: &Store) -> Result<usize> {
    let runs_dir = store.audit_dir().join("runs");
    if !runs_dir.exists() {
        return Ok(0);
    }
    Ok(fs::read_dir(runs_dir)?.count())
}
