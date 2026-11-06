use std::path::Path;

use anyhow::Result;
use core::applying::aggregates::backup_manifest::BackupManifest;
use core::packing::aggregates::pack_result::PackResult;
use core::preset::models::resolution::EditionProfile;
use core::shared_kernel::execution::cancel_token::CancelToken;
use core::system::ports::shell::{ShellIntegrationPort as _, ShellMenuConfig};
use core::system::ports::updater::{
    SelfInstallStatus, UpdateChannel, UpdateStage, UpdaterPort as _,
};
use di::AppContainer;
use platform::resolve_headless_target_path;

use crate::cli::CleanScope;

pub(crate) fn run_headless_clean(scope: CleanScope, target_path: &Path) -> u64 {
    let container = AppContainer::new();
    let project_root = resolve_headless_target_path(&target_path.to_string_lossy());
    let (clean_backups, clean_temp, clean_ipc) = match scope {
        CleanScope::All => (true, true, true),
        CleanScope::Backups => (true, false, false),
        CleanScope::Temp => (false, true, false),
    };
    container.clean_housekeeping(Some(&project_root), clean_backups, clean_temp, clean_ipc)
}

pub(crate) fn run_headless_pack(target_path: &Path) -> Result<PackResult> {
    let container = AppContainer::new();
    let project_root = resolve_headless_target_path(&target_path.to_string_lossy());
    let (preset, from_untrusted, _warnings) =
        container.preset_service().resolve_headless(&project_root);
    let cancel_token = CancelToken::new();

    let result = container.pack_service().pack_headless(
        &project_root,
        preset,
        from_untrusted,
        &cancel_token,
    )?;
    Ok(result)
}

pub(crate) fn run_headless_apply(target_path: &Path) -> Result<BackupManifest> {
    let container = AppContainer::new();
    let project_root = resolve_headless_target_path(&target_path.to_string_lossy());
    let raw_clipboard = container.clipboard().read_text()?;
    let (fixed_text, _preamble_stripped, _closed_truncated) =
        container.autofix_response_for_project(&raw_clipboard, Some(&project_root));
    let create_backup = container.preset_service().load_global_config().create_backups_by_default;
    let manifest = container.execute_plan_for_project(&project_root, &fixed_text, create_backup)?;
    Ok(manifest)
}

pub(crate) fn register_os_context_menu() -> Result<()> {
    let container = AppContainer::new();
    container.shell_integration().register_context_menu()?;
    Ok(())
}

pub(crate) fn unregister_os_context_menu() -> Result<()> {
    let container = AppContainer::new();
    container.shell_integration().unregister_context_menu()?;
    Ok(())
}

pub(crate) fn run_modular_setup(
    edition_str: &str,
    target_dir: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf> {
    let container = AppContainer::new();
    let probe = container.updater().probe_system();
    let edition = EditionProfile::parse(edition_str);
    println!("====================================================================");
    println!(" CodeContext Modular Installer (v{})", container.updater().current_version());
    println!(" OS / Distro     : {} ({})", probe.os_name, probe.distro_or_edition);
    println!(" CPU / SIMD      : {} cores · {:?}", probe.logical_cores, probe.simd_level);
    println!(" Display / Shell : {} · {}", probe.display_server, probe.file_manager);
    println!(" Edition Profile : {}", edition.as_str());
    println!("====================================================================");
    let path =
        container.perform_modular_setup(edition, target_dir, true).map_err(anyhow::Error::msg)?;
    println!(" [OK] Installed binary & manifest to: {}", path.display());
    Ok(path)
}

pub(crate) fn run_uninstall(purge_user_data: bool) -> Result<usize> {
    let container = AppContainer::new();
    let _ = container.shell_integration().unregister_context_menu();
    let removed = container.updater().execute_uninstall(purge_user_data)?;
    println!(" [OK] CodeContext uninstalled (removed {removed} artifacts).");
    Ok(removed)
}

pub(crate) fn run_self_install() -> Result<SelfInstallStatus> {
    let container = AppContainer::new();
    let status = container.updater().perform_self_install()?;
    if let SelfInstallStatus::InstalledCanonical { ref install_path } = status {
        let cfg = ShellMenuConfig::with_executable(install_path.clone());
        container.shell_integration().register_with_config(&cfg)?;
    }
    Ok(status)
}

pub(crate) fn run_check_update() -> Result<UpdateStage> {
    let container = AppContainer::new();
    let global_cfg = container.preset_service().load_global_config();
    let channel = UpdateChannel::parse(&global_cfg.update_channel);
    let stage = container.updater().check_for_updates(channel, &global_cfg.update_repo_url)?;
    Ok(stage)
}
