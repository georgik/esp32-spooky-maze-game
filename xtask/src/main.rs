use anyhow::Result;
use clap::{Parser, Subcommand};

mod modules;

use modules::{
    project::discover_projects,
    build::{build_all_projects, format_all_projects},
    update::update_dependencies,
    migrate::{search_pattern, migrate_psram_init},
    config::{clean_deprecated_config, fix_workspace_issues},
    psram_feature::remove_psram_feature,
    fix_cargo::{fix_corrupted_cargo_toml, scan_corrupted_files, scan_and_fix_corrupted_files},
    embassy::migrate_embassy_api,
    toml_fix::fix_cargo_toml_quotes,
};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "ESP32 Embedded Projects Maintenance Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build all ESP32 projects with release profile
    Build {
        #[arg(long)]
        keep_going: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    /// Update dependencies across all projects
    Update {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        incompatible: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    /// Format all project code using cargo fmt
    Format {
        #[arg(long, short)]
        verbose: bool,
    },
    /// Run all maintenance tasks: format, update (compatible), and build
    All {
        #[arg(long)]
        keep_going: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    /// List all discovered ESP32 projects
    List,
    /// Fix workspace issues by adding empty [workspace] to all projects
    FixWorkspace,
    /// Remove deprecated build configuration options
    CleanConfig {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    /// Search for code patterns across all projects
    Search {
        pattern: String,
        #[arg(long, default_value = "main.rs")]
        file_type: String,
    },
    /// Migrate PSRAM initialization to esp-hal 1.1.0 API
    MigratePsram {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    /// Remove deprecated psram feature from esp-hal dependency
    RemovePsramFeature {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    /// Fix corrupted Cargo.toml files
    FixCargo {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    /// Scan for corrupted Cargo.toml files
    ScanCargo,
    /// Scan and fix corrupted Cargo.toml files
    ScanFixCargo {
        #[arg(long)]
        dry_run: bool,
    },
    /// Migrate Embassy API for esp-hal 1.1.0
    MigrateEmbassy {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    /// Fix missing quotes in Cargo.toml using toml crate
    FixQuotes {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("ESP32 Embedded Projects Maintenance Tool");
    println!("{}", "=".repeat(60));

    let projects = discover_projects().await?;
    if projects.is_empty() {
        println!("No ESP32 projects found matching patterns");
        return Ok(());
    }

    match cli.command {
        Commands::List => list_projects(&projects).await,
        Commands::Build {
            keep_going,
            verbose,
        } => build_all_projects(&projects, keep_going, verbose).await,
        Commands::Update {
            dry_run,
            incompatible,
            verbose,
        } => update_dependencies(&projects, dry_run, incompatible, verbose).await,
        Commands::Format { verbose } => format_all_projects(&projects, verbose).await,
        Commands::All {
            keep_going,
            verbose,
        } => run_all_tasks(&projects, keep_going, verbose).await,
        Commands::FixWorkspace => fix_workspace_issues(&projects).await,
        Commands::CleanConfig { dry_run, verbose } => {
            clean_deprecated_config(&projects, dry_run, verbose).await
        }
        Commands::Search { pattern, file_type } => {
            search_pattern(&projects, &pattern, &file_type).await
        }
        Commands::MigratePsram { dry_run, verbose } => {
            migrate_psram_init(&projects, dry_run, verbose).await
        }
        Commands::RemovePsramFeature { dry_run, verbose } => {
            remove_psram_feature(&projects, dry_run, verbose).await
        }
        Commands::FixCargo { dry_run, verbose } => {
            fix_corrupted_cargo_toml(&projects, dry_run, verbose).await
        }
        Commands::ScanCargo => {
            scan_corrupted_files(&projects).await?;
            Ok(())
        }
        Commands::ScanFixCargo { dry_run } => {
            scan_and_fix_corrupted_files(&projects, dry_run).await
        }
        Commands::MigrateEmbassy { dry_run, verbose } => {
            migrate_embassy_api(&projects, dry_run, verbose).await
        }
        Commands::FixQuotes { dry_run, verbose } => {
            fix_cargo_toml_quotes(&projects, dry_run, verbose).await
        }
    }
}

async fn list_projects(projects: &[crate::modules::project::ProjectInfo]) -> Result<()> {
    println!("\n[LIST] Discovered ESP32 Projects ({} total):", projects.len());
    println!("┌─────┬──────────────────────────────────────┬────────────┐");
    println!("│ #   │ Project Name                         │ Status     │");
    println!("├─────┼──────────────────────────────────────┼────────────┤");

    for (i, project) in projects.iter().enumerate() {
        let status = if project.has_cargo_toml {
            "Ready"
        } else {
            "No Cargo.toml"
        };
        println!("│ {:3} │ {:<44} │ {} │", i + 1, project.name, status);
    }

    println!("└─────┴──────────────────────────────────────┴────────────┘");

    let ready_count = projects.iter().filter(|p| p.has_cargo_toml).count();
    println!(
        "\nSummary: {} ready for build, {} missing Cargo.toml",
        ready_count,
        projects.len() - ready_count
    );

    Ok(())
}

async fn run_all_tasks(
    projects: &[crate::modules::project::ProjectInfo],
    keep_going: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[RUN ALL] Executing all maintenance tasks");
    println!("{}", "=".repeat(60));
    println!("Tasks: Format -> Update (compatible) -> Build");

    format_all_projects(projects, verbose).await?;
    update_dependencies(projects, false, false, verbose).await?;
    build_all_projects(projects, keep_going, verbose).await?;

    println!("\nAll maintenance tasks completed!");
    Ok(())
}
