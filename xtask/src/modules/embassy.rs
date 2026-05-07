use crate::modules::project::{ProjectInfo, TaskResult, TaskSummary};
use anyhow::Result;
use std::fs;

pub async fn migrate_embassy_api(
    projects: &[ProjectInfo],
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[MIGRATE] Embassy API updates for esp-hal 1.1.0");
    if dry_run {
        println!("DRY-RUN mode - no changes will be made");
    }
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();
    let mut results = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nProcessing: {}", project.name);

        let result = migrate_project_embassy(project, dry_run, verbose).await?;

        if result.success {
            if !result.message.is_empty() {
                println!("OK: {}", project.name);
                if verbose {
                    println!("Changes: {}", result.message);
                }
            } else {
                if verbose {
                    println!("OK: No changes needed: {}", project.name);
                }
            }
        } else {
            println!("FAILED: {}", project.name);
            if !result.message.is_empty() {
                println!("Error: {}", result.message);
            }
        }

        summary.add_result(&result);
        results.push(result);
    }

    println!("\n{}", "=".repeat(60));
    println!("Migration Summary:");
    println!("Successfully processed: {} projects", summary.success);
    if summary.failed > 0 {
        println!("Failed: {} projects", summary.failed);
    }

    let migrated_count = results.iter().filter(|r| r.success && !r.message.is_empty()).count();
    if migrated_count > 0 {
        println!("\nProjects migrated: {}", migrated_count);
        for result in results.iter().filter(|r| r.success && !r.message.is_empty()) {
            println!("  * {}", result.project);
        }
    }

    if migrated_count > 0 {
        println!("\nNext Steps:");
        if !dry_run {
            println!("1. Review updated main.rs files");
            println!("2. Test builds: cargo xtask build");
        } else {
            println!("1. Review the above output");
            println!("2. Run without --dry-run to apply changes");
        }
    }

    Ok(())
}

async fn migrate_project_embassy(
    project: &ProjectInfo,
    dry_run: bool,
    verbose: bool,
) -> Result<TaskResult> {
    let main_rs_path = project.path.join("src").join("main.rs");

    if !main_rs_path.exists() {
        return Ok(TaskResult {
            project: project.name.clone(),
            success: true,
            message: String::new(),
            warnings: Vec::new(),
        });
    }

    let content = match fs::read_to_string(&main_rs_path) {
        Ok(content) => content,
        Err(e) => {
            return Ok(TaskResult {
                project: project.name.clone(),
                success: false,
                message: format!("Failed to read main.rs: {}", e),
                warnings: Vec::new(),
            });
        }
    };

    let mut changes = Vec::new();
    let mut new_content = content.clone();

    // Fix 1: esp_rtos::start() now requires SoftwareInterrupt parameter
    if new_content.contains("esp_rtos::start(") &&
       !new_content.contains("esp_rtos::start(timer0, sw_ints.software_interrupt0)") {
        new_content = new_content.replace(
            "esp_rtos::start(timer0);",
            "esp_rtos::start(timer0, sw_ints.software_interrupt0);"
        );
        changes.push("Added SoftwareInterrupt parameter to esp_rtos::start".to_string());
    }

    // Fix 2: esp_rtos::start_second_core() API changed
    // Old: 5 args including stack
    // New: 4 args, stack wrapped in closure
    if new_content.contains("esp_rtos::start_second_core(") {
        // Remove the stack parameter and wrap it in a closure
        if new_content.contains("app_core_stack,") {
            new_content = new_content.replace(
                "esp_rtos::start_second_core(\n        peripherals.CPU_CTRL,\n        sw_ints.software_interrupt0,\n        app_core_stack,",
                "esp_rtos::start_second_core(\n        peripherals.CPU_CTRL,\n        sw_ints.software_interrupt0,"
            );
            // Add closure wrapper for the task
            new_content = new_content.replace(
                "|| { app_core_task() }",
                "{ || { app_core_task() } }"
            );
            changes.push("Updated esp_rtos::start_second_core() API".to_string());
        }
    }

    // Fix 3: Remove .ok() and .unwrap() from spawn calls
    if new_content.contains(".spawn(") && new_content.contains(".ok()") {
        new_content = new_content.replace(".spawn(", ".spawn_")
            .replace(".ok();", ";");
        changes.push("Removed .ok() from spawn calls".to_string());
    }

    if new_content.contains(".spawn(") && new_content.contains(".unwrap()") {
        new_content = new_content.replace(".spawn(", ".spawn_")
            .replace(".unwrap();", ";");
        changes.push("Removed .unwrap() from spawn calls".to_string());
    }

    // Fix 4: Change spawn to spawn_ for embassy compatibility
    if new_content.contains(".spawn(") {
        new_content = new_content.replace(".spawn(", ".spawn_(");
        changes.push("Updated spawn() to spawn_()".to_string());
    }

    if changes.is_empty() {
        return Ok(TaskResult {
            project: project.name.clone(),
            success: true,
            message: String::new(),
            warnings: Vec::new(),
        });
    }

    if verbose {
        println!("   Planned changes: {}", changes.join(", "));
    }

    if !dry_run {
        if let Err(e) = fs::write(&main_rs_path, &new_content) {
            return Ok(TaskResult {
                project: project.name.clone(),
                success: false,
                message: format!("Failed to write main.rs: {}", e),
                warnings: Vec::new(),
            });
        }
    }

    Ok(TaskResult {
        project: project.name.clone(),
        success: true,
        message: changes.join(", "),
        warnings: Vec::new(),
    })
}
