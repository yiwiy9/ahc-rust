use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, FixedOffset, Local, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "ahc",
    version,
    about = "AHC local workflow without cargo-compete"
)]
struct Cli {
    /// Contest ID or path. Usually inferred from the current directory.
    #[arg(long, global = true)]
    contest: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check the local toolchain and workspace configuration.
    Doctor,
    /// Create a contest workspace and optionally download official tools.
    New {
        contest_id: String,
        #[arg(long)]
        tools_url: Option<String>,
    },
    /// Download or replace the official tools for an existing contest.
    Tools {
        #[arg(long)]
        url: Option<String>,
    },
    /// Add a solver scaffold without modifying existing solvers.
    Add {
        #[arg(value_enum)]
        template: SolverTemplate,
    },
    /// Build a solver and the official tools.
    Build {
        #[arg(long, default_value = "a")]
        solver: String,
        #[arg(long)]
        debug: bool,
    },
    /// Run one generated seed, score it, and generate a visualization.
    Run {
        #[arg(default_value_t = 0)]
        seed: usize,
        #[arg(long, default_value = "a")]
        solver: String,
        #[arg(long)]
        debug: bool,
        #[arg(long)]
        no_vis: bool,
        /// Reuse an already-built solver and official tools.
        #[arg(long)]
        no_build: bool,
    },
    /// Run the same solver on seeds 0..cases and save normalized JSON results.
    Bench {
        #[arg(long, default_value = "a")]
        solver: String,
        #[arg(long, default_value_t = 10)]
        cases: usize,
        /// Use the simple sequential runner instead of workspace-local pahcer.
        #[arg(long)]
        builtin: bool,
        /// Parallel workers for pahcer. 0 uses its physical-core default.
        #[arg(long, default_value_t = 0)]
        threads: usize,
        /// Evaluate against the local history by rank score.
        #[arg(long)]
        rank: bool,
        /// Do not update pahcer's per-seed best scores.
        #[arg(long)]
        freeze_best_scores: bool,
        /// Short experiment note stored with the measurement.
        #[arg(short, long)]
        comment: Option<String>,
    },
    /// Benchmark and save the complete current src directory as a snapshot.
    Save {
        name: String,
        #[arg(long, default_value = "a")]
        solver: String,
        #[arg(long, default_value_t = 10)]
        cases: usize,
        /// Use the simple sequential runner instead of workspace-local pahcer.
        #[arg(long)]
        builtin: bool,
        /// Parallel workers for pahcer. 0 uses its physical-core default.
        #[arg(long, default_value_t = 0)]
        threads: usize,
    },
    /// Show pahcer's measurement history.
    History {
        #[arg(long)]
        rank: bool,
        #[arg(short, long, default_value_t = 10)]
        number: usize,
    },
    /// List saved snapshots.
    List,
    /// Restore a snapshot after creating an automatic backup.
    Restore { id: usize },
    /// Compare two snapshots by ID or name.
    Compare { left: String, right: String },
    /// Expand include! files into one self-contained Rust source file.
    Export {
        #[arg(long, default_value = "a")]
        solver: String,
        #[arg(long)]
        clipboard: bool,
    },
    /// Print or open the latest generated visualization for a seed.
    Vis {
        #[arg(default_value_t = 0)]
        seed: usize,
        #[arg(long, default_value = "a")]
        solver: String,
        #[arg(long)]
        debug: bool,
        #[arg(long)]
        open: bool,
    },
    /// Serve generated visualizations over localhost.
    Serve {
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
    /// Print or open the configured web visualizer/problem URL.
    Web {
        #[arg(long)]
        open: bool,
    },
    /// Search past contests, templates, and the optional common library clone.
    Search {
        query: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        lib: bool,
        #[arg(long)]
        contests_only: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SolverTemplate {
    RandomSearch,
    Beam,
    BeamFast,
    LocalSearchDirect,
    LocalSearchRebuild,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceConfig {
    #[serde(default = "default_search_roots")]
    search_roots: Vec<String>,
    #[serde(default)]
    common_library_path: Option<String>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            search_roots: default_search_roots(),
            common_library_path: Some("../atcoder-lib".to_string()),
        }
    }
}

fn default_search_roots() -> Vec<String> {
    vec!["contests".to_string(), "templates".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContestConfig {
    contest_id: String,
    #[serde(default = "default_score_direction")]
    score_direction: ScoreDirection,
    #[serde(default = "default_score_pattern")]
    score_pattern: String,
    #[serde(default = "default_pahcer_score_pattern")]
    pahcer_score_pattern: String,
    #[serde(default)]
    web_visualizer_url: Option<String>,
    #[serde(default)]
    total_time_limit_seconds: Option<f64>,
}

fn default_score_direction() -> ScoreDirection {
    ScoreDirection::Maximize
}

fn default_score_pattern() -> String {
    r"Score\s*=\s*(-?[0-9]+)".to_string()
}

fn default_pahcer_score_pattern() -> String {
    r"(?m)^\s*Score\s*=\s*(?P<score>\d+)\s*$".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ScoreDirection {
    Maximize,
    Minimize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseResult {
    seed: usize,
    score: i64,
    elapsed_milliseconds: u128,
    output_path: String,
    log_path: String,
    visualization_path: Option<String>,
    #[serde(default)]
    relative_score: Option<f64>,
    #[serde(default)]
    error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkResult {
    schema_version: u32,
    contest: String,
    solver: String,
    created_at: String,
    cases: usize,
    total_score: i128,
    average_score: f64,
    minimum_score: i64,
    maximum_score: i64,
    wall_time_milliseconds: u128,
    case_results: Vec<CaseResult>,
    #[serde(default = "default_benchmark_runner")]
    runner: String,
    #[serde(default)]
    score_mode: String,
    #[serde(default)]
    threads: usize,
    #[serde(default)]
    accepted_cases: usize,
    #[serde(default)]
    average_relative_score: Option<f64>,
    #[serde(default)]
    maximum_execution_milliseconds: Option<u128>,
    #[serde(default)]
    comment: String,
    #[serde(default)]
    source_result_path: Option<String>,
}

fn default_benchmark_runner() -> String {
    "builtin".to_string()
}

#[derive(Debug, Deserialize)]
struct PahcerResult {
    start_time: String,
    case_count: usize,
    total_score: i128,
    total_relative_score: f64,
    max_execution_time: f64,
    comment: String,
    wa_seeds: Vec<usize>,
    cases: Vec<PahcerCaseResult>,
}

#[derive(Debug, Deserialize)]
struct PahcerCaseResult {
    seed: usize,
    score: i64,
    relative_score: f64,
    execution_time: f64,
    error_message: String,
}

#[derive(Debug, Serialize)]
struct PahcerConfigFile {
    general: PahcerGeneralConfig,
    problem: PahcerProblemConfig,
    test: PahcerTestConfig,
}

#[derive(Debug, Serialize)]
struct PahcerGeneralConfig {
    version: String,
}

#[derive(Debug, Serialize)]
struct PahcerProblemConfig {
    problem_name: String,
    objective: String,
    score_regex: String,
}

#[derive(Debug, Serialize)]
struct PahcerTestConfig {
    start_seed: usize,
    end_seed: usize,
    threads: usize,
    out_dir: String,
    compile_steps: Vec<PahcerCompileStep>,
    test_steps: Vec<PahcerTestStep>,
}

#[derive(Debug, Serialize)]
struct PahcerCompileStep {
    program: String,
    args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PahcerTestStep {
    program: String,
    args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<String>,
    measure_time: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotEntry {
    id: usize,
    name: String,
    solver: String,
    created_at: String,
    result_path: String,
    source_path: String,
    summary: BenchmarkResult,
}

#[derive(Debug, Serialize)]
struct Event<'a, T: Serialize> {
    schema_version: u32,
    event_type: &'a str,
    occurred_at: String,
    payload: T,
}

#[derive(Debug)]
struct BuiltTools {
    solver: PathBuf,
    generator: Option<PathBuf>,
    tester: Option<PathBuf>,
    visualizer: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = workspace_root()?;

    match cli.command {
        Commands::Doctor => doctor(&root, cli.contest.as_deref()),
        Commands::New {
            contest_id,
            tools_url,
        } => new_contest(&root, &contest_id, tools_url.as_deref()),
        Commands::Tools { url } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            let config = load_contest_config(&contest)?;
            install_tools_for_contest(&contest, &config.contest_id, url.as_deref())
        }
        Commands::Add { template } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            add_solver(&root, &contest, template)
        }
        Commands::Build { solver, debug } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            let config = load_contest_config(&contest)?;
            build(&contest, &config, &solver, debug).map(|_| ())
        }
        Commands::Run {
            seed,
            solver,
            debug,
            no_vis,
            no_build,
        } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            let config = load_contest_config(&contest)?;
            let result = run_case(&contest, &config, &solver, seed, debug, !no_vis, !no_build)?;
            print_case_result(&result);
            Ok(())
        }
        Commands::Bench {
            solver,
            cases,
            builtin,
            threads,
            rank,
            freeze_best_scores,
            comment,
        } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            let config = load_contest_config(&contest)?;
            let result = benchmark(
                &root,
                &contest,
                &config,
                &solver,
                cases,
                builtin,
                threads,
                rank,
                freeze_best_scores,
                comment.as_deref().unwrap_or(&solver),
            )?;
            print_benchmark(&result);
            Ok(())
        }
        Commands::Save {
            name,
            solver,
            cases,
            builtin,
            threads,
        } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            let config = load_contest_config(&contest)?;
            save_snapshot(
                &root, &contest, &config, &name, &solver, cases, builtin, threads,
            )
        }
        Commands::History { rank, number } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            show_pahcer_history(&root, &contest, rank, number)
        }
        Commands::List => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            list_snapshots(&contest)
        }
        Commands::Restore { id } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            restore_snapshot(&contest, id)
        }
        Commands::Compare { left, right } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            compare_snapshots(&contest, &left, &right)
        }
        Commands::Export { solver, clipboard } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            export_solver(&contest, &solver, clipboard)
        }
        Commands::Vis {
            seed,
            solver,
            debug,
            open,
        } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            show_visualization(&contest, &solver, seed, debug, open)
        }
        Commands::Serve { port } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            serve_visualizations(&contest, port)
        }
        Commands::Web { open } => {
            let contest = resolve_contest(&root, cli.contest.as_deref())?;
            show_web_url(&contest, open)
        }
        Commands::Search {
            query,
            tag,
            lib,
            contests_only,
        } => search(&root, query.as_deref(), tag.as_deref(), lib, contests_only),
    }
}

