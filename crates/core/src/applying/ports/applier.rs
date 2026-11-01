use std::path::Path;

use thiserror::Error;

use crate::applying::aggregates::apply_plan::ApplyPlan;
use crate::applying::aggregates::backup_manifest::BackupManifest;
use crate::preset::models::template::{OutputTemplate, ParserConfig};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ApplierPortError {
    #[error("План применения пуст")]
    EmptyPlan,
    #[error("Нарушение границ песочницы проекта: {0}")]
    SandboxViolation(String),
    #[error("Конфликт наложения патча для файла `{path}`: не найдено блоков {unmatched}")]
    PatchConflict { path: String, unmatched: usize },
    #[error("Ошибка записи или восстановления бэкапа: {0}")]
    StorageError(String),
}

pub trait ApplierPort: Send + Sync {
    fn autofix_with_config(
        &self,
        raw_response: &str,
        template: &OutputTemplate,
        config: &ParserConfig,
    ) -> (String, bool, bool);

    fn build_apply_plan(
        &self,
        project_root: Option<&Path>,
        raw_response: &str,
        template: &OutputTemplate,
        config: &ParserConfig,
    ) -> ApplyPlan;

    fn execute_apply_plan(
        &self,
        project_root: &Path,
        plan: &ApplyPlan,
        create_backup: bool,
    ) -> Result<BackupManifest, ApplierPortError>;

    fn undo_last_backup(&self, project_root: &Path) -> Result<usize, ApplierPortError>;
}
