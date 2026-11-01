use std::path::{Path, PathBuf};

use platform::resolve_headless_target_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LaunchMode {
    HeadlessPack { project_root: PathBuf },
    HeadlessApply { project_root: PathBuf },
    RegisterShell,
    UnregisterShell,
    SelfInstall,
    CheckUpdate,
    Version,
    Gui { initial_project_root: Option<PathBuf> },
}

pub(crate) fn parse_args<I>(args: I) -> LaunchMode
where
    I: IntoIterator<Item = String>,
{
    let mut iter = args.into_iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "--version" || arg == "-V" {
            return LaunchMode::Version;
        }
        if arg == "--pack-headless" {
            let path = iter
                .next()
                .map_or_else(|| PathBuf::from("."), |raw| resolve_headless_target_path(&raw));
            return LaunchMode::HeadlessPack { project_root: path };
        }
        if arg == "--apply-headless" {
            let path = iter
                .next()
                .map_or_else(|| PathBuf::from("."), |raw| resolve_headless_target_path(&raw));
            return LaunchMode::HeadlessApply { project_root: path };
        }
        if arg == "--register-shell" {
            return LaunchMode::RegisterShell;
        }
        if arg == "--unregister-shell" {
            return LaunchMode::UnregisterShell;
        }
        if arg == "--self-install" {
            return LaunchMode::SelfInstall;
        }
        if arg == "--check-update" {
            return LaunchMode::CheckUpdate;
        }
        if arg == "--project" || arg == "-p" {
            let path = iter.next().map(|raw| resolve_headless_target_path(&raw));
            return LaunchMode::Gui { initial_project_root: path };
        }
        if !arg.starts_with('-') {
            return LaunchMode::Gui {
                initial_project_root: Some(resolve_headless_target_path(&arg)),
            };
        }
    }

    let current_dir = std::env::current_dir().ok().map(|dir| resolve_workspace_root(&dir));
    LaunchMode::Gui { initial_project_root: current_dir }
}

fn resolve_workspace_root(start_dir: &Path) -> PathBuf {
    let mut cursor = Some(start_dir);
    let mut highest_cargo_root: Option<PathBuf> = None;

    for _ in 0..4 {
        let Some(dir) = cursor else {
            break;
        };
        if dir.join(".git").exists() || dir.join(".ctxpack.json").exists() {
            return dir.to_path_buf();
        }
        if dir.join("Cargo.toml").exists() {
            highest_cargo_root = Some(dir.to_path_buf());
        }
        cursor = dir.parent();
    }

    highest_cargo_root.unwrap_or_else(|| start_dir.to_path_buf())
}
