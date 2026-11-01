use std::path::Path;

use crate::applying::aggregates::apply_plan::ApplyPlan;
use crate::applying::aggregates::backup_manifest::BackupManifest;
use crate::applying::ports::applier::{ApplierPort, ApplierPortError};
use crate::preset::models::preset::Preset;

#[derive(Debug, Clone)]
pub struct ApplyService<A> {
    applier: A,
}

impl<A> ApplyService<A>
where
    A: ApplierPort,
{
    #[inline]
    pub const fn new(applier: A) -> Self {
        Self { applier }
    }

    #[inline]
    pub const fn applier(&self) -> &A {
        &self.applier
    }

    #[inline]
    pub fn autofix_response(&self, raw_response: &str, preset: &Preset) -> (String, bool, bool) {
        self.applier.autofix_with_config(raw_response, &preset.template, &preset.parser)
    }

    #[inline]
    pub fn build_plan(
        &self,
        project_root: Option<&Path>,
        raw_response: &str,
        preset: &Preset,
    ) -> ApplyPlan {
        self.applier.build_apply_plan(project_root, raw_response, &preset.template, &preset.parser)
    }

    #[inline]
    pub fn execute_plan(
        &self,
        project_root: &Path,
        plan: &ApplyPlan,
        create_backup: bool,
    ) -> Result<BackupManifest, ApplierPortError> {
        self.applier.execute_apply_plan(project_root, plan, create_backup)
    }

    #[inline]
    pub fn undo_last(&self, project_root: &Path) -> Result<usize, ApplierPortError> {
        self.applier.undo_last_backup(project_root)
    }
}
