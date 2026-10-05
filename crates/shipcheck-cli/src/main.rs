use std::path::PathBuf;

use clap::Parser;
use shipcheck_core::{score, Finding};

#[derive(Parser)]
#[command(
    name = "shipcheck",
    version,
    about = "Scan a codebase for legal, security and design problems"
)]
struct Cli {
    /// Path to the project to scan
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    // The real scan engine arrives in milestone 1.
    let findings: Vec<Finding> = Vec::new();
    println!(
        "Scanned {} | score {}/100 | {} findings",
        cli.path.display(),
        score(&findings),
        findings.len()
    );
}
