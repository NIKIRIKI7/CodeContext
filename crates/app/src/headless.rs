use std::path::Path;

use anyhow::Result;
use core::applying::aggregates::backup_manifest::BackupManifest;
use core::packing::aggregates::pack_result::PackResult;
use core::shared_kernel::execution::cancel_token::CancelToken;
use core::system::ports::shell::ShellIntegrationPort as _;
use di::AppContainer;
use platform::resolve_headless_target_path;

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
