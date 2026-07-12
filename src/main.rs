mod archive;
mod bookmarks;
mod crypto;
mod report;
mod store;
mod types;

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use store::Store;
use types::{Coverage, ImportKind, SourceManifest};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, global = true, value_name = "DIR")]
    store: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Doctor,
    Import(ImportCommand),
    Report,
}

#[derive(Args)]
struct ImportCommand {
    #[command(subcommand)]
    source: ImportSource,
}

#[derive(Subcommand)]
enum ImportSource {
    ChatgptExport { zip: PathBuf },
    AtlasMemoryText { path: Option<PathBuf> },
    Bookmarks { html: PathBuf },
    Urls { path: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = Store::open(cli.store)?;

    match cli.command {
        Command::Doctor => doctor(&store),
        Command::Import(import) => import_source(&store, import.source),
        Command::Report => {
            let path = report::write_report(&store)?;
            println!("wrote {}", path.display());
            Ok(())
        }
    }
}

fn doctor(store: &Store) -> Result<()> {
    store.ensure_layout()?;
    crypto::load_or_create_key()?;
    report::write_status_event(store, "doctor")?;
    println!("store: {}", store.root().display());
    println!("keychain: ok");
    println!("network: not used by this tool");
    println!("codex native memories: not modified");
    Ok(())
}

fn import_source(store: &Store, source: ImportSource) -> Result<()> {
    store.ensure_layout()?;

    match source {
        ImportSource::ChatgptExport { zip } => {
            let summary = archive::inspect_chatgpt_export(&zip)
                .with_context(|| format!("failed to inspect {}", zip.display()))?;
            let zip_sha256 = store.encrypt_blob(&zip)?;
            let manifest = SourceManifest::new(
                ImportKind::ChatgptExport,
                "safe_zip_inspection",
                Coverage::Unverifiable,
            )
            .with_evidence_sha256(zip_sha256)
            .with_observed_items(summary.conversation_count.unwrap_or(summary.entries.len()));
            store.write_manifest(&manifest)?;
            store.append_records(summary.into_records(&manifest.id)?)?;
            println!("imported ChatGPT export manifest {}", manifest.id);
            println!(
                "wrote {}",
                report::write_run_log(store, &manifest)?.display()
            );
        }
        ImportSource::AtlasMemoryText { path } => {
            let text = read_text_or_stdin(path.as_ref())?;
            if text.trim().is_empty() {
                bail!("atlas memory text is empty");
            }
            let manifest = SourceManifest::new(
                ImportKind::AtlasMemoryText,
                "user_selected_text",
                Coverage::Unverifiable,
            )
            .with_evidence_sha256(store::sha256_hex(text.as_bytes()))
            .with_observed_items(text.lines().filter(|line| !line.trim().is_empty()).count());
            store.write_manifest(&manifest)?;
            store.append_records(types::records_from_atlas_text(&text, &manifest.id)?)?;
            println!("imported Atlas memory text manifest {}", manifest.id);
            println!(
                "wrote {}",
                report::write_run_log(store, &manifest)?.display()
            );
        }
        ImportSource::Bookmarks { html } => {
            let source_sha256 = store.encrypt_blob(&html)?;
            let text = fs::read_to_string(&html)
                .with_context(|| format!("failed to read {}", html.display()))?;
            let items = bookmarks::parse_bookmarks(&text)?;
            let manifest = SourceManifest::new(
                ImportKind::Bookmarks,
                "bookmarks_html",
                Coverage::Unverifiable,
            )
            .with_evidence_sha256(source_sha256)
            .with_observed_items(items.len());
            store.write_manifest(&manifest)?;
            store.append_records(types::records_from_bookmarks(items, &manifest.id)?)?;
            println!("imported bookmarks manifest {}", manifest.id);
            println!(
                "wrote {}",
                report::write_run_log(store, &manifest)?.display()
            );
        }
        ImportSource::Urls { path } => {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let source_sha256 = store.encrypt_blob(&path)?;
            let urls = types::parse_url_lines(&text)?;
            let manifest = SourceManifest::new(ImportKind::Urls, "url_lines", Coverage::Partial)
                .with_evidence_sha256(source_sha256)
                .with_observed_items(urls.len());
            store.write_manifest(&manifest)?;
            store.append_records(types::records_from_urls(urls, &manifest.id)?)?;
            println!("imported URL list manifest {}", manifest.id);
            println!(
                "wrote {}",
                report::write_run_log(store, &manifest)?.display()
            );
        }
    }

    let path = report::write_report(store)?;
    println!("updated {}", path.display());
    Ok(())
}

fn read_text_or_stdin(path: Option<&PathBuf>) -> Result<String> {
    match path {
        Some(path) => {
            fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
        }
        None => {
            let mut text = String::new();
            io::stdin().read_to_string(&mut text)?;
            Ok(text)
        }
    }
}
