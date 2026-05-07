use crate::modules::project::{ProjectInfo, TaskResult, TaskSummary};
use anyhow::Result;
use std::fs;

pub async fn search_pattern(projects: &[ProjectInfo], pattern: &str, file_type: &str) -> Result<()> {
    println!("\n[SEARCH] Pattern: '{}' in {} files", pattern, file_type);
    println!("{}", "=".repeat(60));

    let mut found_count = 0;
    let mut results = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        let src_path = project.path.join("src").join(file_type);

        if !src_path.exists() {
            continue;
        }

        match fs::read_to_string(&src_path) {
            Ok(content) => {
                let mut file_matches = Vec::new();

                for (line_num, line) in content.lines().enumerate() {
                    if line.contains(pattern) {
                        file_matches.push((line_num + 1, line.trim().to_string()));
                    }
                }

                if !file_matches.is_empty() {
                    found_count += 1;
                    results.push((project.name.clone(), file_matches));
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", src_path.display(), e);
            }
        }
    }

    if results.is_empty() {
        println!("Pattern not found in any projects");
    } else {
        for (project_name, matches) in &results {
            println!("\n[{}] {} matches:", project_name, matches.len());
            for (line_num, line) in matches {
                println!("   {:5}: {}", line_num, line);
            }
        }

        println!("\n{}", "=".repeat(60));
        println!("Search Results:");
        println!("Found in {} projects", found_count);
        println!("Total matches: {}", results.iter().map(|(_, m)| m.len()).sum::<usize>());
    }

    Ok(())
}

pub async fn migrate_psram_init(
    projects: &[ProjectInfo],
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[MIGRATE] PSRAM initialization to esp-hal 1.1.0 API");
    if dry_run {
        println!("DRY-RUN mode - no changes will be made");
    }
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();
    let mut results = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nProcessing: {}", project.name);

        let result = migrate_project_psram(project, dry_run, verbose).await?;

        if result.success {
            if !result.message.is_empty() {
                println!("OK: {}", project.name);
                if verbose {
                    println!("   Changes: {}", result.message);
                }
            } else {
                if verbose {
                    println!("OK: No changes needed: {}", project.name);
                }
            }
        } else {
            println!("FAILED: {}", project.name);
            if !result.message.is_empty() {
                println!("   Error: {}", result.message);
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
            println!("3. Verify functionality of updated projects");
        } else {
            println!("1. Review the above output");
            println!("2. Run without --dry-run to apply changes");
        }
    }

    Ok(())
}

async fn migrate_project_psram(
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

    if !content.contains("esp_alloc::psram_allocator!") {
        return Ok(TaskResult {
            project: project.name.clone(),
            success: true,
            message: String::new(),
            warnings: Vec::new(),
        });
    }

    let mut changes = Vec::new();
    let mut new_content = content.clone();

    if !new_content.contains("use esp_hal::psram::Psram;") {
        if let Some(pos) = new_content.find("use esp_hal::") {
            if let Some(end_pos) = new_content[pos..].find(';') {
                let insert_pos = pos + end_pos + 1;
                new_content.insert_str(insert_pos, "\nuse esp_hal::psram::Psram;");
                changes.push("Added Psram import".to_string());
            }
        }
    }

    let old_pattern = "esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);";
    let new_pattern = "let psram = Psram::new(peripherals.PSRAM, Default::default());\n    esp_alloc::psram_allocator!(&psram);";

    if new_content.contains(old_pattern) {
        new_content = new_content.replace(old_pattern, new_pattern);
        changes.push("Updated PSRAM initialization to 1.1.0 API".to_string());
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
