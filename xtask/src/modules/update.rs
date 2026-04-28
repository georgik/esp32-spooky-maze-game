use crate::modules::project::{ProjectInfo, TaskResult, TaskSummary};
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

pub async fn update_dependencies(
    projects: &[ProjectInfo],
    dry_run: bool,
    incompatible: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[UPDATE] Updating dependencies across all projects");

    let cargo_edit_check = TokioCommand::new("cargo")
        .arg("upgrade")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    if let Err(e) = cargo_edit_check {
        println!("Error: Failed to check for cargo-edit: {}", e);
        println!("\nRequired dependency missing: cargo-edit");
        println!("The 'cargo upgrade' command is provided by the cargo-edit crate.");
        println!("\nPlease install it with:");
        println!("   cargo install cargo-edit");
        return Ok(());
    } else if let Ok(output) = cargo_edit_check {
        if !output.status.success() {
            println!("Error: cargo-edit is not installed");
            println!("\nRequired dependency missing: cargo-edit");
            println!("\nPlease install it with:");
            println!("   cargo install cargo-edit");
            return Ok(());
        }
    }

    if dry_run {
        println!("DRY-RUN mode - no changes will be made");
    }
    if incompatible {
        println!("Including incompatible updates (potentially breaking)");
    }
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nProcessing: {}", project.name);

        let result = update_project_deps(project, dry_run, incompatible, verbose).await?;

        if result.success {
            println!("OK: {}", project.name);
        } else {
            println!("FAILED: {}", project.name);
            if verbose {
                println!("Error: {}", result.message);
            }
        }

        summary.add_result(&result);
    }

    println!("\n{}", "=".repeat(60));
    println!("Update Summary:");
    println!("Successfully updated: {} projects", summary.success);
    if summary.failed > 0 {
        println!("Failed: {} projects", summary.failed);
    }

    if summary.success > 0 {
        println!("\nNext Steps:");
        if !dry_run {
            println!("1. Review updated Cargo.toml files for breaking changes");
            println!("2. Test builds: cargo xtask build");
            println!("3. Verify functionality of updated projects");
        } else {
            println!("1. Review the above output");
            println!("2. Run without --dry-run to apply changes");
        }
    }

    Ok(())
}

async fn update_project_deps(
    project: &ProjectInfo,
    dry_run: bool,
    incompatible: bool,
    verbose: bool,
) -> Result<TaskResult> {
    let mut cmd = TokioCommand::new("cargo");
    cmd.arg("upgrade")
        .current_dir(&project.path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if dry_run {
        cmd.arg("--dry-run");
    }
    if incompatible {
        cmd.arg("--incompatible");
    }
    if verbose {
        cmd.arg("--verbose");
    }

    let output = cmd
        .output()
        .await
        .with_context(|| format!("Failed to run cargo upgrade in {}", project.name))?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut message_lines = Vec::new();
    for line in stdout.lines().chain(stderr.lines()) {
        if line.contains("Upgrading")
            || line.contains("Updated")
            || line.contains("incompatible")
            || line.contains("latest")
            || line.trim().starts_with("name")
            || line.contains("->")
        {
            message_lines.push(line.to_string());
            if verbose {
                println!("   {}", line);
            }
        }
    }

    let message = if success {
        "Dependencies processed successfully".to_string()
    } else {
        stderr.to_string()
    };

    Ok(TaskResult {
        project: project.name.clone(),
        success,
        message,
        warnings: Vec::new(),
    })
}
