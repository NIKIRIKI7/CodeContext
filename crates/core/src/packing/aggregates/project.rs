//! Корень агрегата проекта для упаковки контекста (`Project`).

use std::path::{Path, PathBuf};

use crate::packing::aggregates::file_entry::FileEntry;
use crate::preset::models::preset::Preset;
use crate::shared_kernel::execution::anomaly::Anomaly;

/// Корень агрегата проекта, управляющий деревом файлов и инвариантами их выбора.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    root: PathBuf,
    entries: Vec<FileEntry>,
    preset: Preset,
    anomalies: Vec<Anomaly>,
    selected_files_count: usize,
    selected_bytes: u64,
}

impl Project {
    /// Создаёт агрегат проекта из отсортированного индекса файлов, пресета и списка аномалий.
    pub fn new(
        root: PathBuf,
        mut entries: Vec<FileEntry>,
        preset: Preset,
        anomalies: Vec<Anomaly>,
    ) -> Self {
        entries.sort_unstable_by(|a, b| a.rel_path.cmp(&b.rel_path));
        let mut selected_files_count = 0;
        let mut selected_bytes = 0_u64;
        for entry in &entries {
            if entry.is_selected {
                selected_files_count += 1;
                selected_bytes = selected_bytes.saturating_add(entry.size_bytes);
            }
        }
        Self { root, entries, preset, anomalies, selected_files_count, selected_bytes }
    }

    /// Возвращает корневую директорию проекта.
    #[inline]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Возвращает срез всех проиндексированных узлов файлов.
    #[inline]
    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    /// Возвращает активный пресет проекта.
    #[inline]
    pub const fn preset(&self) -> &Preset {
        &self.preset
    }

    /// Возвращает список обнаруженных при индексации аномалий.
    #[inline]
    pub fn anomalies(&self) -> &[Anomaly] {
        &self.anomalies
    }

    /// Возвращает текущее количество выбранных файлов (пересчитывается мгновенно).
    #[inline]
    pub const fn selected_files_count(&self) -> usize {
        self.selected_files_count
    }

    /// Возвращает суммарный размер выбранных файлов в байтах (без учёта маркеров шаблона).
    #[inline]
    pub const fn selected_bytes(&self) -> u64 {
        self.selected_bytes
    }

    /// Оценивает итоговый размер упакованного буфера в байтах (с учётом маркеров шаблона) за $O(1)$.
    #[inline]
    pub fn estimated_pack_capacity_bytes(&self) -> u64 {
        let marker_overhead =
            (self.selected_files_count * self.preset.template.fixed_marker_bytes_per_file()) as u64;
        self.selected_bytes.saturating_add(marker_overhead)
    }

    /// Переключает выбор одного файла по относительному пути за $O(\log N)$ (через бинарный поиск).
    /// Возвращает `true`, если состояние файла изменилось.
    pub fn set_file_selected(&mut self, rel_path: &str, selected: bool) -> bool {
        let Ok(idx) = self.entries.binary_search_by(|e| e.rel_path.as_str().cmp(rel_path)) else {
            return false;
        };
        let is_allowed_secret = self.preset.is_secret_explicitly_allowed(rel_path);
        let Some(entry) = self.entries.get_mut(idx) else {
            return false;
        };
        let size = entry.size_bytes;
        if !entry.set_selected(selected, is_allowed_secret) {
            return false;
        }
        if entry.is_selected {
            self.selected_files_count += 1;
            self.selected_bytes = self.selected_bytes.saturating_add(size);
        } else {
            self.selected_files_count = self.selected_files_count.saturating_sub(1);
            self.selected_bytes = self.selected_bytes.saturating_sub(size);
        }
        true
    }

    /// Каскадно включает или выключает все файлы внутри указанной директории (`dir_prefix`)
    /// без обращения к диску (выполняется быстрее `1 мс` на 50 000 файлов для Сценария A2).
    /// Возвращает число файлов, изменивших своё состояние.
    pub fn set_directory_selected(&mut self, dir_prefix: &str, selected: bool) -> usize {
        let clean_prefix = dir_prefix.trim_matches('/');
        let mut changed_count = 0;

        for entry in &mut self.entries {
            let matches_dir = if clean_prefix.is_empty() {
                true
            } else {
                entry.rel_path.starts_with(clean_prefix)
                    && entry.rel_path.as_bytes().get(clean_prefix.len()).copied() == Some(b'/')
            };

            if !matches_dir {
                continue;
            }

            let is_allowed_secret =
                self.preset.explicitly_allowed_secrets.iter().any(|s| s == &entry.rel_path);
            let size = entry.size_bytes;
            if entry.set_selected(selected, is_allowed_secret) {
                changed_count += 1;
                if entry.is_selected {
                    self.selected_files_count += 1;
                    self.selected_bytes = self.selected_bytes.saturating_add(size);
                } else {
                    self.selected_files_count = self.selected_files_count.saturating_sub(1);
                    self.selected_bytes = self.selected_bytes.saturating_sub(size);
                }
            }
        }

        changed_count
    }

    /// Возвращает список относительных путей всех выбранных пользователем файлов в алфавитном порядке.
    #[inline]
    pub fn selected_rel_paths(&self) -> Vec<String> {
        self.entries.iter().filter(|e| e.is_selected).map(|e| e.rel_path.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Project;
    use crate::packing::aggregates::file_entry::FileEntry;
    use crate::preset::models::preset::Preset;
    use crate::shared_kernel::fs::file_kind::FileKind;

    #[test]
    fn toggling_directory_updates_counts_and_respects_locked_secrets() {
        let entries = vec![
            FileEntry::new("crates/ui/src/lib.rs".to_owned(), 1000, FileKind::SourceCode),
            FileEntry::new("crates/ui/src/view.rs".to_owned(), 2000, FileKind::SourceCode),
            FileEntry::new("crates/core/src/lib.rs".to_owned(), 500, FileKind::SourceCode),
            FileEntry::new_blocked_secret(
                "crates/ui/.env".to_owned(),
                120,
                "DotEnvFile".to_owned(),
            ),
        ];

        let mut project =
            Project::new(PathBuf::from("/workspace"), entries, Preset::default(), Vec::new());

        assert_eq!(project.selected_files_count(), 3);
        assert_eq!(project.selected_bytes(), 3500);

        // Выключаем директорию crates/ui
        let changed = project.set_directory_selected("crates/ui", false);
        assert_eq!(changed, 2);
        assert_eq!(project.selected_files_count(), 1);
        assert_eq!(project.selected_bytes(), 500);

        // Включаем обратно crates/ui — секрет crates/ui/.env НЕ должен включиться!
        let changed_back = project.set_directory_selected("crates/ui", true);
        assert_eq!(changed_back, 2);
        assert_eq!(project.selected_files_count(), 3);
        assert_eq!(project.selected_bytes(), 3500);

        assert!(!project.set_file_selected("crates/ui/.env", true));
    }
}