fn workspace_root() -> Result<PathBuf> {
    if let Ok(current) = env::current_dir() {
        if let Some(root) = find_workspace_root(&current) {
            return root
                .canonicalize()
                .context("failed to resolve AHC workspace root");
        }
    }

    if let Ok(executable) = env::current_exe() {
        if let Some(parent) = executable.parent() {
            if let Some(root) = find_workspace_root(parent) {
                return root
                    .canonicalize()
                    .context("failed to resolve AHC workspace root");
            }
        }
    }

    bail!("AHC workspace marker (.ahc-root) was not found")
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| path.join(".ahc-root").is_file())
        .map(Path::to_path_buf)
}

fn workspace_config(root: &Path) -> Result<WorkspaceConfig> {
    let path = root.join("workspace.toml");
    if !path.is_file() {
        return Ok(WorkspaceConfig::default());
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

fn load_contest_config(contest: &Path) -> Result<ContestConfig> {
    let path = contest.join("ahc.toml");
    let text = fs::read_to_string(&path)
        .with_context(|| format!("contest config not found: {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

fn resolve_contest(root: &Path, requested: Option<&str>) -> Result<PathBuf> {
    if let Some(value) = requested {
        let direct = PathBuf::from(value);
        let candidates = [direct.clone(), root.join("contests").join(value)];
        for candidate in candidates {
            if candidate.join("ahc.toml").is_file() {
                return candidate
                    .canonicalize()
                    .with_context(|| format!("failed to resolve {}", candidate.display()));
            }
        }
        bail!("contest was not found: {value}");
    }

    let mut current = env::current_dir()?;
    loop {
        if current.join("ahc.toml").is_file() {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }
    bail!("run this command inside a contest directory or pass --contest <id>")
}

fn doctor(root: &Path, requested_contest: Option<&str>) -> Result<()> {
    println!("AHC workspace: {}", root.display());
    println!();
    println!("Required commands:");
    let required = ["cargo", "rustc", "curl", "unzip", "rg"];
    let mut missing = Vec::new();
    for command in required {
        if !command_exists(command) {
            println!("  [missing] {command}");
            missing.push(command);
        } else {
            let version = command_version(command).unwrap_or_else(|| "installed".to_string());
            println!("  [ok] {command}: {version}");
        }
    }

    println!();
    println!("Optional commands:");
    for command in ["pbcopy", "python3", "docker"] {
        if command_exists(command) {
            println!("  [ok] {command}");
        } else {
            println!("  [not found] {command}");
        }
    }
    let pahcer = pahcer_binary(root);
    if pahcer.is_file() {
        let version = command_version_at(&pahcer).unwrap_or_else(|| "installed".to_string());
        println!("  [ok] pahcer: {version} (workspace-local)");
    } else {
        println!("  [not found] pahcer: run ./scripts/install-pahcer.sh");
    }

    let config = workspace_config(root)?;
    println!();
    println!("Search roots:");
    for value in &config.search_roots {
        let path = root.join(value);
        print_path_status(&path);
    }
    if let Some(value) = &config.common_library_path {
        let path = root.join(value);
        println!("Common library (optional):");
        print_path_status(&path);
        if path.join(".git").exists() {
            let branch = git_output(&path, ["branch", "--show-current"])
                .unwrap_or_else(|| "unknown".to_string());
            let revision = git_output(&path, ["rev-parse", "--short", "HEAD"])
                .unwrap_or_else(|| "unknown".to_string());
            let dirty = git_output(&path, ["status", "--short"])
                .map(|output| if output.is_empty() { "clean" } else { "dirty" })
                .unwrap_or("unknown");
            println!("    branch={branch} revision={revision} status={dirty}");
        }
    }

    if let Ok(contest) = resolve_contest(root, requested_contest) {
        println!();
        println!("Contest: {}", contest.display());
        print_path_status(&contest.join("Cargo.toml"));
        print_path_status(&contest.join("tools/Cargo.toml"));
        print_path_status(&contest.join("tools/in/0000.txt"));
    }

    if missing.is_empty() {
        println!();
        println!("Ready.");
        Ok(())
    } else {
        bail!("missing required commands: {}", missing.join(", "))
    }
}

fn command_version(command: &str) -> Option<String> {
    command_version_at(Path::new(command))
}

fn command_version_at(command: &Path) -> Option<String> {
    let output = Command::new(command).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(
        text.lines()
            .next()
            .unwrap_or("installed")
            .trim()
            .to_string(),
    )
}

fn pahcer_binary(root: &Path) -> PathBuf {
    root.join(".tools/bin/pahcer")
}

fn command_exists(command: &str) -> bool {
    env::var_os("PATH").is_some_and(|paths| {
        env::split_paths(&paths).any(|directory| directory.join(command).is_file())
    })
}

fn print_path_status(path: &Path) {
    let marker = if path.exists() { "ok" } else { "not found" };
    println!("  [{marker}] {}", path.display());
}

fn git_output<const N: usize>(path: &Path, args: [&str; N]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn new_contest(root: &Path, contest_id: &str, tools_url: Option<&str>) -> Result<()> {
    validate_contest_id(contest_id)?;
    let destination = root.join("contests").join(contest_id);
    if destination.exists() {
        bail!("contest already exists: {}", destination.display());
    }

    let url = match tools_url {
        Some(url) => url.to_string(),
        None if !contest_has_started(contest_id)? => {
            bail!(
                "contest {contest_id} has not started, so no contest workspace was created.\nRun ./ahc new {contest_id} after the contest begins."
            );
        }
        None => match discover_tools_url(contest_id) {
            Ok(url) => url,
            Err(error) => {
                create_contest_scaffold(root, contest_id, &destination)?;
                eprintln!("Official tools URL was not found automatically: {error:#}");
                eprintln!("Created the contest workspace without official tools.");
                eprintln!("Copy the official tools.zip URL from the problem page, then run:");
                eprintln!("  ./ahc tools --url <URL>");
                println!("Created: {}", destination.display());
                println!("Next: ./ahc tools --url <URL>");
                return Ok(());
            }
        },
    };

    create_contest_scaffold(root, contest_id, &destination)?;
    if let Err(error) = install_official_tools(&destination, &url) {
        fs::remove_dir_all(&destination).ok();
        return Err(error)
            .context("official tools could not be installed; contest workspace was removed");
    }
    println!("Official tools installed from {url}");

    println!("Created: {}", destination.display());
    println!("Next: cd contests/{contest_id} && ./ahc doctor");
    Ok(())
}

fn create_contest_scaffold(root: &Path, contest_id: &str, destination: &Path) -> Result<()> {
    let template = root.join("templates/contest");
    copy_tree(&template, destination)?;
    replace_in_tree(destination, "__CONTEST_ID__", contest_id)
}

fn install_tools_for_contest(contest: &Path, contest_id: &str, url: Option<&str>) -> Result<()> {
    let url = match url {
        Some(value) => value.to_string(),
        None => discover_tools_url(contest_id)?,
    };
    let destination = contest.join("tools");
    let backup = if destination.exists() {
        let backup = contest.join(format!(
            "results/tools-backup-{}",
            Local::now().format("%Y%m%d-%H%M%S")
        ));
        fs::create_dir_all(contest.join("results"))?;
        fs::rename(&destination, &backup)?;
        println!("Previous tools backed up to: {}", backup.display());
        Some(backup)
    } else {
        None
    };

    if let Err(error) = install_official_tools(contest, &url) {
        if destination.exists() {
            fs::remove_dir_all(&destination).ok();
        }
        if let Some(backup) = backup {
            fs::rename(&backup, &destination)
                .context("tool installation failed and the previous tools could not be restored")?;
            eprintln!("Tool installation failed; previous tools were restored.");
        }
        return Err(error);
    }
    println!("Official tools installed from {url}");
    Ok(())
}

fn validate_contest_id(value: &str) -> Result<()> {
    let valid = Regex::new(r"^[a-z0-9][a-z0-9_-]*$").unwrap();
    if valid.is_match(value) {
        Ok(())
    } else {
        bail!("contest ID must use lowercase letters, digits, '_' or '-': {value}")
    }
}

fn discover_tools_url(contest_id: &str) -> Result<String> {
    let task_url = format!(
        "https://atcoder.jp/contests/{0}/tasks/{0}_a?lang=ja",
        contest_id
    );
    let output = Command::new("curl")
        .args(["-fsSL", &task_url])
        .output()
        .with_context(|| format!("failed to fetch {task_url}"))?;
    if !output.status.success() {
        bail!("AtCoder returned an error for {task_url}");
    }
    let html = String::from_utf8_lossy(&output.stdout).replace("&amp;", "&");
    select_tools_url(&html).ok_or_else(|| anyhow!("tools.zip link was not found on {task_url}"))
}

fn contest_has_started(contest_id: &str) -> Result<bool> {
    let contest_url = format!("https://atcoder.jp/contests/{contest_id}?lang=en");
    let output = Command::new("curl")
        .args(["-fsSL", &contest_url])
        .output()
        .with_context(|| format!("failed to fetch {contest_url}"))?;
    if !output.status.success() {
        bail!("AtCoder returned an error for {contest_url}");
    }
    let html = String::from_utf8_lossy(&output.stdout);
    let start = select_contest_start_time(&html)
        .ok_or_else(|| anyhow!("contest start time was not found on {contest_url}"))?;
    Ok(Utc::now() >= start.with_timezone(&Utc))
}

fn select_contest_start_time(html: &str) -> Option<DateTime<FixedOffset>> {
    let regex = Regex::new(r#"var\s+startTime\s*=\s*moment\(\"([^\"]+)\"\)"#).unwrap();
    let value = regex.captures(html)?.get(1)?.as_str();
    DateTime::parse_from_rfc3339(value).ok()
}

fn select_tools_url(html: &str) -> Option<String> {
    let regex = Regex::new(r#"https://img\.atcoder\.jp/[^\"'<> ]+\.zip"#).unwrap();
    let urls = regex
        .find_iter(html)
        .map(|matched| matched.as_str().to_string())
        .collect::<Vec<_>>();
    urls.iter()
        .find(|url| url.to_ascii_lowercase().contains("tool"))
        .or_else(|| {
            urls.iter()
                .find(|url| !url.to_ascii_lowercase().contains("windows"))
        })
        .cloned()
}

fn install_official_tools(contest: &Path, url: &str) -> Result<()> {
    let temp_root = env::temp_dir().join(format!(
        "ahc-tools-{}-{}",
        std::process::id(),
        Local::now().timestamp_millis()
    ));
    fs::create_dir_all(&temp_root)?;
    let zip_path = temp_root.join("tools.zip");
    let extract_path = temp_root.join("extract");
    fs::create_dir_all(&extract_path)?;

    let status = Command::new("curl")
        .args(["-fL", url, "-o"])
        .arg(&zip_path)
        .status()
        .context("failed to start curl")?;
    if !status.success() {
        bail!("failed to download {url}");
    }

    let status = Command::new("unzip")
        .arg("-q")
        .arg(&zip_path)
        .arg("-d")
        .arg(&extract_path)
        .status()
        .context("failed to start unzip")?;
    if !status.success() {
        bail!("failed to extract {}", zip_path.display());
    }

    let source = locate_tools_root(&extract_path)?;
    let destination = contest.join("tools");
    copy_tree(&source, &destination)?;
    ensure_standalone_tools_manifest(&destination)?;
    fs::remove_dir_all(&temp_root).ok();
    Ok(())
}

fn ensure_standalone_tools_manifest(tools: &Path) -> Result<()> {
    let manifest = tools.join("Cargo.toml");
    let mut text = fs::read_to_string(&manifest)
        .with_context(|| format!("failed to read {}", manifest.display()))?;
    let workspace_header = Regex::new(r"(?m)^\s*\[workspace\]\s*$").unwrap();
    if !workspace_header.is_match(&text) {
        text.push_str(
            "\n# Keep official tools independent from the contest workspace.\n[workspace]\n",
        );
        fs::write(&manifest, text)?;
    }
    Ok(())
}

fn locate_tools_root(extract_path: &Path) -> Result<PathBuf> {
    if extract_path.join("Cargo.toml").is_file() {
        return Ok(extract_path.to_path_buf());
    }
    let directories = fs::read_dir(extract_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();
    if directories.len() == 1 && directories[0].path().join("Cargo.toml").is_file() {
        return Ok(directories[0].path());
    }
    bail!("could not locate tools/Cargo.toml in the downloaded archive")
}

fn add_solver(root: &Path, contest: &Path, template: SolverTemplate) -> Result<()> {
    let mappings: &[(&str, &str)] = match template {
        SolverTemplate::RandomSearch => &[
            (
                "framework/random_search.rs",
                "src/framework/random_search.rs",
            ),
            ("solvers/random_search.rs", "src/bin/random_search.rs"),
        ],
        SolverTemplate::Beam => &[
            ("framework/beam.rs", "src/framework/beam.rs"),
            ("solvers/beam.rs", "src/bin/beam.rs"),
        ],
        SolverTemplate::BeamFast => &[
            ("framework/beam_fast.rs", "src/framework/beam_fast.rs"),
            ("solvers/beam_fast.rs", "src/bin/beam_fast.rs"),
        ],
        SolverTemplate::LocalSearchDirect => &[
            ("framework/local_search.rs", "src/framework/local_search.rs"),
            (
                "states/local_search_direct.rs",
                "src/local_search_state_direct.rs",
            ),
            (
                "solvers/local_search_direct.rs",
                "src/bin/local_search_direct.rs",
            ),
        ],
        SolverTemplate::LocalSearchRebuild => &[
            ("framework/local_search.rs", "src/framework/local_search.rs"),
            (
                "states/local_search_rebuild.rs",
                "src/local_search_state_rebuild.rs",
            ),
            (
                "solvers/local_search_rebuild.rs",
                "src/bin/local_search_rebuild.rs",
            ),
        ],
    };

    for (source, destination) in mappings {
        let source = root.join("templates").join(source);
        let destination = contest.join(destination);
        if destination.exists() {
            println!("Kept existing: {}", destination.display());
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, &destination).with_context(|| {
            format!(
                "failed to copy template {} to {}",
                source.display(),
                destination.display()
            )
        })?;
        println!("Added: {}", destination.display());
    }
    println!("Existing solver files were not modified.");
    Ok(())
}

fn build(contest: &Path, _config: &ContestConfig, solver: &str, debug: bool) -> Result<BuiltTools> {
    let profile = if debug { "debug" } else { "release" };
    let mut solver_build = Command::new("cargo");
    solver_build
        .arg("build")
        .arg("--manifest-path")
        .arg(contest.join("Cargo.toml"))
        .arg("--bin")
        .arg(solver)
        .env("CARGO_TARGET_DIR", contest.join("target"));
    if !debug {
        solver_build.arg("--release");
    }
    run_checked(&mut solver_build, "solver build failed")?;

    let solver_path = contest.join("target").join(profile).join(solver);
    let tools_manifest = contest.join("tools/Cargo.toml");
    if !tools_manifest.is_file() {
        bail!(
            "official tools were not found: {}",
            tools_manifest.display()
        );
    }

    let tools_target = contest.join("tools/target");
    let mut tools_build = Command::new("cargo");
    tools_build
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(&tools_manifest)
        .arg("--bins")
        .env("CARGO_TARGET_DIR", &tools_target);
    run_checked(&mut tools_build, "official tools build failed")?;

    let release = tools_target.join("release");
    let built = BuiltTools {
        solver: solver_path,
        generator: executable_if_exists(release.join("gen")),
        tester: executable_if_exists(release.join("tester")),
        visualizer: executable_if_exists(release.join("vis")),
    };
    ensure_inputs(contest, &built)?;
    Ok(built)
}

fn run_checked(command: &mut Command, message: &str) -> Result<()> {
    let status = command.status().with_context(|| message.to_string())?;
    if status.success() {
        Ok(())
    } else {
        bail!("{message}: {status}")
    }
}

fn executable_if_exists(path: PathBuf) -> Option<PathBuf> {
    path.is_file().then_some(path)
}

fn ensure_inputs(contest: &Path, tools: &BuiltTools) -> Result<()> {
    let first_input = contest.join("tools/in/0000.txt");
    if first_input.is_file() {
        return Ok(());
    }
    let Some(generator) = &tools.generator else {
        bail!("official input 0000.txt is missing and no gen binary was found");
    };
    let seeds = contest.join("tools/seeds.txt");
    if !seeds.is_file() {
        bail!("official tools/seeds.txt was not found");
    }
    let input_dir = contest.join("tools/in");
    fs::create_dir_all(&input_dir)?;
    let mut command = Command::new(generator);
    command.arg(&seeds).arg("--dir").arg(&input_dir);
    run_checked(&mut command, "official input generation failed")
}

fn run_case(
    contest: &Path,
    config: &ContestConfig,
    solver: &str,
    seed: usize,
    debug: bool,
    create_visualization: bool,
    rebuild: bool,
) -> Result<CaseResult> {
    let tools = if rebuild {
        build(contest, config, solver, debug)?
    } else {
        locate_built_tools(contest, solver, debug)?
    };
    run_case_with_tools(
        contest,
        config,
        solver,
        seed,
        debug,
        create_visualization,
        &tools,
    )
}

fn locate_built_tools(contest: &Path, solver: &str, debug: bool) -> Result<BuiltTools> {
    let solver_path = contest
        .join("target")
        .join(if debug { "debug" } else { "release" })
        .join(solver);
    if !solver_path.is_file() {
        bail!("solver is not built: {}", solver_path.display());
    }
    let release = contest.join("tools/target/release");
    Ok(BuiltTools {
        solver: solver_path,
        generator: executable_if_exists(release.join("gen")),
        tester: executable_if_exists(release.join("tester")),
        visualizer: executable_if_exists(release.join("vis")),
    })
}

fn run_case_with_tools(
    contest: &Path,
    config: &ContestConfig,
    solver: &str,
    seed: usize,
    debug: bool,
    create_visualization: bool,
    tools: &BuiltTools,
) -> Result<CaseResult> {
    let seed_id = format!("{seed:04}");
    let input = contest.join(format!("tools/in/{seed_id}.txt"));
    if !input.is_file() {
        bail!("input was not found: {}", input.display());
    }

    let mode = if debug { "debug" } else { "release" };
    let output = contest.join(format!("out/{solver}/{mode}/{seed_id}.txt"));
    let log = contest.join(format!("results/logs/{solver}/{mode}/{seed_id}.log"));
    create_parent(&output)?;
    create_parent(&log)?;

    let input_file = File::open(&input)?;
    let output_file = File::create(&output)?;
    let log_file = File::create(&log)?;
    let started = Instant::now();

    let mut command = if let Some(tester) = &tools.tester {
        let mut command = Command::new(tester);
        command.arg(&tools.solver);
        command
    } else {
        Command::new(&tools.solver)
    };
    command
        .stdin(Stdio::from(input_file))
        .stdout(Stdio::from(output_file))
        .stderr(Stdio::from(log_file))
        .env("RUST_BACKTRACE", if debug { "1" } else { "0" });
    let status = command.status().context("solver execution failed")?;
    let elapsed = started.elapsed().as_millis();
    if !status.success() {
        let details = fs::read_to_string(&log).unwrap_or_default();
        bail!("solver failed for seed {seed_id}: {status}\n{details}");
    }

    let (score, visualization_path) = score_and_visualize(
        contest,
        config,
        tools.visualizer.as_deref(),
        &input,
        &output,
        solver,
        mode,
        &seed_id,
        create_visualization,
        &log,
    )?;

    Ok(CaseResult {
        seed,
        score,
        elapsed_milliseconds: elapsed,
        output_path: relative_display(contest, &output),
        log_path: relative_display(contest, &log),
        visualization_path,
        relative_score: None,
        error_message: String::new(),
    })
}

#[allow(clippy::too_many_arguments)]
fn score_and_visualize(
    contest: &Path,
    config: &ContestConfig,
    visualizer: Option<&Path>,
    input: &Path,
    output: &Path,
    solver: &str,
    mode: &str,
    seed_id: &str,
    create_visualization: bool,
    log: &Path,
) -> Result<(i64, Option<String>)> {
    let score_text;
    let mut visualization_path = None;
    if let Some(visualizer) = visualizer {
        let work = contest.join(format!("results/.vis-work/{solver}/{mode}/{seed_id}"));
        fs::create_dir_all(&work)?;
        let command_output = Command::new(visualizer)
            .arg(input)
            .arg(output)
            .current_dir(&work)
            .output()
            .context("official visualizer failed to start")?;
        if !command_output.status.success() {
            bail!(
                "official visualizer failed: {}",
                String::from_utf8_lossy(&command_output.stderr)
            );
        }
        score_text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&command_output.stdout),
            String::from_utf8_lossy(&command_output.stderr)
        );
        let generated = work.join("vis.html");
        if create_visualization && generated.is_file() {
            let destination =
                contest.join(format!("visualizations/{solver}/{mode}/{seed_id}.html"));
            create_parent(&destination)?;
            fs::copy(&generated, &destination)?;
            visualization_path = Some(relative_display(contest, &destination));
        }
    } else {
        score_text = fs::read_to_string(log).unwrap_or_default();
    }

    let regex = Regex::new(&config.score_pattern)
        .with_context(|| format!("invalid score_pattern: {}", config.score_pattern))?;
    let captures = regex
        .captures(&score_text)
        .ok_or_else(|| anyhow!("score was not found in official tool output:\n{score_text}"))?;
    let score = captures[1]
        .parse::<i64>()
        .context("official score was not an integer")?;
    Ok((score, visualization_path))
}

#[allow(clippy::too_many_arguments)]
fn benchmark(
    root: &Path,
    contest: &Path,
    config: &ContestConfig,
    solver: &str,
    cases: usize,
    builtin: bool,
    threads: usize,
    rank: bool,
    freeze_best_scores: bool,
    comment: &str,
) -> Result<BenchmarkResult> {
    if cases == 0 {
        bail!("cases must be positive");
    }
    validate_solver_name(solver)?;
    if builtin {
        benchmark_builtin(contest, config, solver, cases, comment)
    } else {
        benchmark_pahcer(
            root,
            contest,
            config,
            solver,
            cases,
            threads,
            rank,
            freeze_best_scores,
            comment,
        )
    }
}

fn benchmark_builtin(
    contest: &Path,
    config: &ContestConfig,
    solver: &str,
    cases: usize,
    comment: &str,
) -> Result<BenchmarkResult> {
    let tools = build(contest, config, solver, false)?;
    let started = Instant::now();
    let mut results = Vec::with_capacity(cases);
    for seed in 0..cases {
        let result = run_case_with_tools(contest, config, solver, seed, false, false, &tools)?;
        println!(
            "[{current:>3}/{cases:>3}] seed={seed:04} score={score} time={time}ms",
            current = seed + 1,
            score = result.score,
            time = result.elapsed_milliseconds
        );
        results.push(result);
    }
    let total_score = results.iter().map(|result| result.score as i128).sum();
    let minimum_score = results.iter().map(|result| result.score).min().unwrap();
    let maximum_score = results.iter().map(|result| result.score).max().unwrap();
    let result = BenchmarkResult {
        schema_version: 2,
        contest: config.contest_id.clone(),
        solver: solver.to_string(),
        created_at: now(),
        cases,
        total_score,
        average_score: total_score as f64 / cases as f64,
        minimum_score,
        maximum_score,
        wall_time_milliseconds: started.elapsed().as_millis(),
        case_results: results,
        runner: "builtin".to_string(),
        score_mode: "raw".to_string(),
        threads: 1,
        accepted_cases: cases,
        average_relative_score: None,
        maximum_execution_milliseconds: None,
        comment: comment.to_string(),
        source_result_path: None,
    };
    save_benchmark(contest, &result)?;
    append_event(contest, "benchmark_completed", &result)?;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn benchmark_pahcer(
    root: &Path,
    contest: &Path,
    config: &ContestConfig,
    solver: &str,
    cases: usize,
    threads: usize,
    rank: bool,
    freeze_best_scores: bool,
    comment: &str,
) -> Result<BenchmarkResult> {
    let pahcer = pahcer_binary(root);
    if !pahcer.is_file() {
        bail!(
            "workspace-local pahcer was not found\nRun: {}/scripts/install-pahcer.sh\nOr use: ./ahc bench --builtin",
            root.display()
        );
    }

    let tools = build(contest, config, solver, false)?;
    if tools.tester.is_none() && tools.visualizer.is_none() {
        bail!("no official tester or visualizer was found; use ./ahc bench --builtin");
    }

    let pahcer_root = contest.join("results/pahcer");
    let work_root = pahcer_root.join(format!("work/{solver}"));
    if work_root.is_dir() {
        fs::remove_dir_all(&work_root)?;
    }
    fs::create_dir_all(work_root.join("out"))?;
    fs::create_dir_all(work_root.join("log"))?;
    let config_path = pahcer_root.join("config.toml");
    write_pahcer_config(
        contest,
        config,
        solver,
        cases,
        threads,
        &tools,
        &config_path,
    )?;

    let started = Instant::now();
    let mut command = Command::new(&pahcer);
    command
        .arg("run")
        .arg("--no-compile")
        .arg("--setting-file")
        .arg(&config_path)
        .arg("--comment")
        .arg(comment)
        .current_dir(contest);
    if rank {
        command.arg("--rank");
    }
    if freeze_best_scores {
        command.arg("--freeze-best-scores");
    }
    run_checked(&mut command, "pahcer benchmark failed")?;
    let wall_time_milliseconds = started.elapsed().as_millis();

    let source_path = latest_pahcer_result(&pahcer_root.join("json"))?;
    let source_text = fs::read_to_string(&source_path)?;
    let source: PahcerResult = serde_json::from_str(&source_text)
        .with_context(|| format!("failed to parse {}", source_path.display()))?;
    if source.case_count == 0 || source.cases.is_empty() {
        bail!("pahcer returned no cases");
    }

    let case_results = source
        .cases
        .into_iter()
        .map(|case| {
            let visualization = contest.join(format!(
                "results/pahcer/work/{solver}/case/{:04}/vis.html",
                case.seed
            ));
            CaseResult {
                seed: case.seed,
                score: case.score,
                elapsed_milliseconds: seconds_to_milliseconds(case.execution_time),
                output_path: format!("results/pahcer/work/{solver}/out/{:04}.txt", case.seed),
                log_path: format!("results/pahcer/work/{solver}/log/{:04}.txt", case.seed),
                visualization_path: visualization
                    .is_file()
                    .then(|| relative_display(contest, &visualization)),
                relative_score: Some(case.relative_score),
                error_message: case.error_message,
            }
        })
        .collect::<Vec<_>>();
    let minimum_score = case_results.iter().map(|case| case.score).min().unwrap();
    let maximum_score = case_results.iter().map(|case| case.score).max().unwrap();
    let average_score = source.total_score as f64 / source.case_count as f64;
    let result = BenchmarkResult {
        schema_version: 2,
        contest: config.contest_id.clone(),
        solver: solver.to_string(),
        created_at: source.start_time,
        cases: source.case_count,
        total_score: source.total_score,
        average_score,
        minimum_score,
        maximum_score,
        wall_time_milliseconds,
        case_results,
        runner: "pahcer".to_string(),
        score_mode: if rank { "rank" } else { "relative" }.to_string(),
        threads,
        accepted_cases: source.case_count.saturating_sub(source.wa_seeds.len()),
        average_relative_score: Some(source.total_relative_score / source.case_count as f64),
        maximum_execution_milliseconds: Some(seconds_to_milliseconds(source.max_execution_time)),
        comment: source.comment,
        source_result_path: Some(relative_display(contest, &source_path)),
    };
    save_benchmark(contest, &result)?;
    append_event(contest, "benchmark_completed", &result)?;
    Ok(result)
}

fn write_pahcer_config(
    contest: &Path,
    config: &ContestConfig,
    solver: &str,
    cases: usize,
    threads: usize,
    tools: &BuiltTools,
    destination: &Path,
) -> Result<()> {
    let output = format!("./results/pahcer/work/{solver}/out/{{SEED04}}.txt");
    let log = format!("./results/pahcer/work/{solver}/log/{{SEED04}}.txt");
    let input = "./tools/in/{SEED04}.txt".to_string();
    let solver_program = executable_for_config(contest, &tools.solver);
    let workspace = find_workspace_root(contest).ok_or_else(|| {
        anyhow!(
            "AHC workspace marker was not found from {}",
            contest.display()
        )
    })?;
    let isolated_runner = workspace.join("scripts/run-isolated.sh");
    if !isolated_runner.is_file() {
        bail!(
            "isolated runner was not found: {}",
            isolated_runner.display()
        );
    }
    let case_work = contest
        .join(format!("results/pahcer/work/{solver}/case/{{SEED04}}"))
        .display()
        .to_string();
    let mut test_steps = Vec::new();
    if let Some(tester) = &tools.tester {
        test_steps.push(PahcerTestStep {
            program: isolated_runner.display().to_string(),
            args: vec![
                case_work,
                tester.display().to_string(),
                tools.solver.display().to_string(),
            ],
            stdin: Some(input),
            stdout: Some(output),
            stderr: Some(log),
            measure_time: true,
        });
    } else {
        test_steps.push(PahcerTestStep {
            program: solver_program,
            args: Vec::new(),
            stdin: Some(input.clone()),
            stdout: Some(output.clone()),
            stderr: Some(log),
            measure_time: true,
        });
        if let Some(visualizer) = &tools.visualizer {
            test_steps.push(PahcerTestStep {
                program: isolated_runner.display().to_string(),
                args: vec![
                    case_work,
                    visualizer.display().to_string(),
                    contest.join("tools/in/{SEED04}.txt").display().to_string(),
                    contest
                        .join(format!("results/pahcer/work/{solver}/out/{{SEED04}}.txt"))
                        .display()
                        .to_string(),
                ],
                stdin: None,
                stdout: None,
                stderr: None,
                measure_time: false,
            });
        }
    }

    let solver_args = vec![
        "build".to_string(),
        "--release".to_string(),
        "--bin".to_string(),
        solver.to_string(),
    ];
    let tool_name = if tools.tester.is_some() {
        "tester"
    } else {
        "vis"
    };
    let tools_args = vec![
        "build".to_string(),
        "--release".to_string(),
        "--manifest-path".to_string(),
        "tools/Cargo.toml".to_string(),
        "--bin".to_string(),
        tool_name.to_string(),
    ];
    let file = PahcerConfigFile {
        general: PahcerGeneralConfig {
            version: "0.4.0".to_string(),
        },
        problem: PahcerProblemConfig {
            problem_name: config.contest_id.clone(),
            objective: match config.score_direction {
                ScoreDirection::Maximize => "Max",
                ScoreDirection::Minimize => "Min",
            }
            .to_string(),
            score_regex: config.pahcer_score_pattern.clone(),
        },
        test: PahcerTestConfig {
            start_seed: 0,
            end_seed: cases,
            threads,
            out_dir: "./results/pahcer".to_string(),
            compile_steps: vec![
                PahcerCompileStep {
                    program: "cargo".to_string(),
                    args: solver_args,
                },
                PahcerCompileStep {
                    program: "cargo".to_string(),
                    args: tools_args,
                },
            ],
            test_steps,
        },
    };
    create_parent(destination)?;
    fs::write(destination, toml::to_string_pretty(&file)?)?;
    Ok(())
}

fn executable_for_config(contest: &Path, executable: &Path) -> String {
    format!("./{}", relative_display(contest, executable))
}

fn latest_pahcer_result(directory: &Path) -> Result<PathBuf> {
    let mut results = fs::read_dir(directory)
        .with_context(|| {
            format!(
                "pahcer result directory was not found: {}",
                directory.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("result_") && name.ends_with(".json"))
        })
        .collect::<Vec<_>>();
    results.sort();
    results
        .pop()
        .ok_or_else(|| anyhow!("pahcer did not create a result JSON"))
}

fn seconds_to_milliseconds(seconds: f64) -> u128 {
    (seconds.max(0.0) * 1000.0).round() as u128
}

fn validate_solver_name(solver: &str) -> Result<()> {
    let regex = Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
    if regex.is_match(solver) {
        Ok(())
    } else {
        bail!("solver name must use letters, digits, '_' or '-': {solver}")
    }
}

fn save_benchmark(contest: &Path, result: &BenchmarkResult) -> Result<()> {
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let path = contest.join(format!("results/benchmarks/{timestamp}.json"));
    write_json_atomic(&path, result)?;
    write_json_atomic(&contest.join("results/latest.json"), result)?;
    Ok(())
}

fn append_event<T: Serialize>(contest: &Path, event_type: &str, payload: &T) -> Result<()> {
    let path = contest.join("results/events.jsonl");
    create_parent(&path)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let event = Event {
        schema_version: 1,
        event_type,
        occurred_at: now(),
        payload,
    };
    serde_json::to_writer(&mut file, &event)?;
    writeln!(file)?;
    Ok(())
}

fn print_case_result(result: &CaseResult) {
    println!("seed={:04}", result.seed);
    println!("score={}", result.score);
    println!("time={}ms", result.elapsed_milliseconds);
    println!("output={}", result.output_path);
    println!("log={}", result.log_path);
    if let Some(path) = &result.visualization_path {
        println!("visualization={path}");
    }
}

fn print_benchmark(result: &BenchmarkResult) {
    println!();
    println!(
        "runner={} mode={} threads={}",
        result.runner,
        result.score_mode,
        if result.threads == 0 {
            "auto".to_string()
        } else {
            result.threads.to_string()
        }
    );
    println!(
        "accepted={}/{} total={} average={:.2} min={} max={} elapsed={}ms",
        result.accepted_cases,
        result.cases,
        result.total_score,
        result.average_score,
        result.minimum_score,
        result.maximum_score,
        result.wall_time_milliseconds
    );
    if let Some(relative) = result.average_relative_score {
        println!("average_{}={relative:.3}", result.score_mode);
    }
    if let Some(maximum) = result.maximum_execution_milliseconds {
        println!("max_execution={maximum}ms");
    }
    if !result.comment.is_empty() {
        println!("comment={}", result.comment);
    }
}

#[allow(clippy::too_many_arguments)]
fn save_snapshot(
    root: &Path,
    contest: &Path,
    config: &ContestConfig,
    name: &str,
    solver: &str,
    cases: usize,
    builtin: bool,
    threads: usize,
) -> Result<()> {
    validate_snapshot_name(name)?;
    let source_before = tree_fingerprint(&contest.join("src"))?;
    let summary = benchmark(
        root, contest, config, solver, cases, builtin, threads, false, false, name,
    )?;
    let source_after = tree_fingerprint(&contest.join("src"))?;
    if source_before != source_after {
        bail!("source changed during benchmark; snapshot was not saved")
    }

    let id = next_snapshot_id(contest)?;
    let directory = contest.join(format!("snapshots/{id:03}_{name}"));
    let source_path = directory.join("src");
    copy_tree(&contest.join("src"), &source_path)?;
    write_json_atomic(&directory.join("benchmark.json"), &summary)?;
    fs::copy(contest.join("ahc.toml"), directory.join("ahc.toml"))?;

    // 実験用snapshotは無視し、検索・Git管理したい断面だけを1ファイルで残す。
    let solver_source = contest.join(format!("src/bin/{solver}.rs"));
    let expanded = expand_includes(&solver_source, &mut Vec::new())?;
    let average = summary.average_score.round() as i64;
    let solution_path = contest.join(format!("solutions/{id:03}_{name}_avg{average}.rs"));
    create_parent(&solution_path)?;
    let header = format!(
        "// AHC-SOLUTION: {name}\n// AHC-SOLVER: {solver}\n// cases={} average={:.2}\n\n",
        summary.cases, summary.average_score
    );
    fs::write(&solution_path, format!("{header}{expanded}"))?;

    let entry = SnapshotEntry {
        id,
        name: name.to_string(),
        solver: solver.to_string(),
        created_at: now(),
        result_path: relative_display(contest, &directory.join("benchmark.json")),
        source_path: relative_display(contest, &source_path),
        summary,
    };
    append_snapshot_entry(contest, &entry)?;
    append_event(contest, "snapshot_saved", &entry)?;
    println!("Saved snapshot {id:03}_{name}");
    println!("Searchable solution: {}", solution_path.display());
    Ok(())
}

fn show_pahcer_history(root: &Path, contest: &Path, rank: bool, number: usize) -> Result<()> {
    if number == 0 {
        bail!("number must be positive");
    }
    let pahcer = pahcer_binary(root);
    if !pahcer.is_file() {
        bail!(
            "pahcer was not found; run {}/scripts/install-pahcer.sh",
            root.display()
        );
    }
    let config = contest.join("results/pahcer/config.toml");
    if !config.is_file() {
        bail!("pahcer history was not found; run ./ahc bench first");
    }
    let mut command = Command::new(pahcer);
    command
        .arg("list")
        .arg("--setting-file")
        .arg(config)
        .arg("--number")
        .arg(number.to_string())
        .current_dir(contest);
    if rank {
        command.arg("--rank");
    }
    run_checked(&mut command, "pahcer history failed")
}

fn validate_snapshot_name(name: &str) -> Result<()> {
    let regex = Regex::new(r"^[a-z0-9][a-z0-9_-]*$").unwrap();
    if regex.is_match(name) {
        Ok(())
    } else {
        bail!("snapshot name must use lowercase letters, digits, '_' or '-': {name}")
    }
}

fn next_snapshot_id(contest: &Path) -> Result<usize> {
    let entries = read_snapshot_entries(contest)?;
    Ok(entries.iter().map(|entry| entry.id).max().unwrap_or(0) + 1)
}

fn snapshot_index(contest: &Path) -> PathBuf {
    contest.join("snapshots/index.jsonl")
}

fn append_snapshot_entry(contest: &Path, entry: &SnapshotEntry) -> Result<()> {
    let path = snapshot_index(contest);
    create_parent(&path)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, entry)?;
    writeln!(file)?;
    Ok(())
}

fn read_snapshot_entries(contest: &Path) -> Result<Vec<SnapshotEntry>> {
    let path = snapshot_index(contest);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let file = File::open(path)?;
    BufReader::new(file)
        .lines()
        .filter_map(|line| match line {
            Ok(line) if !line.trim().is_empty() => Some(Ok(line)),
            Ok(_) => None,
            Err(error) => Some(Err(error.into())),
        })
        .map(|line: Result<String>| {
            let line = line?;
            serde_json::from_str(&line).context("invalid snapshot index entry")
        })
        .collect()
}

fn list_snapshots(contest: &Path) -> Result<()> {
    let entries = read_snapshot_entries(contest)?;
    if entries.is_empty() {
        println!("No snapshots saved.");
        return Ok(());
    }
    let direction = load_contest_config(contest)?.score_direction;
    let mut best_by_cases: BTreeMap<usize, f64> = BTreeMap::new();
    for entry in &entries {
        best_by_cases
            .entry(entry.summary.cases)
            .and_modify(|value| {
                *value = match direction {
                    ScoreDirection::Maximize => value.max(entry.summary.average_score),
                    ScoreDirection::Minimize => value.min(entry.summary.average_score),
                }
            })
            .or_insert(entry.summary.average_score);
    }
    println!(
        "{:<4} {:<22} {:<12} {:>5} {:>14} {:>14} {:>14} {:>5}",
        "ID", "NAME", "SOLVER", "CASES", "AVERAGE", "MIN", "MAX", "BEST"
    );
    for entry in entries {
        let best = best_by_cases[&entry.summary.cases] == entry.summary.average_score;
        println!(
            "{:<4} {:<22} {:<12} {:>5} {:>14.2} {:>14} {:>14} {:>5}",
            format!("{:03}", entry.id),
            entry.name,
            entry.solver,
            entry.summary.cases,
            entry.summary.average_score,
            entry.summary.minimum_score,
            entry.summary.maximum_score,
            if best { "*" } else { "" }
        );
    }
    Ok(())
}

fn restore_snapshot(contest: &Path, id: usize) -> Result<()> {
    let entries = read_snapshot_entries(contest)?;
    let entry = entries
        .into_iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| anyhow!("snapshot not found: {id:03}"))?;
    let source = contest.join(entry.source_path);
    if !source.is_dir() {
        bail!("snapshot source was not found: {}", source.display());
    }

    let current = contest.join("src");
    let backup = contest.join(format!(
        "snapshots/autosave/{}-before-{id:03}",
        Local::now().format("%Y%m%d-%H%M%S")
    ));
    copy_tree(&current, &backup)?;
    fs::remove_dir_all(&current)?;
    copy_tree(&source, &current)?;
    println!("Restored snapshot {id:03}: {}", entry.name);
    println!("Previous source: {}", backup.display());
    Ok(())
}

fn compare_snapshots(contest: &Path, left: &str, right: &str) -> Result<()> {
    let entries = read_snapshot_entries(contest)?;
    let left = find_snapshot(&entries, left)?;
    let right = find_snapshot(&entries, right)?;
    if left.summary.cases != right.summary.cases {
        eprintln!(
            "warning: case counts differ ({} vs {})",
            left.summary.cases, right.summary.cases
        );
    }
    let delta = right.summary.average_score - left.summary.average_score;
    let direction = load_contest_config(contest)?.score_direction;
    let improvement = match direction {
        ScoreDirection::Maximize => delta,
        ScoreDirection::Minimize => -delta,
    };
    let ratio = if left.summary.average_score == 0.0 {
        None
    } else {
        Some(right.summary.average_score / left.summary.average_score)
    };
    println!(
        "{} ({:.2}) -> {} ({:.2})",
        left.name, left.summary.average_score, right.name, right.summary.average_score
    );
    println!("delta={delta:+.2}");
    println!("improvement={improvement:+.2}");
    if let Some(ratio) = ratio {
        println!("ratio={ratio:.5} ({:+.2}%)", (ratio - 1.0) * 100.0);
    }
    Ok(())
}

fn find_snapshot<'a>(entries: &'a [SnapshotEntry], query: &str) -> Result<&'a SnapshotEntry> {
    if let Ok(id) = query.parse::<usize>() {
        return entries
            .iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| anyhow!("snapshot not found: {query}"));
    }
    let matches = entries
        .iter()
        .filter(|entry| entry.name == query)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => bail!("snapshot not found: {query}"),
        [entry] => Ok(*entry),
        _ => Ok(*matches.last().unwrap()),
    }
}

fn export_solver(contest: &Path, solver: &str, clipboard: bool) -> Result<()> {
    let source = contest.join(format!("src/bin/{solver}.rs"));
    if !source.is_file() {
        bail!("solver source was not found: {}", source.display());
    }
    let mut stack = Vec::new();
    let expanded = expand_includes(&source, &mut stack)?;
    let destination = contest.join(format!("submit/{solver}.rs"));
    create_parent(&destination)?;
    fs::write(&destination, &expanded)?;
    println!("Exported: {}", destination.display());

    if clipboard {
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .context("pbcopy was not found; use the exported file instead")?;
        child.stdin.take().unwrap().write_all(expanded.as_bytes())?;
        let status = child.wait()?;
        if !status.success() {
            bail!("pbcopy failed: {status}");
        }
        println!("Copied to clipboard.");
    }
    Ok(())
}

fn visualization_path(contest: &Path, solver: &str, seed: usize, debug: bool) -> PathBuf {
    contest.join(format!(
        "visualizations/{solver}/{}/{seed:04}.html",
        if debug { "debug" } else { "release" }
    ))
}

fn show_visualization(
    contest: &Path,
    solver: &str,
    seed: usize,
    debug: bool,
    open: bool,
) -> Result<()> {
    let path = visualization_path(contest, solver, seed, debug);
    if !path.is_file() {
        bail!(
            "visualization was not found: {}\nRun ./ahc run {seed} --solver {solver} first.",
            path.display()
        );
    }
    println!("{}", path.display());
    if open {
        let status = Command::new("open")
            .arg(&path)
            .status()
            .context("failed to open the visualization")?;
        if !status.success() {
            bail!("open failed: {status}");
        }
    }
    Ok(())
}

fn serve_visualizations(contest: &Path, port: u16) -> Result<()> {
    let directory = contest.join("visualizations");
    fs::create_dir_all(&directory)?;
    println!("Serving {}", directory.display());
    println!("Open: http://127.0.0.1:{port}/");
    println!("Stop with Ctrl-C.");
    let status = Command::new("python3")
        .args([
            "-m",
            "http.server",
            &port.to_string(),
            "--bind",
            "127.0.0.1",
        ])
        .current_dir(directory)
        .status()
        .context("failed to start python3 http.server")?;
    if status.success() {
        Ok(())
    } else {
        bail!("visualization server failed: {status}")
    }
}

fn show_web_url(contest: &Path, open: bool) -> Result<()> {
    let config = load_contest_config(contest)?;
    let url = config
        .web_visualizer_url
        .ok_or_else(|| anyhow!("web_visualizer_url is not configured in ahc.toml"))?;
    println!("{url}");
    if open {
        let status = Command::new("open")
            .arg(&url)
            .status()
            .context("failed to open the web URL")?;
        if !status.success() {
            bail!("open failed: {status}");
        }
    }
    Ok(())
}

fn expand_includes(path: &Path, stack: &mut Vec<PathBuf>) -> Result<String> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("failed to resolve include source: {}", path.display()))?;
    if stack.contains(&canonical) {
        bail!("cyclic include detected: {}", path.display());
    }
    stack.push(canonical.clone());
    let text = fs::read_to_string(&canonical)?;
    let include = Regex::new(r#"include!\(\s*\"([^\"]+)\"\s*\);"#).unwrap();
    let mut result = String::new();
    let mut previous = 0;
    for captures in include.captures_iter(&text) {
        let matched = captures.get(0).unwrap();
        result.push_str(&text[previous..matched.start()]);
        let included = canonical.parent().unwrap().join(&captures[1]);
        result.push_str(&format!("// BEGIN include: {}\n", included.display()));
        result.push_str(&expand_includes(&included, stack)?);
        result.push_str(&format!("\n// END include: {}", included.display()));
        previous = matched.end();
    }
    result.push_str(&text[previous..]);
    stack.pop();
    Ok(result)
}

fn search(
    root: &Path,
    query: Option<&str>,
    tag: Option<&str>,
    lib: bool,
    contests_only: bool,
) -> Result<()> {
    let pattern = match (query, tag) {
        (Some(query), None) => query.to_string(),
        (None, Some(tag)) => format!("AHC-{}", tag.to_ascii_uppercase()),
        (Some(query), Some(tag)) => {
            format!(
                "AHC-{}.*{}|{}.*AHC-{}",
                tag.to_ascii_uppercase(),
                query,
                query,
                tag.to_ascii_uppercase()
            )
        }
        (None, None) => bail!("pass a query or --tag <name>"),
    };

    let config = workspace_config(root)?;
    let mut paths = Vec::new();
    if contests_only {
        paths.push(root.join("contests"));
    } else if lib {
        let value = config
            .common_library_path
            .ok_or_else(|| anyhow!("common_library_path is not configured"))?;
        paths.push(root.join(value));
    } else {
        for value in config.search_roots {
            paths.push(root.join(value));
        }
        if let Some(value) = config.common_library_path {
            paths.push(root.join(value));
        }
    }
    paths.retain(|path| path.exists());
    if paths.is_empty() {
        bail!("no configured search roots exist")
    }

    let mut command = Command::new("rg");
    command.arg("-n").arg("--smart-case").arg(&pattern);
    for path in paths {
        command.arg(path);
    }
    let status = command.status().context("failed to start rg")?;
    if status.success() || status.code() == Some(1) {
        Ok(())
    } else {
        bail!("rg failed: {status}")
    }
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_dir() {
        bail!("source directory was not found: {}", source.display());
    }
    fs::create_dir_all(destination)?;
    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(source)?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            create_parent(&target)?;
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn replace_in_tree(root: &Path, from: &str, to: &str) -> Result<()> {
    for entry in WalkDir::new(root) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let mut bytes = Vec::new();
        File::open(entry.path())?.read_to_end(&mut bytes)?;
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        if text.contains(from) {
            fs::write(entry.path(), text.replace(from, to))?;
        }
    }
    Ok(())
}

fn tree_fingerprint(root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut result = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(root)?.display().to_string();
            result.push((relative, fs::read(entry.path())?));
        }
    }
    result.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(result)
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    create_parent(path)?;
    let temporary = path.with_extension("tmp");
    let mut file = File::create(&temporary)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    writeln!(file)?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn create_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn relative_display(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn now() -> String {
    Local::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_url_prefers_non_windows_archive_without_tools_in_name() {
        let html = r#"
            <a href="https://img.atcoder.jp/ahc999/windows.zip">Windows</a>
            <a href="https://img.atcoder.jp/ahc999/a1b2c3.zip">Source</a>
        "#;
        assert_eq!(
            select_tools_url(html).as_deref(),
            Some("https://img.atcoder.jp/ahc999/a1b2c3.zip")
        );
    }

    #[test]
    fn contest_start_time_is_read_from_atcoder_page() {
        let html = r#"<script>var startTime = moment("2026-09-13T19:00:00+09:00");</script>"#;
        assert_eq!(
            select_contest_start_time(html).unwrap().to_rfc3339(),
            "2026-09-13T19:00:00+09:00"
        );
    }

    #[test]
    fn include_export_is_recursive() {
        let directory = env::temp_dir().join(format!("ahc-cli-test-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let root = directory.join("root.rs");
        let child = directory.join("child.rs");
        fs::write(&root, "mod child { include!(\"child.rs\"); }").unwrap();
        fs::write(&child, "pub const VALUE: usize = 42;").unwrap();
        let expanded = expand_includes(&root, &mut Vec::new()).unwrap();
        assert!(expanded.contains("pub const VALUE: usize = 42;"));
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn pahcer_config_runs_selected_binaries_in_place() {
        let directory =
            env::temp_dir().join(format!("ahc-pahcer-config-test-{}", std::process::id()));
        fs::create_dir_all(directory.join("target/release")).unwrap();
        fs::create_dir_all(directory.join("tools/target/release")).unwrap();
        fs::create_dir_all(directory.join("scripts")).unwrap();
        fs::write(directory.join(".ahc-root"), "test workspace\n").unwrap();
        fs::write(
            directory.join("scripts/run-isolated.sh"),
            "#!/bin/sh\nexec \"$@\"\n",
        )
        .unwrap();
        let destination = directory.join("results/pahcer/config.toml");
        let config = ContestConfig {
            contest_id: "ahc999".to_string(),
            score_direction: ScoreDirection::Maximize,
            score_pattern: default_score_pattern(),
            pahcer_score_pattern: default_pahcer_score_pattern(),
            web_visualizer_url: None,
            total_time_limit_seconds: None,
        };
        let tools = BuiltTools {
            solver: directory.join("target/release/beam"),
            generator: None,
            tester: None,
            visualizer: Some(directory.join("tools/target/release/vis")),
        };

        write_pahcer_config(&directory, &config, "beam", 10, 4, &tools, &destination).unwrap();
        let text = fs::read_to_string(destination).unwrap();
        assert!(text.contains("program = \"./target/release/beam\""));
        assert!(text.contains("run-isolated.sh"));
        assert!(text.contains("tools/target/release/vis"));
        assert!(text.contains("threads = 4"));
        assert!(!text.contains("program = \"rm\""));
        assert!(!text.contains("program = \"mv\""));
        fs::remove_dir_all(&directory).unwrap();
    }
}
