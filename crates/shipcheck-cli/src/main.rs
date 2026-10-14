use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use shipcheck_core::{score, Finding};
use shipcheck_engine::{
    grade_rules, overall_robustness, render_gym, review, scan, Baseline, Catalog, Error,
    ProjectConfig, Review,
};

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
    /// Exit with code 1 when the score is below this value (with --gym: the robustness)
    #[arg(long, default_value_t = 0)]
    fail_under: u32,
    /// Print the disclosure ledger (third parties, cookies, personal data) instead of scanning
    #[arg(long)]
    ledger: bool,
    /// Attack every rule with rewritten versions of its own examples and grade the result
    #[arg(long)]
    gym: bool,
    /// Hide the findings that are listed in this baseline file
    #[arg(long, value_name = "FILE")]
    baseline: Option<PathBuf>,
    /// Save the current findings as a baseline file instead of reporting them
    #[arg(long, value_name = "FILE")]
    write_baseline: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if cli.ledger {
        print_ledger(&cli.path, cli.format);
        return ExitCode::SUCCESS;
    }
    let catalog = match Catalog::load(&cli.rules) {
        Ok(catalog) => catalog,
        Err(error) => return fail(&error),
    };
    if cli.gym {
        return run_gym(&catalog, cli.fail_under);
    }
    match run_scan(&cli, &catalog) {
        Ok(code) => code,
        Err(error) => fail(&error),
    }
}

fn fail(error: &Error) -> ExitCode {
    eprintln!("error: {error}");
    ExitCode::from(2)
}

fn run_gym(catalog: &Catalog, minimum: u32) -> ExitCode {
    let grades = grade_rules(catalog);
    println!("{}", render_gym(&grades));
    if overall_robustness(&grades) < minimum {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run_scan(cli: &Cli, catalog: &Catalog) -> Result<ExitCode, Error> {
    let config = ProjectConfig::load(&cli.path)?;
    let baseline = cli.baseline.as_deref().map(Baseline::load).transpose()?;
    let found = scan(&cli.path, catalog);
    let reviewed = review(&cli.path, found, &config, baseline.as_ref());

    if let Some(target) = &cli.write_baseline {
        let saved = Baseline::from_findings(&cli.path, &reviewed.findings);
        saved.save(target)?;
        println!(
            "Baseline with {} findings written to {}",
            saved.accepted(),
            target.display()
        );
        return Ok(ExitCode::SUCCESS);
    }

    let total = score(&reviewed.findings);
    match cli.format {
        Format::Text => print_text(&cli.path, &reviewed, total),
        Format::Json => print_json(&reviewed, total),
    }
    let threshold = if cli.fail_under > 0 {
        cli.fail_under
    } else {
        config.fail_under.unwrap_or(0)
    };
    Ok(if total < threshold {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn print_finding(item: &Finding) {
    println!(
        "[{:?}] {} {}:{} {}",
        item.severity, item.rule_id, item.file, item.line, item.message
    );
    if let Some(fix) = &item.fix {
        println!("    fix: {fix}");
    }
}

fn print_text(path: &Path, reviewed: &Review, total: u32) {
    for item in &reviewed.findings {
        print_finding(item);
    }
    println!(
        "Scanned {} | score {total}/100 | {} findings",
        path.display(),
        reviewed.findings.len()
    );
    if reviewed.ignored + reviewed.suppressed + reviewed.baselined > 0 {
        println!(
            "Set aside: {} ignored by shipcheck.yml, {} silenced in code, {} already in the baseline",
            reviewed.ignored, reviewed.suppressed, reviewed.baselined
        );
    }
}

fn print_json(reviewed: &Review, total: u32) {
    let report = serde_json::json!({
        "score": total,
        "findings": reviewed.findings,
        "ignored": reviewed.ignored,
        "suppressed": reviewed.suppressed,
        "baselined": reviewed.baselined,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("report serializes to JSON")
    );
}

fn print_ledger(path: &Path, format: Format) {
    let analysis = shipcheck_engine::ledger(path);
    match format {
        Format::Text => {
            println!("{}", shipcheck_ledger::render_text(&analysis.ledger));
            if !analysis.findings.is_empty() {
                println!("\nDrift between the code and the policy");
                for item in &analysis.findings {
                    print_finding(item);
                }
            }
        }
        Format::Json => println!(
            "{}",
            serde_json::to_string_pretty(&analysis.ledger).expect("ledger serializes to JSON")
        ),
    }
}
