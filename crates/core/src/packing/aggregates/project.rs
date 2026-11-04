//! Корень агрегата проекта для упаковки контекста (`Project`).

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use crate::packing::aggregates::file_entry::FileEntry;
use crate::preset::models::preset::Preset;
use crate::shared_kernel::execution::anomaly::Anomaly;

/// Сравнивает два относительных пути по правилу «папки перед файлами на каждом уровне иерархии»
/// без единой аллокации памяти.
#[inline]
fn compare_entry_with_dir_prefix(entry_path: &str, dir_prefix: &str) -> Ordering {
    let mut rem_e = entry_path;
    let mut rem_p = dir_prefix;
    loop {
        let (seg_p, next_p) = match rem_p.split_once('/') {
            Some((head, tail)) => (head, Some(tail)),
            None => (rem_p, None),
        };
        let (seg_e, next_e) = match rem_e.split_once('/') {
            Some((head, tail)) => (head, Some(tail)),
            None => (rem_e, None),
        };
        if next_e.is_none() {
            return Ordering::Greater;
        }
        let cmp = seg_e.cmp(seg_p);
        if cmp != Ordering::Equal {
            return cmp;
        }
        if let Some(np) = next_p {
            rem_p = np;
            rem_e = next_e.unwrap_or("");
        } else {
            return Ordering::Equal;
        }
    }
}

#[inline]
pub fn compare_paths_folders_first(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let mut rem_a = a;
    let mut rem_b = b;
    loop {
        let (seg_a, next_a) = match rem_a.split_once('/') {
            Some((head, tail)) => (head, Some(tail)),
            None => (rem_a, None),
        };
        let (seg_b, next_b) = match rem_b.split_once('/') {
            Some((head, tail)) => (head, Some(tail)),
            None => (rem_b, None),
        };

        match (next_a.is_some(), next_b.is_some()) {
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            (true, true) => {
                let cmp = seg_a.cmp(seg_b);
                if cmp != Ordering::Equal {
                    return cmp;
                }
                rem_a = next_a.unwrap_or("");
                rem_b = next_b.unwrap_or("");
            }
            (false, false) => return seg_a.cmp(seg_b),
        }
    }
}

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
        entries.sort_unstable_by(|a, b| compare_paths_folders_first(&a.rel_path, &b.rel_path));
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
        let Ok(idx) = self
            .entries
            .binary_search_by(|e| compare_paths_folders_first(e.rel_path.as_str(), rel_path))
        else {
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
        let has_allowed_secrets = !self.preset.explicitly_allowed_secrets.is_empty();
        let allowed_secrets = &self.preset.explicitly_allowed_secrets;
        let start_idx = if clean_prefix.is_empty() {
            0
        } else {
            self.entries.partition_point(|e| {
                compare_entry_with_dir_prefix(&e.rel_path, clean_prefix) == Ordering::Less
            })
        };
        let mut changed_count = 0;
        let Some(slice) = self.entries.get_mut(start_idx..) else {
            return 0;
        };
        for entry in slice {
            if !clean_prefix.is_empty() {
                let inside = entry.rel_path.starts_with(clean_prefix)
                    && entry.rel_path.as_bytes().get(clean_prefix.len()).copied() == Some(b'/');
                if !inside {
                    break;
                }
            }
            let is_allowed_secret =
                has_allowed_secrets && allowed_secrets.iter().any(|s| s == &entry.rel_path);
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
