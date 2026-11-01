use std::path::Path;
use std::process::Command;

pub(crate) fn check_dirty_git_files(
    project_root: &Path,
    target_rel_paths: &[String],
) -> Vec<String> {
    if target_rel_paths.is_empty() || !project_root.join(".git").exists() {
        return Vec::new();
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--porcelain")
        .arg("--untracked-files=no")
        .arg("--")
        .args(target_rel_paths)
        .output();

    let Ok(out) = output else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.len() > 3 {
                trimmed.get(2..).map(|p| p.trim().trim_matches('"').to_owned())
            } else {
                None
            }
        })
        .collect()
}
