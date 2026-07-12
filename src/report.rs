use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::store::Store;

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
