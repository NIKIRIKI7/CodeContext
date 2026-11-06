use std::path::{Path, PathBuf};

use platform::resolve_headless_target_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CleanScope {
    All,
    Backups,
    Temp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LaunchMode {
    HeadlessPack { project_root: PathBuf },
    HeadlessApply { project_root: PathBuf },
    Clean { scope: CleanScope, project_root: PathBuf },
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
        if arg == "--clean" {
            let next = iter.next().unwrap_or_else(|| "all".to_owned());
            let scope = match next.as_str() {
                "backups" => CleanScope::Backups,
                "tmp" | "temp" => CleanScope::Temp,
                _ => CleanScope::All,
            };
            let path = iter
                .next()
                .map_or_else(|| PathBuf::from("."), |raw| resolve_headless_target_path(&raw));
            return LaunchMode::Clean { scope, project_root: path };
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
            let path = iter
                .next()
                .map(|raw| resolve_headless_target_path(&raw))
                .filter(|p| !is_root_or_system_directory(p));
            return LaunchMode::Gui { initial_project_root: path };
        }
        if !arg.starts_with('-') {
            if arg == "app" && !Path::new("app").exists() {
                continue;
            }
            let path = resolve_headless_target_path(&arg);
            let safe_path = if is_root_or_system_directory(&path) || !path.exists() {
                None
            } else {
                Some(path)
            };
            return LaunchMode::Gui { initial_project_root: safe_path };
        }
    }

    LaunchMode::Gui { initial_project_root: None }
}

fn is_root_or_system_directory(path: &Path) -> bool {
    let resolved = if path == Path::new(".") {
        std::env::current_dir().unwrap_or_else(|_| path.to_path_buf())
    } else {
        path.to_path_buf()
    };
    if resolved.parent().is_none() {
        return true;
    }
    let s = resolved.to_string_lossy();
    let trimmed = s.trim_end_matches(['\\', '/']);
    let bytes = trimmed.as_bytes();
    if (bytes.len() == 2 || bytes.len() == 3)
        && bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes.get(1).copied() == Some(b':')
    {
        return true;
    }
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))
        && resolved == Path::new(&home)
    {
        return true;
    }
    false
}
