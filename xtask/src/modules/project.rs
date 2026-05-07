use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub const PROJECT_PATTERNS: &[&str] = &["spooky-maze-esp32*", "spooky-maze-m5stack*"];

#[derive(Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub path: PathBuf,
    pub has_cargo_toml: bool,
}

pub struct TaskResult {
    pub project: String,
    pub success: bool,
    pub message: String,
    pub warnings: Vec<String>,
}

pub struct TaskSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub warnings: usize,
}

impl TaskSummary {
    pub fn new() -> Self {
        Self {
            total: 0,
            success: 0,
            failed: 0,
            warnings: 0,
        }
    }

    pub fn add_result(&mut self, result: &TaskResult) {
        self.total += 1;
        if result.success {
            self.success += 1;
        } else {
            self.failed += 1;
        }
        self.warnings += result.warnings.len();
    }
}

pub async fn discover_projects() -> Result<Vec<ProjectInfo>> {
    let current_dir = std::env::current_dir()?;
    let mut projects = Vec::new();

    let mut entries = fs::read_dir(&current_dir)
        .with_context(|| "Failed to read current directory")?;

    while let Some(entry) = entries.next().transpose()? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        let matches_pattern = PROJECT_PATTERNS.iter().any(|pattern| {
            let pattern = pattern.trim_end_matches('*');
            dir_name.starts_with(pattern)
        });

        if matches_pattern {
            let cargo_toml = path.join("Cargo.toml");
            let has_cargo_toml = cargo_toml.exists();

            projects.push(ProjectInfo {
                name: dir_name.to_string(),
                path,
                has_cargo_toml,
            });
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(projects)
}
