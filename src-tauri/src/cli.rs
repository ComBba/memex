//! `memex` CLI mode. Activates when the binary is invoked with a recognized
//! subcommand. Otherwise main.rs falls through to the Tauri GUI.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::{indexer, parser};

#[derive(Debug, Parser)]
#[command(name = "memex", version, about = "Time Machine for AI session JSONL")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Walk a `~/.claude/projects` root and print a one-line summary per session.
    Scan {
        /// Path to scan. Defaults to `~/.claude/projects`.
        #[arg(long)]
        path: Option<PathBuf>,
        /// Also index parsed sessions into Qdrant (creates collection if needed).
        #[arg(long)]
        index: bool,
        /// Cap the number of sessions printed.
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Vector search against the indexed `content` field.
    Search {
        /// Free-text query.
        query: String,
        /// Number of results.
        #[arg(long, default_value_t = 10)]
        limit: u64,
    },
    /// Snapshot management.
    Snapshot {
        #[command(subcommand)]
        op: SnapshotOp,
    },
}

#[derive(Debug, Subcommand)]
pub enum SnapshotOp {
    /// Export a snapshot of the current collection to `path`.
    Export { path: PathBuf },
    /// Restore a collection from a snapshot file.
    Import { path: PathBuf },
}

pub fn run(args: Vec<String>) -> Result<()> {
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
        Command::Scan { path, index, limit } => cmd_scan(path, index, limit),
        Command::Search { query, limit } => cmd_search(query, limit),
        Command::Snapshot { op } => cmd_snapshot(op),
    }
}

fn cmd_scan(path: Option<PathBuf>, index: bool, limit: Option<usize>) -> Result<()> {
    let root = path.unwrap_or_else(default_projects_root);
    eprintln!("scanning {}", root.display());
    let mut sessions = parser::scan_dir(&root)?;
    sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    let total = sessions.len();
    let to_show = limit.unwrap_or(total).min(total);

    println!(
        "{:<19} {:<24} {:<5} {:<5} {:<9} {:<11} {}",
        "start", "project", "user", "asst", "tools", "branch", "title"
    );
    println!("{}", "-".repeat(120));
    for s in sessions.iter().take(to_show) {
        println!("{}", parser::summary_line(s));
    }

    let tool_total: usize = sessions
        .iter()
        .flat_map(|s| s.turns.iter())
        .map(|t| t.tool_calls.len())
        .sum();
    eprintln!(
        "\nparsed {} session(s) (shown: {}), {} total tool calls",
        total, to_show, tool_total
    );

    if index {
        eprintln!("\nindexing into qdrant…");
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .context("building tokio runtime")?;
        rt.block_on(async {
            let client = indexer::connect().await?;
            indexer::ensure_collection(&client).await?;
            let embedder = indexer::Embedder::new()?;
            let ok = indexer::bulk_index(&client, &embedder, &sessions).await?;
            eprintln!("\nindexed {ok}/{} session(s) into '{}'", total, indexer::COLLECTION);
            anyhow::Ok(())
        })?;
    }
    Ok(())
}

fn cmd_search(query: String, limit: u64) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("building tokio runtime")?;
    rt.block_on(async {
        let client = indexer::connect().await?;
        let embedder = indexer::Embedder::new()?;
        let hits = indexer::search_content(&client, &embedder, &query, limit).await?;
        if hits.is_empty() {
            eprintln!("no results for {query:?}");
            return anyhow::Ok(());
        }
        println!(
            "{:<6} {:<19} {:<22} {:<40} session",
            "score", "start", "project", "title"
        );
        println!("{}", "-".repeat(120));
        for h in &hits {
            println!(
                "{:<6.4} {:<19} {:<22} {:<40} {}",
                h.score,
                h.start_iso
                    .get(..16)
                    .unwrap_or(&h.start_iso)
                    .replace('T', " "),
                truncate(&h.project_name, 22),
                truncate(if h.ai_title.is_empty() { "(untitled)" } else { &h.ai_title }, 40),
                h.session_id
            );
        }
        anyhow::Ok(())
    })
}

fn cmd_snapshot(op: SnapshotOp) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        match op {
            SnapshotOp::Export { path } => {
                let name = indexer::snapshot_export(&path).await?;
                eprintln!("snapshot '{name}' exported to {}", path.display());
            }
            SnapshotOp::Import { path } => {
                indexer::snapshot_import(&path).await?;
                eprintln!("snapshot imported from {}", path.display());
            }
        }
        anyhow::Ok(())
    })
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn default_projects_root() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".claude");
        p.push("projects");
        p
    } else {
        PathBuf::from(".claude/projects")
    }
}
