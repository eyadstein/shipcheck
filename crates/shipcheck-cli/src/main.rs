use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use shipcheck_core::{score, Finding};
use shipcheck_engine::{scan, Catalog};

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Parser)]
#[command(
    name = "shipcheck",
    version,
    about = "Scan a codebase for legal, security and design problems"
)]
struct Cli {
    /// Path to the project to scan
    path: PathBuf,
    /// Directory containing rule files
    #[arg(long, default_value = "rules")]
    rules: PathBuf,
    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    /// Exit with code 1 when the score is below this value
    #[arg(long, default_value_t = 0)]
    fail_under: u32,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let catalog = match Catalog::load(&cli.rules) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };

    let findings = scan(&cli.path, &catalog);
    let total = score(&findings);
    match cli.format {
        Format::Text => print_text(&cli.path, &findings, total),
        Format::Json => print_json(&findings, total),
    }

    if total < cli.fail_under {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn print_text(path: &Path, findings: &[Finding], total: u32) {
    for item in findings {
        println!(
            "[{:?}] {} {}:{} {}",
            item.severity, item.rule_id, item.file, item.line, item.message
        );
        if let Some(fix) = &item.fix {
            println!("    fix: {fix}");
        }
    }
    println!(
        "Scanned {} | score {total}/100 | {} findings",
        path.display(),
        findings.len()
    );
}

fn print_json(findings: &[Finding], total: u32) {
    let report = serde_json::json!({ "score": total, "findings": findings });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("report serializes to JSON")
    );
}
