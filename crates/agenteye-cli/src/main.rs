//! Developer CLI that exercises each ingestion tier and reports timings.
//!
//! ```text
//! agenteye discover                       # tier 1: list sessions (metadata + head/tail)
//! agenteye open claude <path>             # tier 2: parse one session in memory
//! agenteye index --db agenteye.db         # tier 3: build or refresh the search index
//! agenteye search --db agenteye.db "text" # query the index
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use agenteye_core::AgentKind;
use agenteye_engine::Engine;
use agenteye_index::{Index, HIGHLIGHT_END, HIGHLIGHT_START};
use agenteye_sources::Catalog;
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Tier 1: list every session using file metadata plus head/tail reads.
    Discover,
    /// Tier 2: parse one session file in memory.
    Open { agent: AgentKind, path: PathBuf },
    /// Tier 3: build or incrementally refresh the full-text index.
    Index {
        #[arg(long, default_value = "agenteye-spike.db")]
        db: PathBuf,
    },
    /// Search the index.
    Search {
        #[arg(long, default_value = "agenteye-spike.db")]
        db: PathBuf,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        query: String,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Discover => {
            discover();
            Ok(())
        }
        Command::Open { agent, path } => open(agent, &path),
        Command::Index { db } => index(&db),
        Command::Search { db, limit, query } => search(&db, limit, &query),
    }
}

fn discover() {
    let catalog = Catalog::from_env();
    let started = Instant::now();
    let files = catalog.discover();
    let discovery = started.elapsed();

    let started = Instant::now();
    let sessions = catalog.list_sessions();
    let listing = started.elapsed();

    let total_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();
    println!(
        "found {} files ({}), discovery {}, discovery+summaries {}",
        files.len(),
        megabytes(total_bytes),
        millis(discovery),
        millis(listing),
    );
    for session in sessions.iter().take(10) {
        println!(
            "  [{}] {}  {}",
            session.agent,
            session.title.as_deref().unwrap_or("(untitled)"),
            session.path.display()
        );
    }
}

fn open(agent: AgentKind, path: &Path) -> Result<()> {
    let catalog = Catalog::from_env();
    let Some(source) = catalog.source(agent) else {
        bail!("no {agent} source is available on this machine");
    };

    let mut messages = 0_usize;
    let mut text_bytes = 0_usize;
    let started = Instant::now();
    let end = source
        .scan_messages(path, 0, &mut |record| {
            messages += 1;
            text_bytes += record.text.len();
        })
        .with_context(|| format!("reading {}", path.display()))?;
    let elapsed = started.elapsed();

    println!(
        "{messages} messages ({} text) from {} in {} ({})",
        megabytes(text_bytes as u64),
        megabytes(end),
        millis(elapsed),
        throughput(end, elapsed),
    );
    Ok(())
}

fn index(db: &Path) -> Result<()> {
    let engine = Engine::new(Catalog::from_env(), db)
        .with_context(|| format!("opening {}", db.display()))?;

    let started = Instant::now();
    let summary = engine.sync(|_| {})?;
    let elapsed = started.elapsed();

    println!(
        "indexed {} messages from {} files ({} failed), {} scanned in {} ({})",
        summary.messages_added,
        summary.files_total,
        summary.files_failed,
        megabytes(summary.bytes_scanned),
        millis(elapsed),
        throughput(summary.bytes_scanned, elapsed),
    );
    Ok(())
}

fn search(db: &Path, limit: usize, query: &str) -> Result<()> {
    // Read-only, so a mistyped --db path is an error instead of creating an empty database.
    let index = Index::open_read_only(db).with_context(|| format!("opening {}", db.display()))?;
    let started = Instant::now();
    let hits = index.search(query, limit)?;
    let elapsed = started.elapsed();

    println!("{} hits in {}", hits.len(), millis(elapsed));
    for hit in &hits {
        // Show hits as [word] in a terminal; the markers themselves are control characters.
        let snippet = hit.snippet.replace(HIGHLIGHT_START, "[").replace(HIGHLIGHT_END, "]");
        println!("  [{}] {}  @{}\n      {snippet}", hit.agent, hit.path.display(), hit.byte_offset);
    }
    Ok(())
}

/// Bytes as mebibytes for display. Precision loss above 2^52 bytes is irrelevant for a readout.
#[allow(clippy::cast_precision_loss)]
fn to_megabytes(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", to_megabytes(bytes))
}

fn millis(duration: Duration) -> String {
    format!("{:.1} ms", duration.as_secs_f64() * 1000.0)
}

fn throughput(bytes: u64, duration: Duration) -> String {
    let seconds = duration.as_secs_f64().max(f64::EPSILON);
    format!("{:.0} MB/s", to_megabytes(bytes) / seconds)
}
