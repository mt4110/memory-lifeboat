# Migration Guide

## Choose the sources deliberately

Memory Lifeboat accepts only four user-selected sources:

- an official ChatGPT data export ZIP
- text you select and copy from Atlas Browser Memories
- an exported bookmarks HTML file
- a plain-text URL list

It deliberately does not inspect login cookies, browser history databases, Keychain items, or internal application databases. Those sources are unsafe, unstable, and cannot provide a reproducible migration path.

## Run an import

```bash
cargo run -- doctor
cargo run -- import chatgpt-export ~/Downloads/chatgpt-export.zip
pbpaste | cargo run -- import atlas-memory-text
cargo run -- import bookmarks ~/Downloads/bookmarks.html
cargo run -- import urls ~/Downloads/urls.txt
cargo run -- report
```

Every import refreshes `audit/latest-report.md`. Review it first: it records which source kinds were observed and the number of items found.

## What this first release will not do

This release performs rescue and audit only. It does not automatically inject candidate records into Codex native memory. Direct writes are intentionally postponed until there is a stable, official import surface; otherwise an update could lose the data or misinterpret imported text as instruction.
