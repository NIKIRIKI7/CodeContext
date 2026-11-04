use std::path::Path;

#[inline]
pub(crate) fn check_dirty_git_files(
    _project_root: &Path,
    _target_rel_paths: &[String],
) -> Vec<String> {
    Vec::new()
}
